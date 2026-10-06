use std::collections::HashMap;
use std::error::Error;

use super::Dependency;

/// Parsea las dependencias declaradas en un archivo POM en formato XML,
/// resolviendo variables de propiedades e ignorando secciones que no son dependencias directas
/// (como dependencyManagement, build/plugins, etc.).
pub fn parse_pom_dependencies(
    xml_content: &str,
) -> Result<Vec<Dependency>, Box<dyn Error + Send + Sync>> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut properties: HashMap<String, String> = HashMap::new();
    let mut project_version = String::new();
    let mut parent_version = String::new();
    let mut project_group = String::new();
    let mut parent_group = String::new();

    let mut tag_stack: Vec<String> = Vec::new();
    let mut raw_dependencies = Vec::new();

    let mut curr_group = String::new();
    let mut curr_artifact = String::new();
    let mut curr_version = String::new();
    let mut curr_scope = None;
    let mut curr_optional = false;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "dependency" && is_in_project_dependencies(&tag_stack) {
                    curr_group.clear();
                    curr_artifact.clear();
                    curr_version.clear();
                    curr_scope = None;
                    curr_optional = false;
                }
                tag_stack.push(name);
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "dependency" && is_in_project_dependencies(&tag_stack) {
                    if !curr_group.is_empty() && !curr_artifact.is_empty() {
                        raw_dependencies.push(Dependency {
                            group_id: curr_group.clone(),
                            artifact_id: curr_artifact.clone(),
                            version: curr_version.clone(),
                            scope: curr_scope.clone(),
                            optional: curr_optional,
                        });
                    }
                }
                if let Some(pos) = tag_stack.iter().rposition(|x| x == &name) {
                    tag_stack.truncate(pos);
                }
            }
            Ok(Event::Text(e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).trim().to_string();
                if !text.is_empty() {
                    if is_in_properties(&tag_stack) {
                        if let Some(prop_tag) = tag_stack.last() {
                            properties.insert(prop_tag.clone(), text.clone());
                        }
                    } else if is_in_dependency(&tag_stack) && is_in_project_dependencies(&tag_stack) {
                        if let Some(current_tag) = tag_stack.last() {
                            match current_tag.as_str() {
                                "groupId" => curr_group = text.clone(),
                                "artifactId" => curr_artifact = text.clone(),
                                "version" => curr_version = text.clone(),
                                "scope" => curr_scope = Some(text.clone()),
                                "optional" => curr_optional = text.eq_ignore_ascii_case("true"),
                                _ => {}
                            }
                        }
                    } else if is_project_version(&tag_stack) {
                        project_version = text.clone();
                    } else if is_parent_version(&tag_stack) {
                        parent_version = text.clone();
                    } else if is_project_group(&tag_stack) {
                        project_group = text.clone();
                    } else if is_parent_group(&tag_stack) {
                        parent_group = text.clone();
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
        buf.clear();
    }

    let final_version = if !project_version.is_empty() {
        project_version
    } else {
        parent_version
    };
    let final_group = if !project_group.is_empty() {
        project_group
    } else {
        parent_group
    };

    if !final_version.is_empty() {
        properties.entry("project.version".to_string()).or_insert(final_version.clone());
        properties.entry("pom.version".to_string()).or_insert(final_version.clone());
        properties.entry("version".to_string()).or_insert(final_version.clone());
    }
    if !final_group.is_empty() {
        properties.entry("project.groupId".to_string()).or_insert(final_group.clone());
        properties.entry("pom.groupId".to_string()).or_insert(final_group.clone());
        properties.entry("groupId".to_string()).or_insert(final_group.clone());
    }

    let mut dependencies = Vec::new();
    for mut dep in raw_dependencies {
        dep.version = resolve_properties(&dep.version, &properties);
        // Solo agregar dependencias que tengan versión válida (no vacía ni placeholder no resuelto)
        if !dep.version.is_empty() && !dep.version.starts_with("${") {
            dependencies.push(dep);
        }
    }

    Ok(dependencies)
}

fn is_in_properties(stack: &[String]) -> bool {
    stack.len() >= 3 && stack[stack.len() - 2] == "properties"
}

fn is_in_dependency(stack: &[String]) -> bool {
    stack.iter().any(|t| t == "dependency")
}

fn is_in_project_dependencies(stack: &[String]) -> bool {
    let in_deps = stack.iter().any(|t| t == "dependencies");
    let in_excluded = stack.iter().any(|t| {
        t == "dependencyManagement" || t == "plugin" || t == "plugins" || t == "build"
    });
    in_deps && !in_excluded
}

fn is_project_version(stack: &[String]) -> bool {
    stack.len() == 2 && stack[0] == "project" && stack[1] == "version"
}

fn is_parent_version(stack: &[String]) -> bool {
    stack.len() == 3 && stack[0] == "project" && stack[1] == "parent" && stack[2] == "version"
}

fn is_project_group(stack: &[String]) -> bool {
    stack.len() == 2 && stack[0] == "project" && stack[1] == "groupId"
}

fn is_parent_group(stack: &[String]) -> bool {
    stack.len() == 3 && stack[0] == "project" && stack[1] == "parent" && stack[2] == "groupId"
}

fn resolve_properties(input: &str, properties: &HashMap<String, String>) -> String {
    let mut result = input.trim().to_string();
    for _ in 0..5 {
        if !result.contains("${") {
            break;
        }
        let mut changed = false;
        for (k, v) in properties {
            let placeholder = format!("${{{}}}", k);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, v);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    result
}

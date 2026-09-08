use std::error::Error;

use super::Dependency;

/// Parsea las dependencias declaradas en un archivo POM en formato XML
pub fn parse_pom_dependencies(
    xml_content: &str,
) -> Result<Vec<Dependency>, Box<dyn Error + Send + Sync>> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut dependencies = Vec::new();
    let mut in_dependencies = false;
    let mut in_dependency = false;
    let mut current_tag = String::new();

    let mut curr_group = String::new();
    let mut curr_artifact = String::new();
    let mut curr_version = String::new();
    let mut curr_scope = None;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "dependencies" {
                    in_dependencies = true;
                } else if in_dependencies && name == "dependency" {
                    in_dependency = true;
                    curr_group.clear();
                    curr_artifact.clear();
                    curr_version.clear();
                    curr_scope = None;
                }
                current_tag = name;
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "dependencies" {
                    in_dependencies = false;
                } else if in_dependencies && name == "dependency" {
                    in_dependency = false;
                    if !curr_group.is_empty() && !curr_artifact.is_empty() {
                        dependencies.push(Dependency {
                            group_id: curr_group.clone(),
                            artifact_id: curr_artifact.clone(),
                            version: curr_version.clone(),
                            scope: curr_scope.clone(),
                        });
                    }
                }
                current_tag.clear();
            }
            Ok(Event::Text(e)) => {
                if in_dependency {
                    let text = String::from_utf8_lossy(e.as_ref()).trim().to_string();
                    match current_tag.as_str() {
                        "groupId" => curr_group = text,
                        "artifactId" => curr_artifact = text,
                        "version" => curr_version = text,
                        "scope" => curr_scope = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(dependencies)
}

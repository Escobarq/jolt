use dialoguer::{theme::ColorfulTheme, Input, Select};
use serde_json::{json, Value};
use std::fs;
use std::io::IsTerminal;
use std::path::Path;

pub const AVAILABLE_TEMPLATES: &[(&str, &str)] = &[
    ("minimal", "Proyecto estandar Java 21 con JUnit 5 integrado"),
    ("cli", "Aplicacion de linea de comandos con Picocli"),
    ("javafx", "Aplicacion con interfaz grafica moderna en JavaFX 21 y CSS"),
    ("swing", "Aplicacion de escritorio Java Swing con Look & Feel moderno (FlatLaf)"),
    ("web", "Microservicio / API REST ligera con Javalin en puerto 7070"),
    ("spring", "Aplicacion web completa con Spring Boot 3.2 y REST Controller"),
];

pub fn print_available_templates() {
    println!("Plantillas disponibles para 'jolt init --template <nombre>':");
    for (name, desc) in AVAILABLE_TEMPLATES {
        println!("  - {:<10} : {}", name, desc);
    }
}

/// Genera y actualiza la configuración completa para que VS Code, Cursor, Eclipse y Language Servers Java reconozcan los JARs, fuentes y archivos TOML
pub fn ensure_ide_configuration(
    project_dir: &Path,
    project_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolved_name = project_name
        .map(|s| s.to_string())
        .or_else(|| {
            let manifest_path = project_dir.join("jolt.toml");
            if manifest_path.exists() {
                crate::manifest::JoltManifest::load_from_file(&manifest_path)
                    .ok()
                    .map(|m| m.project.name)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            project_dir
                .canonicalize()
                .ok()
                .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
                .unwrap_or_else(|| "app".to_string())
        });

    // 1. .vscode/settings.json
    let vscode_dir = project_dir.join(".vscode");
    fs::create_dir_all(&vscode_dir)?;
    let settings_path = vscode_dir.join("settings.json");

    let mut settings_json: Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    if !settings_json.is_object() {
        settings_json = json!({});
    }

    if let Some(obj) = settings_json.as_object_mut() {
        // Files associations para sintaxis y soporte TOML en VS Code
        let mut fa = obj
            .get("files.associations")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        fa.insert("jolt.toml".to_string(), json!("toml"));
        fa.insert("jolt.lock".to_string(), json!("toml"));
        obj.insert("files.associations".to_string(), Value::Object(fa));

        // Referenced libraries (.jolt/modules y .jolt/dev-modules)
        let mut libs: Vec<String> = vec![
            ".jolt/modules/**/*.jar".to_string(),
            ".jolt/dev-modules/**/*.jar".to_string(),
            "lib/**/*.jar".to_string(),
        ];
        if let Some(existing_libs) = obj.get("java.project.referencedLibraries").and_then(|v| v.as_array()) {
            for item in existing_libs {
                if let Some(s) = item.as_str() {
                    if !libs.contains(&s.to_string()) {
                        libs.push(s.to_string());
                    }
                }
            }
        }
        obj.insert("java.project.referencedLibraries".to_string(), json!(libs));

        // Source paths
        let mut source_paths = Vec::new();
        if project_dir.join("src/main/java").exists() || !project_dir.join("src").exists() {
            source_paths.push("src/main/java".to_string());
        }
        if project_dir.join("src/test/java").exists() || !project_dir.join("src").exists() {
            source_paths.push("src/test/java".to_string());
        }
        if source_paths.is_empty() && project_dir.join("src").exists() {
            source_paths.push("src".to_string());
        }
        obj.insert("java.project.sourcePaths".to_string(), json!(source_paths));

        // Output path
        if !obj.contains_key("java.project.outputPath") {
            obj.insert("java.project.outputPath".to_string(), json!("target/classes"));
        }

        // Automatic build configuration updates
        obj.insert(
            "java.configuration.updateBuildConfiguration".to_string(),
            json!("automatic"),
        );
    }

    fs::write(&settings_path, serde_json::to_string_pretty(&settings_json)? + "\n")?;

    // 2. .vscode/extensions.json (Recomendaciones de extensiones para Java y TOML)
    let extensions_path = vscode_dir.join("extensions.json");
    let mut ext_json: Value = if extensions_path.exists() {
        let content = fs::read_to_string(&extensions_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    if !ext_json.is_object() {
        ext_json = json!({});
    }

    let default_recs = [
        "vscjava.vscode-java-pack",
        "redhat.java",
        "vscjava.vscode-java-dependency",
        "tamasfe.even-better-toml",
    ];

    if let Some(obj) = ext_json.as_object_mut() {
        let mut recs: Vec<String> = obj
            .get("recommendations")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        for rec in &default_recs {
            if !recs.contains(&rec.to_string()) {
                recs.push(rec.to_string());
            }
        }
        obj.insert("recommendations".to_string(), json!(recs));
    }

    fs::write(&extensions_path, serde_json::to_string_pretty(&ext_json)? + "\n")?;

    // 3. .project (Descriptor Eclipse / Java Language Server para VS Code)
    let project_file = project_dir.join(".project");
    let project_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<projectDescription>
	<name>{}</name>
	<comment>Generated by Jolt</comment>
	<projects>
	</projects>
	<buildSpec>
		<buildCommand>
			<name>org.eclipse.jdt.core.javabuilder</name>
			<arguments>
			</arguments>
		</buildCommand>
	</buildSpec>
	<natures>
		<nature>org.eclipse.jdt.core.javanature</nature>
	</natures>
</projectDescription>
"#,
        resolved_name
    );
    fs::write(project_file, project_content)?;

    // 4. .classpath (Descriptor Classpath para Language Server)
    let classpath_file = project_dir.join(".classpath");
    let mut cp_entries = Vec::new();

    // Carpetas fuente
    if project_dir.join("src/main/java").exists() {
        cp_entries.push(r#"	<classpathentry kind="src" path="src/main/java"/>"#.to_string());
    }
    if project_dir.join("src/main/resources").exists() {
        cp_entries.push(r#"	<classpathentry kind="src" path="src/main/resources"/>"#.to_string());
    }
    if project_dir.join("src/test/java").exists() {
        cp_entries.push(r#"	<classpathentry kind="src" output="target/test-classes" path="src/test/java"/>"#.to_string());
    }
    if project_dir.join("src/test/resources").exists() {
        cp_entries.push(r#"	<classpathentry kind="src" output="target/test-classes" path="src/test/resources"/>"#.to_string());
    }
    if cp_entries.is_empty() && project_dir.join("src").exists() {
        cp_entries.push(r#"	<classpathentry kind="src" path="src"/>"#.to_string());
    }
    if cp_entries.is_empty() {
        cp_entries.push(r#"	<classpathentry kind="src" path="src/main/java"/>"#.to_string());
        cp_entries.push(r#"	<classpathentry kind="src" output="target/test-classes" path="src/test/java"/>"#.to_string());
    }

    // JRE Container
    cp_entries.push(r#"	<classpathentry kind="con" path="org.eclipse.jdt.launching.JRE_CONTAINER"/>"#.to_string());

    // JARs en .jolt/modules y .jolt/dev-modules
    let mut jar_paths = Vec::new();
    let modules_dir = project_dir.join(".jolt").join("modules");
    if modules_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&modules_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("jar") {
                    jar_paths.push(format!(".jolt/modules/{}", entry.file_name().to_string_lossy()));
                }
            }
        }
    }
    let dev_modules_dir = project_dir.join(".jolt").join("dev-modules");
    if dev_modules_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&dev_modules_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("jar") {
                    jar_paths.push(format!(".jolt/dev-modules/{}", entry.file_name().to_string_lossy()));
                }
            }
        }
    }

    jar_paths.sort();
    for jar in jar_paths {
        cp_entries.push(format!(r#"	<classpathentry kind="lib" path="{}"/>"#, jar));
    }

    // Output dir
    cp_entries.push(r#"	<classpathentry kind="output" path="target/classes"/>"#.to_string());

    let classpath_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<classpath>
{}
</classpath>
"#,
        cp_entries.join("\n")
    );
    fs::write(classpath_file, classpath_content)?;

    // 5. .gitignore estándar (si no existe)
    let gitignore_path = project_dir.join(".gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = r#"target/
.jolt/modules/
.jolt/dev-modules/
*.jar
.classpath
.project
.settings/
bin/
"#;
        let _ = fs::write(gitignore_path, gitignore_content);
    }

    Ok(())
}


pub fn init_project(
    name: Option<&str>,
    template: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let is_interactive = std::io::stdin().is_terminal();

    // 1. Resolver el nombre del proyecto
    let project_name = if let Some(n) = name {
        n.to_string()
    } else if is_interactive {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Nombre del proyecto")
            .default("app".to_string())
            .interact_text()?
    } else {
        "app".to_string()
    };

    // 2. Resolver la plantilla
    let tmpl = if let Some(t) = template {
        t.to_lowercase()
    } else if is_interactive && name.is_none() {
        let items: Vec<String> = AVAILABLE_TEMPLATES
            .iter()
            .map(|(n, d)| format!("{:<10} - {}", n, d))
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Selecciona una plantilla de inicio")
            .items(&items)
            .default(0)
            .interact()?;

        AVAILABLE_TEMPLATES[selection].0.to_string()
    } else {
        "minimal".to_string()
    };

    let base_dir = Path::new(&project_name);

    let valid_templates = ["minimal", "cli", "javafx", "swing", "web", "spring", "spring-boot"];
    if !valid_templates.contains(&tmpl.as_str()) {
        println!("[ERROR] Plantilla '{}' no reconocida.", tmpl);
        print_available_templates();
        return Ok(());
    }

    if base_dir.exists() {
        println!("[WARN] El directorio '{}' ya existe.", project_name);
        return Ok(());
    }

    // Crear la estructura de carpetas estándar
    fs::create_dir_all(base_dir.join("src/main/java"))?;
    fs::create_dir_all(base_dir.join("src/main/resources"))?;
    fs::create_dir_all(base_dir.join("src/test/java"))?;

    // Configurar IDE para detección automática de dependencias y soporte TOML
    let _ = ensure_ide_configuration(base_dir, Some(&project_name));

    match tmpl.as_str() {
        "cli" => {
            let toml_content = include_str!("../templates/cli/jolt.toml").replace("{{project_name}}", &project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/cli/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/test/java/MainTest.java"), include_str!("../templates/cli/src/test/java/MainTest.java"))?;
        }
        "javafx" => {
            let toml_content = include_str!("../templates/javafx/jolt.toml").replace("{{project_name}}", &project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/App.java"), include_str!("../templates/javafx/src/main/java/App.java"))?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/javafx/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/main/resources/style.css"), include_str!("../templates/javafx/src/main/resources/style.css"))?;
        }
        "swing" => {
            let toml_content = include_str!("../templates/swing/jolt.toml").replace("{{project_name}}", &project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/swing/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/main/resources/app.properties"), include_str!("../templates/swing/src/main/resources/app.properties"))?;
            fs::write(base_dir.join("src/test/java/SwingAppTest.java"), include_str!("../templates/swing/src/test/java/SwingAppTest.java"))?;
        }
        "web" => {
            let toml_content = include_str!("../templates/web/jolt.toml").replace("{{project_name}}", &project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/web/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/main/resources/application.properties"), include_str!("../templates/web/src/main/resources/application.properties"))?;
        }
        "spring" | "spring-boot" => {
            let toml_content = include_str!("../templates/spring/jolt.toml").replace("{{project_name}}", &project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/spring/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/main/resources/application.properties"), include_str!("../templates/spring/src/main/resources/application.properties"))?;
            fs::write(base_dir.join("src/test/java/SpringAppTest.java"), include_str!("../templates/spring/src/test/java/SpringAppTest.java"))?;
        }
        _ => {
            // Plantilla minimal por defecto
            let toml_content = include_str!("../templates/minimal/jolt.toml").replace("{{project_name}}", &project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/minimal/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/test/java/AppTest.java"), include_str!("../templates/minimal/src/test/java/AppTest.java"))?;
        }
    }

    println!("[OK] Proyecto '{}' inicializado correctamente (Plantilla: '{}').", project_name, tmpl);
    println!("     Sugerencia: Ejecuta 'cd {} && jolt install' para sincronizar librerias.", project_name);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_ide_configuration_creates_files() {
        let temp_dir = std::env::temp_dir().join("jolt_ide_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(temp_dir.join("src/main/java")).unwrap();
        fs::create_dir_all(temp_dir.join(".jolt/modules")).unwrap();
        fs::write(temp_dir.join(".jolt/modules/guava-33.0.0.jar"), b"dummy").unwrap();

        ensure_ide_configuration(&temp_dir, Some("mi-proyecto-test")).expect("Failed to configure IDE");

        // Verify .vscode/settings.json
        let settings_path = temp_dir.join(".vscode/settings.json");
        assert!(settings_path.exists());
        let settings_str = fs::read_to_string(&settings_path).unwrap();
        assert!(settings_str.contains("\"files.associations\""));
        assert!(settings_str.contains("\"jolt.toml\": \"toml\""));
        assert!(settings_str.contains("\"java.project.referencedLibraries\""));
        assert!(settings_str.contains("\"java.project.sourcePaths\""));

        // Verify .vscode/extensions.json
        let ext_path = temp_dir.join(".vscode/extensions.json");
        assert!(ext_path.exists());
        let ext_str = fs::read_to_string(&ext_path).unwrap();
        assert!(ext_str.contains("vscjava.vscode-java-pack"));
        assert!(ext_str.contains("tamasfe.even-better-toml"));

        // Verify .project
        let project_path = temp_dir.join(".project");
        assert!(project_path.exists());
        let project_str = fs::read_to_string(&project_path).unwrap();
        assert!(project_str.contains("<name>mi-proyecto-test</name>"));
        assert!(project_str.contains("org.eclipse.jdt.core.javanature"));

        // Verify .classpath
        let cp_path = temp_dir.join(".classpath");
        assert!(cp_path.exists());
        let cp_str = fs::read_to_string(&cp_path).unwrap();
        assert!(cp_str.contains("kind=\"src\" path=\"src/main/java\""));
        assert!(cp_str.contains("kind=\"lib\" path=\".jolt/modules/guava-33.0.0.jar\""));
        assert!(cp_str.contains("kind=\"output\" path=\"target/classes\""));

        // Verify .gitignore
        let gitignore_path = temp_dir.join(".gitignore");
        assert!(gitignore_path.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}


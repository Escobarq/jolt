use serde_json::{json, Value};
use std::fs;
use std::path::Path;

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
                crate::core::manifest::JoltManifest::load_from_file(&manifest_path)
                    .ok()
                    .and_then(|m| m.project.map(|p| p.name))
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
        let mut referenced_libs = Vec::new();
        let modules_dir = project_dir.join(".jolt").join("modules");
        if modules_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&modules_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().and_then(|s| s.to_str()) == Some("jar") {
                        referenced_libs.push(format!(".jolt/modules/{}", entry.file_name().to_string_lossy()));
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
                        referenced_libs.push(format!(".jolt/dev-modules/{}", entry.file_name().to_string_lossy()));
                    }
                }
            }
        }
        referenced_libs.sort();
        obj.insert(
            "java.project.referencedLibraries".to_string(),
            json!(referenced_libs),
        );

        // Source paths
        let mut source_paths = Vec::new();
        if project_dir.join("src/main/java").exists() {
            source_paths.push("src/main/java".to_string());
        }
        if project_dir.join("src/main/resources").exists() {
            source_paths.push("src/main/resources".to_string());
        }
        if project_dir.join("src/test/java").exists() {
            source_paths.push("src/test/java".to_string());
        }
        if project_dir.join("src/test/resources").exists() {
            source_paths.push("src/test/resources".to_string());
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

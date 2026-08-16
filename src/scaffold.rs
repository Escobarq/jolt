use std::fs;
use std::path::Path;

pub fn init_project(name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let project_name = name.unwrap_or("app");
    let base_dir = Path::new(project_name);

    if base_dir.exists() {
        println!("[WARN] El directorio '{}' ya existe.", project_name);
        return Ok(());
    }

    // Crear la estructura de carpetas
    fs::create_dir_all(base_dir.join("src/main/java"))?;
    fs::create_dir_all(base_dir.join("src/main/resources"))?;
    fs::create_dir_all(base_dir.join("src/test/java"))?;
    
    // Crear el archivo jolt.toml
    let toml_content = format!(
        r#"[project]
name = "{}"
version = "0.1.0"
java_version = "21"

[dependencies]

[dev-dependencies]
"#, project_name
    );
    fs::write(base_dir.join("jolt.toml"), toml_content)?;

    // Crear Main.java básico
    let java_content = r#"public class Main {
    public static void main(String[] args) {
        System.out.println("Hola desde Jolt!");
    }
}
"#;
    fs::write(base_dir.join("src/main/java/Main.java"), java_content)?;

    println!("[OK] Proyecto '{}' inicializado correctamente.", project_name);

    Ok(())
}

use std::fs;
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

pub fn init_project(name: Option<&str>, template: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let project_name = name.unwrap_or("app");
    let base_dir = Path::new(project_name);
    let tmpl = template.unwrap_or("minimal").to_lowercase();

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

    match tmpl.as_str() {
        "cli" => {
            let toml_content = include_str!("../templates/cli/jolt.toml").replace("{{project_name}}", project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/cli/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/test/java/MainTest.java"), include_str!("../templates/cli/src/test/java/MainTest.java"))?;
        }
        "javafx" => {
            let toml_content = include_str!("../templates/javafx/jolt.toml").replace("{{project_name}}", project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/App.java"), include_str!("../templates/javafx/src/main/java/App.java"))?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/javafx/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/main/resources/style.css"), include_str!("../templates/javafx/src/main/resources/style.css"))?;
        }
        "swing" => {
            let toml_content = include_str!("../templates/swing/jolt.toml").replace("{{project_name}}", project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/swing/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/main/resources/app.properties"), include_str!("../templates/swing/src/main/resources/app.properties"))?;
            fs::write(base_dir.join("src/test/java/SwingAppTest.java"), include_str!("../templates/swing/src/test/java/SwingAppTest.java"))?;
        }
        "web" => {
            let toml_content = include_str!("../templates/web/jolt.toml").replace("{{project_name}}", project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/web/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/main/resources/application.properties"), include_str!("../templates/web/src/main/resources/application.properties"))?;
        }
        "spring" | "spring-boot" => {
            let toml_content = include_str!("../templates/spring/jolt.toml").replace("{{project_name}}", project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/spring/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/main/resources/application.properties"), include_str!("../templates/spring/src/main/resources/application.properties"))?;
            fs::write(base_dir.join("src/test/java/SpringAppTest.java"), include_str!("../templates/spring/src/test/java/SpringAppTest.java"))?;
        }
        _ => {
            // Plantilla minimal por defecto
            let toml_content = include_str!("../templates/minimal/jolt.toml").replace("{{project_name}}", project_name);
            fs::write(base_dir.join("jolt.toml"), toml_content)?;
            fs::write(base_dir.join("src/main/java/Main.java"), include_str!("../templates/minimal/src/main/java/Main.java"))?;
            fs::write(base_dir.join("src/test/java/AppTest.java"), include_str!("../templates/minimal/src/test/java/AppTest.java"))?;
        }
    }

    println!("[OK] Proyecto '{}' inicializado correctamente (Plantilla: '{}').", project_name, tmpl);
    println!("     Sugerencia: Ejecuta 'cd {} && jolt install' para sincronizar librerias.", project_name);

    Ok(())
}

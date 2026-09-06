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
    package: Option<&str>,
    group_id: Option<&str>,
    is_workspace_flag: bool,
    enable_graalvm: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let is_interactive = std::io::stdin().is_terminal();
    let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let existing_ws = crate::manifest::JoltManifest::find_root_workspace(&current_dir);

    // 1. Si se pidió explícitamente workspace (--workspace)
    if is_workspace_flag {
        let ws_name = if let Some(n) = name {
            n.to_string()
        } else if is_interactive {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Nombre del Workspace Monorepo")
                .default("my-workspace".to_string())
                .interact_text()?
        } else {
            "my-workspace".to_string()
        };

        let base_dir = Path::new(&ws_name);
        if base_dir.exists() {
            println!("[WARN] El directorio '{}' ya existe.", ws_name);
            return Ok(());
        }

        fs::create_dir_all(base_dir)?;
        let manifest_path = base_dir.join("jolt.toml");
        crate::manifest::JoltManifest::create_workspace_file(&manifest_path)?;

        let _ = fs::write(
            base_dir.join(".gitignore"),
            "target/\n.jolt/\n*.jar\n.classpath\n.project\n.settings/\nbin/\ndist/\n",
        );

        println!("[OK] Workspace Monorepo '{}' creado exitosamente.", ws_name);
        println!("     Sugerencia: Ejecuta 'cd {}' y crea submódulos con 'jolt init <modulo>'.", ws_name);
        return Ok(());
    }

    // 2. Si es interactivo y no se pasaron argumentos por CLI
    let mut chosen_workspace = false;
    let mut module_context = None;

    if is_interactive && name.is_none() && template.is_none() && package.is_none() {
        if let Some((root_ws_dir, _)) = &existing_ws {
            println!("🏢 Workspace activo detectado en '{}'", root_ws_dir.display());
            println!("   Creando un nuevo módulo para este workspace...");
            module_context = Some(root_ws_dir.clone());
        } else {
            let project_types = [
                "📦 Proyecto Individual (Aplicación estándar con plantilla)",
                "🏢 Workspace Monorepo (Workspace multimódulo para múltiples proyectos)",
            ];
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("¿Qué tipo de proyecto deseas crear?")
                .items(&project_types)
                .default(0)
                .interact()?;

            if selection == 1 {
                chosen_workspace = true;
            }
        }
    }

    if chosen_workspace {
        let ws_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Nombre del Workspace Monorepo")
            .default("my-workspace".to_string())
            .interact_text()?;

        let base_dir = Path::new(&ws_name);
        if base_dir.exists() {
            println!("[WARN] El directorio '{}' ya existe.", ws_name);
            return Ok(());
        }

        fs::create_dir_all(base_dir)?;
        let manifest_path = base_dir.join("jolt.toml");
        crate::manifest::JoltManifest::create_workspace_file(&manifest_path)?;

        let _ = fs::write(
            base_dir.join(".gitignore"),
            "target/\n.jolt/\n*.jar\n.classpath\n.project\n.settings/\nbin/\ndist/\n",
        );

        println!("[OK] Workspace Monorepo '{}' creado exitosamente.", ws_name);
        println!("     Sugerencia: Ejecuta 'cd {}' y crea submódulos con 'jolt init <modulo>'.", ws_name);
        return Ok(());
    }

    // 3. Resolver nombre del proyecto y directorio base
    let (base_dir, project_name) = if let Some(n) = name {
        let p = Path::new(n);
        let base_name = p.file_name().and_then(|f| f.to_str()).unwrap_or(n);
        (p.to_path_buf(), base_name.to_string())
    } else if is_interactive {
        let prompt_text = if module_context.is_some() {
            "Nombre del módulo"
        } else {
            "Nombre del proyecto"
        };
        let input_name = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt_text)
            .default("app".to_string())
            .interact_text()?;
        let p = Path::new(&input_name);
        let base_name = p.file_name().and_then(|f| f.to_str()).unwrap_or(&input_name);
        (p.to_path_buf(), base_name.to_string())
    } else {
        (std::path::PathBuf::from("app"), "app".to_string())
    };

    // 4. Resolver Paquete Java / Namespace
    let sanitized_pkg_name: String = project_name.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect();
    let default_pkg = if existing_ws.is_some() {
        format!("org.app.{}", sanitized_pkg_name.to_lowercase())
    } else {
        format!("org.example.{}", sanitized_pkg_name.to_lowercase())
    };

    let resolved_pkg: Option<String> = if let Some(p) = package {
        let trimmed = p.trim().to_string();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    } else if is_interactive && name.is_none() {
        let pkg_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Paquete raíz Java (namespace FQCN)")
            .default(default_pkg)
            .interact_text()?;
        let trimmed = pkg_input.trim().to_string();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    } else {
        None
    };

    let resolved_group_id: Option<String> = group_id.map(|s| s.to_string()).or_else(|| {
        resolved_pkg.as_ref().map(|pkg| {
            let parts: Vec<&str> = pkg.split('.').collect();
            if parts.len() >= 2 {
                format!("{}.{}", parts[0], parts[1])
            } else {
                pkg.clone()
            }
        })
    });

    // 5. Resolver la plantilla
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

    let valid_templates = ["minimal", "cli", "javafx", "swing", "web", "spring", "spring-boot"];
    if !valid_templates.contains(&tmpl.as_str()) {
        println!("[ERROR] Plantilla '{}' no reconocida.", tmpl);
        print_available_templates();
        return Ok(());
    }

    let mut resolved_graalvm = enable_graalvm;
    if is_interactive && name.is_none() && !enable_graalvm {
        let graal_choices = [
            "☕ JVM Estándar (Ejecución rápida con bytecode en HotSpot)",
            "🚀 GraalVM Native Image (Binario ejecutable nativo AOT)",
        ];
        if let Ok(selection) = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("¿Deseas activar GraalVM Native Image por defecto?")
            .items(&graal_choices)
            .default(0)
            .interact()
        {
            if selection == 1 {
                resolved_graalvm = true;
            }
        }
    }

    if base_dir.exists() {
        println!("[WARN] El directorio '{}' ya existe.", project_name);
        return Ok(());
    }

    // Configurar rutas de directorios de fuentes según el paquete
    let (src_main_java, src_test_java, pkg_stmt, main_class_name) = if let Some(ref pkg) = resolved_pkg {
        let pkg_rel = pkg.replace('.', "/");
        (
            base_dir.join("src/main/java").join(&pkg_rel),
            base_dir.join("src/test/java").join(&pkg_rel),
            format!("package {};\n\n", pkg),
            format!("{}.Main", pkg),
        )
    } else {
        (
            base_dir.join("src/main/java"),
            base_dir.join("src/test/java"),
            String::new(),
            "Main".to_string(),
        )
    };

    // Crear la estructura de carpetas estándar
    fs::create_dir_all(&src_main_java)?;
    fs::create_dir_all(base_dir.join("src/main/resources"))?;
    fs::create_dir_all(&src_test_java)?;

    // Configurar IDE para detección automática de dependencias y soporte TOML
    let _ = ensure_ide_configuration(&base_dir, Some(&project_name));

    // Generar bloque project para jolt.toml
    let mut toml_project_header = format!(
        "[project]\nname = \"{}\"\nversion = \"0.1.0\"\njava_version = \"21\"",
        project_name
    );
    if let Some(ref pkg) = resolved_pkg {
        toml_project_header.push_str(&format!("\npackage = \"{}\"", pkg));
    }
    if let Some(ref gid) = resolved_group_id {
        toml_project_header.push_str(&format!("\ngroup_id = \"{}\"", gid));
    }
    toml_project_header.push_str(&format!("\nmain_class = \"{}\"", main_class_name));

    match tmpl.as_str() {
        "cli" => {
            let toml_content = format!(
                "{}\n\n[dependencies]\n\"info.picocli:picocli\" = \"4.7.6\"\n\n[dev-dependencies]\n\"org.junit.jupiter:junit-jupiter-api\" = \"5.10.2\"\n\n[graalvm]\nenabled = {}\nname = \"{}-cli\"\nargs = [\n    \"--no-fallback\",\n    \"-H:+ReportExceptionStackTraces\",\n    \"--initialize-at-build-time\"\n]\n",
                toml_project_header, resolved_graalvm, project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let main_content = format!(
                "{}import picocli.CommandLine;\nimport picocli.CommandLine.Command;\nimport picocli.CommandLine.Option;\nimport java.util.concurrent.Callable;\n\n@Command(name = \"{}\", mixinStandardHelpOptions = true, version = \"0.1.0\",\n        description = \"CLI desarrollada con Jolt y Picocli\")\npublic class Main implements Callable<Integer> {{\n\n    @Option(names = {{\"--name\", \"-n\"}}, description = \"Nombre a saludar\", defaultValue = \"Mundo\")\n    private String name;\n\n    @Override\n    public Integer call() {{\n        System.out.println(\"Hola, \" + name + \"! Bienvenido a tu CLI con Jolt.\");\n        return 0;\n    }}\n\n    public static void main(String[] args) {{\n        int exitCode = new CommandLine(new Main()).execute(args);\n        System.exit(exitCode);\n    }}\n}}\n",
                pkg_stmt, project_name
            );
            fs::write(src_main_java.join("Main.java"), main_content)?;

            let test_content = format!(
                "{}import org.junit.jupiter.api.Test;\nimport static org.junit.jupiter.api.Assertions.assertEquals;\n\npublic class MainTest {{\n    @Test\n    void testCliExecution() {{\n        Main app = new Main();\n        assertEquals(0, app.call());\n    }}\n}}\n",
                pkg_stmt
            );
            fs::write(src_test_java.join("MainTest.java"), test_content)?;
        }
        "javafx" => {
            let toml_content = format!(
                "{}\n\n[dependencies]\n\"org.openjfx:javafx-controls\" = \"21.0.2:linux\"\n\"org.openjfx:javafx-graphics\" = \"21.0.2:linux\"\n\"org.openjfx:javafx-base\" = \"21.0.2:linux\"\n\n[dev-dependencies]\n\"org.junit.jupiter:junit-jupiter-api\" = \"5.10.2\"\n\n[graalvm]\nenabled = {}\nname = \"{}-gui\"\nargs = [\n    \"--no-fallback\",\n    \"-H:+ReportExceptionStackTraces\"\n]\n",
                toml_project_header, resolved_graalvm, project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let app_content = format!(
                "{}import javafx.application.Application;\nimport javafx.geometry.Pos;\nimport javafx.scene.Scene;\nimport javafx.scene.control.Button;\nimport javafx.scene.control.Label;\nimport javafx.scene.layout.VBox;\nimport javafx.stage.Stage;\n\npublic class App extends Application {{\n\n    @Override\n    public void start(Stage primaryStage) {{\n        Label label = new Label(\"⚡ ¡Hola desde JavaFX 21 con Jolt!\");\n        label.getStyleClass().add(\"title-label\");\n\n        Button btn = new Button(\"Haz clic aquí\");\n        btn.setOnAction(e -> label.setText(\"🚀 ¡Jolt impulsa tu desarrollo en Java!\"));\n\n        VBox root = new VBox(20, label, btn);\n        root.setAlignment(Pos.CENTER);\n        root.getStyleClass().add(\"main-container\");\n\n        Scene scene = new Scene(root, 480, 320);\n        var css = getClass().getResource(\"/style.css\");\n        if (css != null) {{\n            scene.getStylesheets().add(css.toExternalForm());\n        }}\n\n        primaryStage.setTitle(\"Jolt - JavaFX App\");\n        primaryStage.setScene(scene);\n        primaryStage.show();\n    }}\n}}\n",
                pkg_stmt
            );
            fs::write(src_main_java.join("App.java"), app_content)?;

            let main_content = format!(
                "{}import javafx.application.Application;\n\npublic class Main {{\n    public static void main(String[] args) {{\n        Application.launch(App.class, args);\n    }}\n}}\n",
                pkg_stmt
            );
            fs::write(src_main_java.join("Main.java"), main_content)?;
            fs::write(base_dir.join("src/main/resources/style.css"), include_str!("../templates/javafx/src/main/resources/style.css"))?;
        }
        "swing" => {
            let toml_content = format!(
                "{}\n\n[dependencies]\n\"com.formdev:flatlaf\" = \"3.4.1\"\n\n[dev-dependencies]\n\"org.junit.jupiter:junit-jupiter-api\" = \"5.10.2\"\n\n[graalvm]\nenabled = {}\nname = \"{}-gui\"\nargs = [\n    \"--no-fallback\",\n    \"-H:+ReportExceptionStackTraces\"\n]\n",
                toml_project_header, resolved_graalvm, project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let main_content = format!(
                "{}import com.formdev.flatlaf.FlatDarkLaf;\nimport javax.swing.*;\nimport java.awt.*;\n\npublic class Main {{\n    public static void main(String[] args) {{\n        FlatDarkLaf.setup();\n        SwingUtilities.invokeLater(() -> {{\n            JFrame frame = new JFrame(\"Jolt - Swing App\");\n            frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);\n            frame.setSize(450, 250);\n            frame.setLocationRelativeTo(null);\n\n            JLabel label = new JLabel(\"☕ ¡Hola desde Java Swing Moderno con Jolt!\", SwingConstants.CENTER);\n            label.setFont(new Font(\"Segoe UI\", Font.BOLD, 14));\n\n            JButton button = new JButton(\"Presióname\");\n            button.addActionListener(e -> JOptionPane.showMessageDialog(frame, \"🚀 ¡Swing ultrarrápido con Jolt!\"));\n\n            JPanel panel = new JPanel(new BorderLayout(10, 10));\n            panel.setBorder(BorderFactory.createEmptyBorder(20, 20, 20, 20));\n            panel.add(label, BorderLayout.CENTER);\n            panel.add(button, BorderLayout.SOUTH);\n\n            frame.add(panel);\n            frame.setVisible(true);\n        }});\n    }}\n}}\n",
                pkg_stmt
            );
            fs::write(src_main_java.join("Main.java"), main_content)?;
            fs::write(base_dir.join("src/main/resources/app.properties"), include_str!("../templates/swing/src/main/resources/app.properties"))?;

            let test_content = format!(
                "{}import org.junit.jupiter.api.Test;\nimport static org.junit.jupiter.api.Assertions.assertNotNull;\n\npublic class SwingAppTest {{\n    @Test\n    void testAppLoads() {{\n        assertNotNull(\"Swing app test\");\n    }}\n}}\n",
                pkg_stmt
            );
            fs::write(src_test_java.join("SwingAppTest.java"), test_content)?;
        }
        "web" => {
            let toml_content = format!(
                "{}\n\n[dependencies]\n\"io.javalin:javalin\" = \"6.1.3\"\n\"org.slf4j:slf4j-simple\" = \"2.0.12\"\n\n[dev-dependencies]\n\"org.junit.jupiter:junit-jupiter-api\" = \"5.10.2\"\n\n[graalvm]\nenabled = {}\nname = \"{}-server\"\nargs = [\n    \"--no-fallback\",\n    \"--enable-http\",\n    \"-H:+ReportExceptionStackTraces\"\n]\n",
                toml_project_header, resolved_graalvm, project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let main_content = format!(
                "{}import io.javalin.Javalin;\nimport java.util.Map;\n\npublic class Main {{\n    public static void main(String[] args) {{\n        var app = Javalin.create(config -> {{\n            config.showJavalinBanner = false;\n        }}).start(7070);\n\n        app.get(\"/\", ctx -> ctx.result(\"⚡ API REST con Javalin y Jolt!\"));\n        app.get(\"/api/saludo\", ctx -> ctx.json(Map.of(\"mensaje\", \"Hola desde Jolt\", \"estado\", \"OK\")));\n\n        System.out.println(\"🚀 Servidor Javalin iniciado en http://localhost:7070\");\n    }}\n}}\n",
                pkg_stmt
            );
            fs::write(src_main_java.join("Main.java"), main_content)?;
            fs::write(base_dir.join("src/main/resources/application.properties"), include_str!("../templates/web/src/main/resources/application.properties"))?;

            let test_content = format!(
                "{}import org.junit.jupiter.api.Test;\nimport static org.junit.jupiter.api.Assertions.assertTrue;\n\npublic class MainTest {{\n    @Test\n    void testServerLoads() {{\n        assertTrue(true, \"Javalin web app test\");\n    }}\n}}\n",
                pkg_stmt
            );
            fs::write(src_test_java.join("MainTest.java"), test_content)?;
        }
        "spring" | "spring-boot" => {
            let toml_content = format!(
                "{}\n\n[dependencies]\n\"org.springframework.boot:spring-boot-starter-web\" = \"3.2.3\"\n\"org.springframework.boot:spring-boot-starter-actuator\" = \"3.2.3\"\n\n[dev-dependencies]\n\"org.springframework.boot:spring-boot-starter-test\" = \"3.2.3\"\n\"org.junit.jupiter:junit-jupiter-api\" = \"5.10.2\"\n\n[graalvm]\nenabled = {}\nname = \"{}-app\"\nargs = [\n    \"--no-fallback\",\n    \"-H:+ReportExceptionStackTraces\"\n]\n",
                toml_project_header, resolved_graalvm, project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let main_content = format!(
                "{}import org.springframework.boot.SpringApplication;\nimport org.springframework.boot.autoconfigure.SpringBootApplication;\nimport org.springframework.web.bind.annotation.GetMapping;\nimport org.springframework.web.bind.annotation.RestController;\nimport java.util.Map;\n\n@SpringBootApplication\n@RestController\npublic class Main {{\n\n    public static void main(String[] args) {{\n        SpringApplication.run(Main.class, args);\n    }}\n\n    @GetMapping(\"/\")\n    public Map<String, String> home() {{\n        return Map.of(\n            \"status\", \"success\",\n            \"message\", \"🚀 Spring Boot 3 ejecutándose ultrarrápido con Jolt!\",\n            \"runtime\", \"Java 21 LTS\"\n        );\n    }}\n}}\n",
                pkg_stmt
            );
            fs::write(src_main_java.join("Main.java"), main_content)?;
            fs::write(base_dir.join("src/main/resources/application.properties"), include_str!("../templates/spring/src/main/resources/application.properties"))?;

            let test_content = format!(
                "{}import org.junit.jupiter.api.Test;\nimport static org.junit.jupiter.api.Assertions.assertTrue;\n\npublic class SpringAppTest {{\n    @Test\n    void contextLoads() {{\n        assertTrue(true);\n    }}\n}}\n",
                pkg_stmt
            );
            fs::write(src_test_java.join("SpringAppTest.java"), test_content)?;
        }
        _ => {
            let toml_content = format!(
                "{}\n\n[dependencies]\n\n[dev-dependencies]\n\"org.junit.jupiter:junit-jupiter-api\" = \"5.10.2\"\n\n[graalvm]\nenabled = {}\nname = \"{}-bin\"\nargs = [\n    \"--no-fallback\"\n]\n",
                toml_project_header, resolved_graalvm, project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let main_content = format!(
                "{}public class Main {{\n    public static void main(String[] args) {{\n        System.out.println(\"Hola desde Jolt!\");\n    }}\n}}\n",
                pkg_stmt
            );
            fs::write(src_main_java.join("Main.java"), main_content)?;

            let test_content = format!(
                "{}import org.junit.jupiter.api.Test;\nimport static org.junit.jupiter.api.Assertions.assertTrue;\n\npublic class AppTest {{\n    @Test\n    void testApp() {{\n        assertTrue(true);\n    }}\n}}\n",
                pkg_stmt
            );
            fs::write(src_test_java.join("AppTest.java"), test_content)?;
        }
    }

    // Auto-registrar en workspace si se creó dentro de uno
    if let Some((root_ws_dir, _)) = crate::manifest::JoltManifest::find_root_workspace(&current_dir) {
        let abs_base = if base_dir.is_relative() {
            current_dir.join(&base_dir)
        } else {
            base_dir.clone()
        };
        if let Ok(canon_base) = abs_base.canonicalize() {
            if let Ok(rel) = canon_base.strip_prefix(&root_ws_dir) {
                let member_str = rel.to_string_lossy().to_string();
                let ws_manifest = root_ws_dir.join("jolt.toml");
                let _ = crate::manifest::JoltManifest::add_member_to_workspace(&ws_manifest, &member_str);
                println!("  [OK] Módulo '{}' registrado en workspace '{}'", member_str, root_ws_dir.display());
            }
        }
    }

    println!("[OK] Proyecto '{}' inicializado correctamente (Plantilla: '{}').", project_name, tmpl);
    if let Some(ref pkg) = resolved_pkg {
        println!("     Paquete Java configurado: {}", pkg);
    }
    if resolved_graalvm {
        println!("     ⚡ Configuración GraalVM Native Image activada por defecto.");
    }
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

    #[test]
    fn test_init_project_with_package() {
        let temp_dir = std::env::temp_dir().join("jolt_pkg_scaffold_test");
        let _ = fs::remove_dir_all(&temp_dir);

        let target_dir = temp_dir.join("my_app");
        init_project(
            Some(target_dir.to_str().unwrap()),
            Some("minimal"),
            Some("org.equipo.demo"),
            Some("org.equipo"),
            false,
            false,
        ).unwrap();

        assert!(target_dir.join("jolt.toml").exists());
        let java_file = target_dir.join("src/main/java/org/equipo/demo/Main.java");
        assert!(java_file.exists());

        let content = fs::read_to_string(&java_file).unwrap();
        assert!(content.contains("package org.equipo.demo;"));
        assert!(content.contains("public class Main"));

        let manifest = crate::manifest::JoltManifest::load_from_file(&target_dir.join("jolt.toml")).unwrap();
        let proj = manifest.project.as_ref().unwrap();
        assert_eq!(proj.package, Some("org.equipo.demo".to_string()));
        assert_eq!(proj.group_id, Some("org.equipo".to_string()));
        assert_eq!(proj.main_class, Some("org.equipo.demo.Main".to_string()));

        let gvm = manifest.graalvm_config().expect("Expected graalvm config in template");
        assert_eq!(gvm.enabled, Some(false));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_init_project_with_graalvm() {
        let temp_dir = std::env::temp_dir().join("jolt_gvm_scaffold_test");
        let _ = fs::remove_dir_all(&temp_dir);

        let target_dir = temp_dir.join("cli_app");
        init_project(
            Some(target_dir.to_str().unwrap()),
            Some("cli"),
            None,
            None,
            false,
            true,
        ).unwrap();

        assert!(target_dir.join("jolt.toml").exists());
        let manifest = crate::manifest::JoltManifest::load_from_file(&target_dir.join("jolt.toml")).unwrap();
        let gvm = manifest.graalvm_config().expect("Expected graalvm config in cli template");
        assert_eq!(gvm.enabled, Some(true));
        assert_eq!(gvm.name, Some("cli_app-cli".to_string()));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_init_project_web_template_has_test() {
        let temp_dir = std::env::temp_dir().join("jolt_web_scaffold_test");
        let _ = fs::remove_dir_all(&temp_dir);

        let target_dir = temp_dir.join("web_app");
        init_project(
            Some(target_dir.to_str().unwrap()),
            Some("web"),
            None,
            None,
            false,
            false,
        ).unwrap();

        assert!(target_dir.join("jolt.toml").exists());
        assert!(target_dir.join("src/main/java/Main.java").exists());
        assert!(target_dir.join("src/test/java/MainTest.java").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_init_project_workspace() {
        let temp_dir = std::env::temp_dir().join("jolt_ws_scaffold_test");
        let _ = fs::remove_dir_all(&temp_dir);

        let ws_dir = temp_dir.join("monorepo");
        init_project(
            Some(ws_dir.to_str().unwrap()),
            None,
            None,
            None,
            true,
            false,
        ).unwrap();

        assert!(ws_dir.join("jolt.toml").exists());
        let manifest = crate::manifest::JoltManifest::load_from_file(&ws_dir.join("jolt.toml")).unwrap();
        assert!(manifest.is_workspace());
        assert_eq!(manifest.workspace.unwrap().members.len(), 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}


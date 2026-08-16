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
            let toml_content = format!(
                r#"[project]
name = "{}"
version = "0.1.0"
java_version = "21"

[dependencies]
"info.picocli:picocli" = "4.7.6"

[dev-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.10.2"
"#,
                project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let java_content = r#"import picocli.CommandLine;
import picocli.CommandLine.Command;
import picocli.CommandLine.Option;
import picocli.CommandLine.Parameters;

@Command(name = "cli-app", mixinStandardHelpOptions = true, version = "1.0",
        description = "Aplicacion CLI construida con Picocli y Jolt.")
public class Main implements Runnable {

    @Option(names = {"-u", "--user"}, description = "Nombre del usuario")
    private String user = "Desarrollador";

    @Parameters(paramLabel = "<mensaje>", defaultValue = "Bienvenido a Jolt!", description = "Mensaje a mostrar")
    private String message;

    @Override
    public void run() {
        System.out.println("Hola, " + user + "! -> " + message);
    }

    public static void main(String[] args) {
        int exitCode = new CommandLine(new Main()).execute(args);
        System.exit(exitCode);
    }
}
"#;
            fs::write(base_dir.join("src/main/java/Main.java"), java_content)?;

            let test_content = r#"import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class MainTest {
    @Test
    void testMainApp() {
        assertEquals(2, 1 + 1);
    }
}
"#;
            fs::write(base_dir.join("src/test/java/MainTest.java"), test_content)?;
        }
        "javafx" => {
            let toml_content = format!(
                r#"[project]
name = "{}"
version = "0.1.0"
java_version = "21"

[dependencies]
"org.openjfx:javafx-base" = "21.0.2:linux"
"org.openjfx:javafx-graphics" = "21.0.2:linux"
"org.openjfx:javafx-controls" = "21.0.2:linux"
"org.openjfx:javafx-fxml" = "21.0.2:linux"

[dev-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.10.2"
"#,
                project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let app_content = r#"import javafx.application.Application;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;

public class App extends Application {
    private int clicks = 0;

    @Override
    public void start(Stage stage) {
        Label title = new Label("Jolt + JavaFX");
        title.setStyle("-fx-font-size: 20px; -fx-font-weight: bold; -fx-text-fill: #333;");

        Label counter = new Label("Clics: 0");
        Button btn = new Button("Presioname");
        btn.setOnAction(e -> {
            clicks++;
            counter.setText("Clics: " + clicks);
        });

        VBox root = new VBox(15, title, btn, counter);
        root.setAlignment(Pos.CENTER);

        Scene scene = new Scene(root, 400, 250);
        stage.setTitle("Jolt JavaFX App");
        stage.setScene(scene);
        stage.show();
    }

    public static void main(String[] args) {
        launch(args);
    }
}
"#;
            fs::write(base_dir.join("src/main/java/App.java"), app_content)?;

            let main_content = r#"public class Main {
    public static void main(String[] args) {
        App.main(args);
    }
}
"#;
            fs::write(base_dir.join("src/main/java/Main.java"), main_content)?;
            fs::write(
                base_dir.join("src/main/resources/style.css"),
                ".root { -fx-font-family: 'sans-serif'; -fx-background-color: #f8fafc; }\n",
            )?;
        }
        "swing" => {
            let toml_content = format!(
                r#"[project]
name = "{}"
version = "0.1.0"
java_version = "21"

[dependencies]
"com.formdev:flatlaf" = "3.4.1"

[dev-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.10.2"
"#,
                project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let java_content = r#"import com.formdev.flatlaf.FlatDarkLaf;
import javax.swing.*;
import java.awt.*;

public class Main {
    private static int counter = 0;

    public static void main(String[] args) {
        // Look and Feel moderno oscuro de FlatLaf
        FlatDarkLaf.setup();

        SwingUtilities.invokeLater(() -> {
            JFrame frame = new JFrame("Jolt + Java Swing (FlatLaf)");
            frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
            frame.setSize(450, 280);
            frame.setLocationRelativeTo(null);

            JPanel panel = new JPanel();
            panel.setLayout(new BoxLayout(panel, BoxLayout.Y_AXIS));
            panel.setBorder(BorderFactory.createEmptyBorder(30, 30, 30, 30));

            JLabel title = new JLabel("Aplicacion Java Swing");
            title.setFont(new Font("SansSerif", Font.BOLD, 22));
            title.setAlignmentX(Component.CENTER_ALIGNMENT);

            JLabel counterLabel = new JLabel("Clics realizados: 0");
            counterLabel.setFont(new Font("SansSerif", Font.PLAIN, 16));
            counterLabel.setAlignmentX(Component.CENTER_ALIGNMENT);

            JButton button = new JButton("Incrementar Contador");
            button.setFont(new Font("SansSerif", Font.PLAIN, 14));
            button.setAlignmentX(Component.CENTER_ALIGNMENT);
            button.addActionListener(e -> {
                counter++;
                counterLabel.setText("Clics realizados: " + counter);
            });

            panel.add(title);
            panel.add(Box.createRigidArea(new Dimension(0, 20)));
            panel.add(button);
            panel.add(Box.createRigidArea(new Dimension(0, 15)));
            panel.add(counterLabel);

            frame.add(panel);
            frame.setVisible(true);
        });
    }
}
"#;
            fs::write(base_dir.join("src/main/java/Main.java"), java_content)?;
            fs::write(base_dir.join("src/main/resources/app.properties"), "theme=dark\n")?;

            let test_content = r#"import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class SwingAppTest {
    @Test
    void testContext() {
        assertTrue(true);
    }
}
"#;
            fs::write(base_dir.join("src/test/java/SwingAppTest.java"), test_content)?;
        }
        "web" => {
            let toml_content = format!(
                r#"[project]
name = "{}"
version = "0.1.0"
java_version = "21"

[dependencies]
"io.javalin:javalin" = "6.1.3"
"org.slf4j:slf4j-simple" = "2.0.12"

[dev-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.10.2"
"#,
                project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let java_content = r#"import io.javalin.Javalin;

public class Main {
    public static void main(String[] args) {
        var app = Javalin.create(/*config*/)
            .get("/", ctx -> ctx.result("Servidor Web en ejecucion con Javalin y Jolt!"))
            .get("/health", ctx -> ctx.json("{\"status\":\"ok\",\"tool\":\"jolt\"}"))
            .start(7070);

        System.out.println("Servidor web iniciado en http://localhost:7070");
    }
}
"#;
            fs::write(base_dir.join("src/main/java/Main.java"), java_content)?;
            fs::write(
                base_dir.join("src/main/resources/application.properties"),
                "server.port=7070\napp.env=development\n",
            )?;
        }
        "spring" | "spring-boot" => {
            let toml_content = format!(
                r#"[project]
name = "{}"
version = "0.1.0"
java_version = "21"

[dependencies]
"org.springframework.boot:spring-boot-starter-web" = "3.2.3"

[dev-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.10.2"
"#,
                project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let java_content = r#"import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

@SpringBootApplication
@RestController
public class Main {

    @GetMapping("/")
    public String index() {
        return "Hola desde Spring Boot con Jolt!";
    }

    @GetMapping("/api/status")
    public String status() {
        return "{\"status\":\"UP\",\"framework\":\"Spring Boot 3\",\"manager\":\"Jolt\"}";
    }

    public static void main(String[] args) {
        SpringApplication.run(Main.class, args);
    }
}
"#;
            fs::write(base_dir.join("src/main/java/Main.java"), java_content)?;
            fs::write(
                base_dir.join("src/main/resources/application.properties"),
                "server.port=8080\nspring.application.name=jolt-spring-app\n",
            )?;

            let test_content = r#"import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class SpringAppTest {
    @Test
    void testContext() {
        assertTrue(true);
    }
}
"#;
            fs::write(base_dir.join("src/test/java/SpringAppTest.java"), test_content)?;
        }
        _ => {
            // Plantilla minimal por defecto
            let toml_content = format!(
                r#"[project]
name = "{}"
version = "0.1.0"
java_version = "21"

[dependencies]

[dev-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.10.2"
"#,
                project_name
            );
            fs::write(base_dir.join("jolt.toml"), toml_content)?;

            let java_content = r#"public class Main {
    public static void main(String[] args) {
        System.out.println("Hola desde Jolt!");
    }
}
"#;
            fs::write(base_dir.join("src/main/java/Main.java"), java_content)?;

            let test_content = r#"import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class AppTest {
    @Test
    void testBasic() {
        assertTrue(true);
    }
}
"#;
            fs::write(base_dir.join("src/test/java/AppTest.java"), test_content)?;
        }
    }

    println!("[OK] Proyecto '{}' inicializado correctamente (Plantilla: '{}').", project_name, tmpl);
    println!("     Sugerencia: Ejecuta 'cd {} && jolt install' para sincronizar librerias.", project_name);

    Ok(())
}

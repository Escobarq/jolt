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

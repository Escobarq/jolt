## Proyecto Jolt ⚡️: Plan de Desarrollo

El desarrollo de Jolt se dividirá en **4 Fases principales** (diseñadas para implementarse a lo largo del año). Este documento detalla la estructura global y profundiza en las especificaciones (specs) técnicas exclusivas de la **Fase 1**, la cual ejecutaremos a continuación.

---

## Estructura de Fases

1. **Fase 1: Core CLI y Cliente de Maven Central (Actual)**
   - Configuración de la interfaz de línea de comandos (CLI).
   - Manipulación de archivos `jolt.toml`.
   - Consulta y descarga básica desde repositorios de Maven Central.
2. **Fase 2: Motor de Resolución Complejo y Lockfiles**
   - Resolución de árboles de dependencias transitivas (analizando archivos `pom.xml` remotos).
   - Generación de un archivo `jolt.lock` estricto y determinista.
3. **Fase 3: Gestión del Entorno (JDKs) y Compilación**
   - Descarga automatizada de la Java Development Kit (Temurin/Corretto).
   - Invocación nativa a `javac` para compilar código desde Rust.
4. **Fase 4: Ejecución, Testing y Pulido Final**
   - Comando `jolt run` para ejecutar el programa configurando el *classpath* automáticamente.
   - Integración nativa con frameworks de pruebas (como JUnit).

---

## Especificaciones (Specs) para la Fase 1

### Objetivo
Construir los cimientos del CLI interactivo de Jolt, permitiendo inicializar un proyecto y agregar dependencias reales consultando la API de Maven Central, modificando el `jolt.toml` en tiempo real.

### Open Questions (Para ti)
- **Caché Global**: ¿Deberíamos guardar los `.jar` descargados en `~/.jolt/cache` (comportamiento por defecto de uv/bun) o preferirías que se guarden dentro del proyecto en una carpeta `.jolt-modules` (similar a `node_modules`)?
- **Mensajes del CLI**: ¿Prefieres que los mensajes de la terminal (ej. "Descargando dependencia...") estén en inglés (estándar de la industria) o en español?

### Cambios Propuestos

---

### Módulo CLI (Interfaz de Línea de Comandos)
Implementaremos `clap` (crate de Rust estándar) para parsear los argumentos.
#### [MODIFY] src/main.rs
```rust
// Ejemplo del nuevo código con clap:
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jolt", version = "0.1.0", about = "Gestor Java super rápido")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicializa un nuevo proyecto
    Init { name: Option<String> },
    /// Agrega una dependencia al jolt.toml
    Add { dependency: String },
}
```

---

### Módulo del Sistema de Archivos
#### [MODIFY] src/manifest.rs
- Extenderemos el parser actual para permitir **escribir** de vuelta al archivo `.toml` sin borrar su formato (añadir dependencias programáticamente cuando se use `jolt add`).

---

### Módulo de Red (Maven Client)
#### [NEW] src/maven.rs
- Usaremos `reqwest` y `tokio` (para asincronía).
- Integraremos la búsqueda mediante la API de Maven Search: `https://search.maven.org/solrsearch/select?q=g:{groupId}+AND+a:{artifactId}&wt=json`
- Funcionalidad para descargar el archivo `.jar` resultante en la caché y verificar su integridad.

---

## Plan de Verificación

### Pruebas Automatizadas (Automated Tests)
- `cargo test`: Verificaremos que el parser de `clap` funciona correctamente enviando argumentos falsos (`jolt add gson`).
- Tests unitarios en `src/maven.rs` para mockear la respuesta de la API de Maven y validar que extrae la última versión correctamente.

### Pruebas Manuales (Manual Verification)
1. Ejecutaremos `cargo run -- init demo`.
2. Verificaremos que se crea un directorio `demo` con su respectivo `jolt.toml`.
3. Ejecutaremos `cargo run -- add com.google.code.gson:gson`.
4. Tú y yo validaremos que el CLI consulte Maven Central y escriba la versión actualizada en el archivo `jolt.toml`.

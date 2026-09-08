# Jolt ⚡️: Gestor de Proyectos Java de Nueva Generación

Este documento explora el diseño arquitectónico y funcional de un nuevo gestor de paquetes y proyectos para Java, inspirado en el rendimiento y la experiencia de desarrollo (DX) de herramientas modernas como **uv** (Python) y **bun** (JavaScript). 

El objetivo principal es simplificar radicalmente la creación de aplicaciones Java, reduciendo la verbosidad y multiplicando la velocidad operativa, manteniendo total compatibilidad con el ecosistema de Maven Central.

---

## 1. Visión y Motivación

Históricamente, los desarrolladores de Java han dependido de Maven y Gradle. Aunque son robustos, sufren de:
- **Tiempos de inicio lentos** (la JVM debe arrancar para ejecutar el gestor).
- **Curva de aprendizaje empinada** y configuración verbosa (XML complejo en Maven, Groovy/Kotlin intrincado en Gradle).
- **Resolución de dependencias pesada**.

**Jolt** es un binario único, ultrarrápido y escrito en **Rust**, que se ejecuta instantáneamente y centraliza todo el ciclo de vida del desarrollo en Java: inicialización de proyectos, gestión de toolchains de JDK, resolución determinista de dependencias, compilación incremental, pruebas unitarias, ejecución con Hot Reload y empaquetado nativo/instaladores.

---

## 2. Características Clave

### 🚀 Velocidad Extrema (Escrito en Rust)
Al igual que `uv` y `bun`, el CLI está construido en Rust. Esto permite:
- **Inicio en milisegundos** (cold start instantáneo).
- Resolución de dependencias y descargas concurrentes asíncronas con `tokio` y `reqwest`.
- Uso de **Hardlinks y Caché Global** (`~/.jolt/cache/`), evitando descargar o duplicar la misma dependencia JAR/POM en múltiples proyectos.

### ☕ Gestión de Entorno y Detección Inteligente de JDKs
- **Detección Multinivel:** Escanea variables de entorno (`GRAALVM_HOME`, `JAVA_HOME`), ejecutables en `PATH` (`javac`, `java`) y directorios estándar del sistema operativo (Oracle GraalVM, Eclipse Temurin, Amazon Corretto, Azul Zulu, BellSoft Liberica, Microsoft OpenJDK, etc.).
- **Compatibilidad Semántica:** Si el proyecto define `java_version = "21"` y el sistema cuenta con un JDK compatible superior (ej. Java 25), Jolt lo aprovecha automáticamente sin descargas innecesarias.
- **Sin Descargas Silenciosas:** Notifica claramente y requiere consentimiento o uso del flag `--download-jdk` para aprovisionar Eclipse Temurin.

### 📄 Manifiesto Simplificado (`jolt.toml`) y Lockfile Determinista (`jolt.lock`)
Adiós al verboso `pom.xml`. Jolt utiliza un formato moderno y legible, inspirado en `Cargo.toml`.

```toml
[project]
name = "mi-app"
version = "1.0.0"
java_version = "21"
package = "com.empresa.app"

[dependencies]
"com.google.code.gson:gson" = "2.14.0"

[dev-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.10.2"

[graalvm]
enabled = true
args = ["--no-fallback", "-H:+ReportExceptionStackTraces"]
```

---

## 3. Arquitectura Interna del Proyecto (`src/`)

La base de código de Jolt está organizada de forma modular en 8 subsistemas especializados dentro de [`src/`](../src/):

| Módulo | Directorio | Responsabilidad Principal |
|---|---|---|
| **Core** | [`src/core/`](../src/core/) | Parser de `jolt.toml` ([`manifest.rs`](../src/core/manifest.rs)), gestión de `jolt.lock` ([`lockfile.rs`](../src/core/lockfile.rs)) y caché global direccionable ([`cache.rs`](../src/core/cache.rs)). |
| **CLI** | [`src/cli/`](../src/cli/) | Definición de argumentos, subcomandos y flags con `clap` ([`args.rs`](../src/cli/args.rs), [`mod.rs`](../src/cli/mod.rs)). |
| **Build & Run** | [`src/build/`](../src/build/) | Compilación con `javac` ([`compiler.rs`](../src/build/compiler.rs)), ejecución y Hot Reload ([`runner.rs`](../src/build/runner.rs)) y runner JUnit 5 ([`test_runner.rs`](../src/build/test_runner.rs)). |
| **Resolver** | [`src/resolver/`](../src/resolver/) | Cliente asíncrono Maven Central ([`maven_client.rs`](../src/resolver/maven_client.rs)) y parser XML de POMs ([`pom_parser.rs`](../src/resolver/pom_parser.rs)). |
| **Packaging** | [`src/packaging/`](../src/packaging/) | Fat-JAR ([`jar.rs`](../src/packaging/jar.rs)), GraalVM AOT ([`native_image.rs`](../src/packaging/native_image.rs)), jpackage ([`jpackage.rs`](../src/packaging/jpackage.rs)), NSIS ([`nsis.rs`](../src/packaging/nsis.rs)) y UPX ([`upx.rs`](../src/packaging/upx.rs)). |
| **Toolchain** | [`src/toolchain/`](../src/toolchain/) | Detección multinivel del sistema ([`detector.rs`](../src/toolchain/detector.rs)) y descarga automatizada de JDKs ([`downloader.rs`](../src/toolchain/downloader.rs)). |
| **Scaffold** | [`src/scaffold/`](../src/scaffold/) | Generador de plantillas y workspaces ([`templates.rs`](../src/scaffold/templates.rs)) y configuración para VS Code / IntelliJ ([`ide_config.rs`](../src/scaffold/ide_config.rs)). |
| **Doctor** | [`src/doctor/`](../src/doctor/) | Diagnóstico y validación de herramientas instaladas y proyectos ([`diagnostics.rs`](../src/doctor/diagnostics.rs)). |

---

## 4. Flujo de Trabajo (UX / CLI)

- `jolt init` - Asistente interactivo para inicializar proyectos individuales o workspaces monorepo.
- `jolt add <groupId:artifactId>` - Busca en Maven Central y registra la versión en `jolt.toml`.
- `jolt install` - Resuelve dependencias y genera el `jolt.lock` determinista enlazando JARs a `.jolt/modules/`.
- `jolt run [--watch]` - Compila y ejecuta con soporte de Hot Reload instantáneo.
- `jolt test` - Ejecuta la suite de pruebas JUnit 5 de forma nativa.
- `jolt build [--standalone | --native | --installer]` - Empaqueta Fat-JARs, binarios GraalVM AOT o instaladores NSIS.
- `jolt check` - Diagnóstica la salud del entorno, toolchains y dependencias.

---

## 5. Roadmap

- **v0.7.0 / v0.7.1 (Completadas):**
  - Detección inteligente de JDKs locales y compatibilidad semántica.
  - GraalVM Native Image y presets en todas las plantillas.
  - Empaquetado en Windows con NSIS y UPX.
  - Workspaces monorepo y dependencias locales por ruta.
  - Reorganización modular del código en `src/`.
- **v0.8.0 (Próxima Versión):**
  - `jolt fmt`: Formateador nativo ultrarrápido de código Java.
  - `jolt doc`: Generador y servidor web embebido para Javadoc.
  - `jolt audit`: Auditoría de vulnerabilidades CVE (OSV.dev / Sonatype).

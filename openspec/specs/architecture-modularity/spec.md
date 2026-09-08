# architecture-modularity Specification

## Purpose
Estructura y organiza la base de código de Jolt (`src/`) en módulos especializados y altamente cohesivos para mejorar la mantenibilidad, legibilidad y escalabilidad del proyecto hacia la versión 0.8.0 y futuras extensiones (`jolt fmt`, `jolt doc`, `jolt audit`).

## Requirements

### Requirement: Organización modular de la base de código en subdirectorios
El sistema SHALL estructurar la lógica interna del binario `jolt` en 8 módulos principales organizados por dominio funcional dentro de `src/`:

1. `src/core/`: Estructuras de datos fundamentales, parser del manifiesto (`manifest.rs`), gestión determinista de `jolt.lock` (`lockfile.rs`) y almacenamiento en caché global (`cache.rs`).
2. `src/cli/`: Definición de comandos, opciones y flags con `clap` (`args.rs`, `mod.rs`).
3. `src/build/`: Motores de compilación Java (`compiler.rs`), ejecución y modo watch (`runner.rs`) y ejecución de pruebas unitarias (`test_runner.rs`).
4. `src/resolver/`: Cliente HTTP asíncrono para Maven Central (`maven_client.rs`) y analizador XML de POMs (`pom_parser.rs`).
5. `src/packaging/`: Motores de empaquetado Fat-JAR (`jar.rs`), GraalVM Native Image (`native_image.rs`), jpackage (`jpackage.rs`), instaladores NSIS para Windows (`nsis.rs`) y compresor de binarios UPX (`upx.rs`).
6. `src/toolchain/`: Detección multinivel de JDKs locales y semántica de versiones (`detector.rs`) y descargador de JDKs Adoptium/Temurin (`downloader.rs`).
7. `src/scaffold/`: Generador de plantillas de proyectos y workspaces (`templates.rs`) y sincronizador de configuraciones IDE (`ide_config.rs`).
8. `src/doctor/`: Diagnóstico integral del entorno del sistema y proyectos (`diagnostics.rs`).

#### Scenario: Acceso coherente a APIs entre módulos
- **WHEN** un módulo requiere interactuar con el manifiesto o el toolchain
- **THEN** importa directamente los tipos canónicos expuestos a través de los `mod.rs` correspondientes (ej. `crate::core::JoltManifest`, `crate::toolchain::ToolchainManager`)

### Requirement: Punto de entrada unificado y despacho de comandos
El archivo `src/main.rs` SHALL actuar como el orquestador principal, inicializando el CLI y despachando cada subcomando hacia los módulos especializados de forma limpia y desacoplada.

#### Scenario: Enrutamiento de subcomandos CLI
- **WHEN** el usuario ejecuta cualquier subcomando de Jolt (`init`, `add`, `build`, `run`, `test`, `check`, etc.)
- **THEN** `src/main.rs` despacha la ejecución al manejador correspondiente dentro de `crate::build`, `crate::packaging`, `crate::scaffold`, `crate::resolver` o `crate::doctor`.

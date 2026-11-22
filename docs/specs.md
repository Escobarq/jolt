# Jolt - Especificaciones Tecnicas (Specs) para Desarrollo Paralelo

Este documento contiene las especificaciones formales (Specs) del proyecto Jolt. El diseno modular permite que multiples ingenieros (o agentes de IA) trabajen de forma concurrente sin pisarse los unos a los otros. 

Cada modulo especificado a continuacion tiene entradas (inputs), salidas (outputs) y responsabilidades claramente delimitadas.

---

## Modulo A: Interfaz de Linea de Comandos (CLI) y scaffolding `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado, testeado y archivado en [`docs/archive/modulo-a-cli-scaffolding.md`](archive/modulo-a-cli-scaffolding.md).

**Responsabilidad:** Manejar la interaccion con el usuario, parsear comandos y gestionar el archivo `jolt.toml`.
**Dependencias:** `clap`, `toml`, `serde`, `toml_edit`.

### Tareas implementadas (Spec-A):
1. [x] **Configuracion de `clap`:** Entrypoint (`main.rs` / `cli.rs`) con subcomandos: `init`, `add`, `install`, `build`, `run`.
2. [x] **Comando `jolt init [nombre]`:** Genera `src/main/java/Main.java` y `jolt.toml`.
3. [x] **Mutacion del archivo TOML:** `JoltManifest::add_dependency_to_file` con `toml_edit`.

**Criterios de Aceptacion:** Cumplidos y verificados con tests.

---

## Modulo B: Cliente de Maven Central y Resolver de Dependencias `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado, testeado y archivado en [`docs/archive/modulo-b-maven-resolver.md`](archive/modulo-b-maven-resolver.md).

**Responsabilidad:** Comunicarse con Internet para resolver versiones de librerias y descargar metadatos (archivos `pom.xml`).
**Dependencias:** `reqwest`, `tokio`, `serde_json`, `quick-xml`.

### Tareas implementadas (Spec-B):
1. [x] **Buscador (Search API):** `MavenClient::fetch_latest_version`.
2. [x] **Descarga de Metadatos (POM):** `MavenClient::fetch_pom`.
3. [x] **Parseo de POM:** `MavenClient::parse_pom_dependencies` con XML ultrarrapido.
4. [x] **Arbol de Dependencias:** `MavenClient::fetch_dependency_tree`.

**Criterios de Aceptacion:** Cumplidos y verificados con tests y llamadas a Maven Central en vivo.

---

## Modulo C: Gestor de Cache Global (Storage Engine) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado, testeado y archivado en [`docs/archive/modulo-c-cache-storage.md`](archive/modulo-c-cache-storage.md).

**Responsabilidad:** Almacenar de manera persistente los archivos `.jar` descargados y enlazarlos (hardlinks) a los proyectos individuales para ahorrar espacio y tiempo.
**Dependencias:** `dirs`, `sha2`, `hex`, `std::fs`.

### Tareas implementadas (Spec-C):
1. [x] **Directorio Global:** `~/.jolt/cache/v1/jars/`.
2. [x] **Descarga y Hash SHA-256:** `CacheManager::save_jar`.
3. [x] **Enlace Duro (Hardlink):** `CacheManager::link_to_project` hacia `.jolt/modules/`.
4. [x] **Sincronizacion:** Subcomando `jolt install`.

**Criterios de Aceptacion:** Cumplidos y verificados con tests y uso de inodos compartidos en disco.

---

## Modulo D: Aprovisionador de Toolchains (Gestion de JDK) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado, testeado y archivado en [`docs/archive/modulo-d-toolchain-provisioner.md`](archive/modulo-d-toolchain-provisioner.md).

**Responsabilidad:** Aislar al desarrollador de la instalacion manual de Java. Descargar y desempaquetar la JDK que el proyecto pida.
**Dependencias:** `reqwest`, `tar`, `flate2`, `dirs`.

### Tareas implementadas (Spec-D):
1. [x] **Deteccion de JDK:** `ToolchainManager::find_system_jdk` y `ToolchainManager::find_cached_jdk`.
2. [x] **API Adoptium / Temurin:** `ToolchainManager::download_and_extract_jdk` con autodeteccion de SO y arquitectura.
3. [x] **Extraccion de `.tar.gz`:** Descompresion directa a la cache global `~/.jolt/jdks/`.
4. [x] **Integracion:** Compilacion y ejecucion automatica segun `java_version` en `jolt.toml`.

**Criterios de Aceptacion:** Cumplidos y verificados con tests y ejecucion real de Java 21.

---

## Modulo E: Motor de Compilacion y Ejecucion (Build & Run Engine) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado, testeado y archivado en [`docs/archive/modulo-e-build-run-engine.md`](archive/modulo-e-build-run-engine.md).

**Responsabilidad:** Coordinar el classpath, llamar a la JDK, compilar el codigo al vuelo y ejecutarlo.
**Dependencias:** `std::process::Command`, `std::fs`.

### Tareas implementadas (Spec-E):
1. [x] **Constructor del Classpath:** `BuildEngine::build_classpath` con soporte multi-plataforma.
2. [x] **Compilacion al Vuelo:** `BuildEngine::compile` (`javac`).
3. [x] **Ejecucion Interactiva:** `BuildEngine::run` (`java`).
4. [x] **Empaquetado JAR:** `BuildEngine::build_jar` (`jar cfe`).

**Criterios de Aceptacion:** Cumplidos y verificados con ejecucion de aplicaciones usando Gson de Maven Central.

---

## Modulos F a U (Implementados y Archivados)

Los modulos siguientes ya fueron implementados, testeados y archivados en `docs/archive/`. Se conservan como referencia historica del proyecto:

| Modulo | Nombre | Estado | Especificacion archivada |
|---|---|---|---|
| **F** | Empaquetador Fat-JAR / Uber-JAR | Completado y archivado | [`docs/archive/modulo-f-i-fatjar-resources.md`](archive/modulo-f-i-fatjar-resources.md) |
| **G** | Pruebas unitarias (JUnit 5) | Completado y archivado | [`docs/archive/modulo-g-unit-testing.md`](archive/modulo-g-unit-testing.md) |
| **H** | Modo Watch / Hot Reload | Completado y archivado | [`docs/archive/modulo-h-watch-mode.md`](archive/modulo-h-watch-mode.md) |
| **I** | Gestion de recursos estaticos | Completado y archivado | [`docs/archive/modulo-f-i-fatjar-resources.md`](archive/modulo-f-i-fatjar-resources.md) |
| **J** | Comprobacion de sistema y proyecto | Completado y archivado | [`docs/archive/modulo-j-system-project-check.md`](archive/modulo-j-system-project-check.md) |
| **K** | Lockfile determinista (`jolt.lock`) | Completado y archivado | [`docs/archive/modulo-k-lockfile.md`](archive/modulo-k-lockfile.md) |
| **L** | Plantillas de proyectos | Completado y archivado | [`docs/archive/modulo-l-templates.md`](archive/modulo-l-templates.md) |
| **M** | Eliminacion de dependencias | Completado y archivado | [`docs/archive/modulo-m-remove-dependency.md`](archive/modulo-m-remove-dependency.md) |
| **N** | Busqueda de paquetes en Maven Central | Completado y archivado | [`docs/archive/modulo-n-search.md`](archive/modulo-n-search.md) |
| **O** | Dev-dependencias e init interactivo | Completado y archivado | [`docs/archive/modulo-o-dev-dependencies-interactive-init.md`](archive/modulo-o-dev-dependencies-interactive-init.md) |
| **P** | `jolt sync` y configuracion de IDEs | Completado y archivado | [`docs/archive/modulo-p-ide-sync.md`](archive/modulo-p-ide-sync.md) |
| **Q** | Empaquetado nativo con jpackage | Completado y archivado | [`docs/archive/modulo-q-jpackage-bundler.md`](archive/modulo-q-jpackage-bundler.md) |
| **R** | Paquetes Java y namespaces | Completado y archivado | [`docs/archive/modulo-r-java-packages.md`](archive/modulo-r-java-packages.md) |
| **S** | Asistente interactivo `jolt init` | Completado y archivado | [`docs/archive/modulo-s-interactive-init-workspace.md`](archive/modulo-s-interactive-init-workspace.md) |
| **T** | Workspaces multimodulo | Completado y archivado | [`docs/archive/modulo-t-multi-module-workspaces.md`](archive/modulo-t-multi-module-workspaces.md) |
| **U** | Instalador Windows (NSIS + UPX) | Completado y archivado | [`docs/archive/modulo-u-windows-nsis-upx.md`](archive/modulo-u-windows-nsis-upx.md) |

> Modulos **V, W y X** (Fase 6: deteccion de GraalVM, configuración `[graalvm]` y compilador nativo `native-image`) fueron especificados originalmente en `docs/specs-v6.md` pero **retirados del CLI actual**: el empaquetado nativo disponible usa `jpackage` (`jolt build --package | --installer`). Su historial completo permanece en `openspec/changes/archive/2026-09-03-jdk-detection-and-graalvm/`.

---

## Roadmap v0.8.0 (Especificado, no implementado)

Detalle completo en [`docs/specs-v7.md`](specs-v7.md).

| Modulo | Capabilidad | Comando | Estado |
|---|---|---|---|
| **Y** | Formateador de codigo Java | `jolt fmt` | Especificado |
| **Z** | Generador y servidor local de Javadoc | `jolt doc` | Especificado |
| **AA** | Auditoria de seguridad y CVEs (OSV.dev) | `jolt audit` | Especificado |

### Modulo Y: `jolt fmt` (Formateador de Codigo Java)
Formatea archivos `.java` instantáneamente (indentación, espaciado, ordenamiento de imports) sin herramientas pesadas de Maven/Gradle.
- `jolt fmt` — formatea todo `src/`
- `jolt fmt --check` — valida formato sin modificar (CI/CD)
- `jolt fmt --diff` — muestra diff unificado de cambios necesarios
- Configuración en `jolt.toml` bajo `[format]`: `style`, `max_width`, `sort_imports`, `remove_unused_imports`.

### Modulo Z: `jolt doc` (Javadoc + Servidor Web)
Genera Javadoc HTML y lo sirve localmente para previsualización inmediata.
- `jolt doc` — genera en `target/doc/`
- `jolt doc --open` — genera y abre en el navegador
- `jolt doc --serve` — servidor HTTP embebido en `http://localhost:8080`
- Resuelve `javadoc` via `ToolchainManager` y el classpath via `BuildEngine::build_classpath`.

### Modulo AA: `jolt audit` (Auditoria de CVEs)
Audita `jolt.lock` contra OSV.dev y Sonatype OSS Index.
- `jolt audit` — audita todas las dependencias
- `jolt audit --severity high` — filtra por severidad mínima
- `jolt audit --json` — salida JSON para pipelines CI/CD
- `jolt audit --fix` — sugiere versiones parcheadas en `jolt.toml`

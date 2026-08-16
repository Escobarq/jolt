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

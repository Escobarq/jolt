# Graph Report - jolt  (2026-09-08)

## Corpus Check
- 98 files · ~48,816 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 553 nodes · 818 edges · 79 communities (30 shown, 36 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 15 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `eae79e3a`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- MavenClient
- Toolchain
- package_native_app
- JoltManifest
- CacheManager
- init_project
- org.junit.jupiter.api.Test
- build_standalone_jar
- App
- JoltLock
- Spring Boot Templates
- Picocli Command Templates
- 3. Módulo Y: Formateador Integrado de Código Java (`jolt fmt`)
- Modulo P: Comando `jolt sync` y Autoconfiguracion de VS Code e IDEs Java
- Archivo de Modulo O: Gestion de Dev-Dependencies, Menu Interactivo y Soporte IDE (v0.2.0)
- 1. Modulo Q: Empaquetado Binario Nativo Multiplataforma (`jolt package`)
- Java Main Entrypoint 2
- Java Main Entrypoint 3
- Java Main Entrypoint 4
- Graphify Markdown Rules
- Benchmark Scripts
- Module A Documentation
- Module B Documentation
- Module C Documentation
- Module D Documentation
- Module E Documentation
- Module F Documentation
- Module I Documentation
- Module G Documentation
- Module H Documentation
- Module J Documentation
- Module K Documentation
- Module L Documentation
- Module M Documentation
- Module N Documentation
- Graphify Agent Rules
- Graphify Agent Workflows
- Benchmark Output Results
- Jolt Architecture Vision
- Phase 1 Documentation
- Phase 2 Documentation
- Phase 3 Documentation
- Phase 4 Documentation
- Maven Test Project
- Jolt Configuration
- Jolt Root Repo
- install_in_dir
- Jolt - Especificaciones Tecnicas Fase 5
- Modulo Q: Empaquetado Binario Nativo Multiplataforma (jpackage)
- Modulo R: Paquetes Java y Namespaces Estructurados
- Modulo S: Asistente Interactivo de Inicializacion
- Modulo T: Workspaces Multimodulo y Dependencias Locales
- openspec-explore/SKILL.md
- opsx-explore.md
- Módulo U: Empaquetado en Windows con UPX y NSIS (Windows Installer)
- Requirements
- Requirements
- Decisions
- ADDED Requirements
- ADDED Requirements
- Jolt - Especificaciones Técnicas Fase 6
- proposal.md
- Commands
- tasks.md
- parse_pom_dependencies
- architecture-modularity Specification

## God Nodes (most connected - your core abstractions)
1. `Toolchain` - 21 edges
2. `CacheManager` - 20 edges
3. `JoltManifest` - 20 edges
4. `MavenClient` - 17 edges
5. `package_native_app()` - 15 edges
6. `ToolchainManager` - 14 edges
7. `init_project()` - 13 edges
8. `build_standalone_jar()` - 12 edges
9. `build_native_image()` - 12 edges
10. `download_and_extract_jdk()` - 12 edges

## Surprising Connections (you probably didn't know these)
- `AGENTS Graphify Usage` --semantically_similar_to--> `GEMINI Graphify Usage`  [INFERRED] [semantically similar]
  AGENTS.md → GEMINI.md
- `package_native_app()` --calls--> `build_standalone_jar()`  [INFERRED]
  src/packaging/jpackage.rs → src/packaging/jar.rs
- `test_generate_nsis_script_content()` --calls--> `generate_nsis_script()`  [INFERRED]
  src/packaging/mod.rs → src/packaging/nsis.rs
- `test_ensure_ide_configuration_creates_files()` --calls--> `ensure_ide_configuration()`  [INFERRED]
  src/scaffold/mod.rs → src/scaffold/ide_config.rs
- `init_project()` --calls--> `print_available_templates()`  [INFERRED]
  src/scaffold/mod.rs → src/scaffold/templates.rs

## Import Cycles
- 2-file cycle: `src/toolchain/downloader.rs -> src/toolchain/mod.rs -> src/toolchain/downloader.rs`

## Hyperedges (group relationships)
- **Jolt Modules** — docs_archive_modulo_a_cli_scaffolding_modulo_a, docs_archive_modulo_b_maven_resolver_modulo_b, docs_archive_modulo_c_cache_storage_modulo_c, docs_archive_modulo_d_toolchain_provisioner_modulo_d, docs_archive_modulo_e_build_run_engine_modulo_e, docs_archive_modulo_f_i_fatjar_resources_modulo_f, docs_archive_modulo_f_i_fatjar_resources_modulo_i, docs_archive_modulo_g_unit_testing_modulo_g, docs_archive_modulo_h_watch_mode_modulo_h, docs_archive_modulo_j_system_project_check_modulo_j, docs_archive_modulo_k_lockfile_modulo_k, docs_archive_modulo_l_templates_modulo_l, docs_archive_modulo_m_remove_dependency_modulo_m, docs_archive_modulo_n_search_modulo_n [EXTRACTED 1.00]

## Communities (79 total, 36 thin omitted)

### Community 0 - "MavenClient"
Cohesion: 0.24
Nodes (15): MavenClient, MavenDoc, MavenSearchDocs, MavenSearchResponse, Box, Client, Default, Error (+7 more)

### Community 1 - "Toolchain"
Cohesion: 0.08
Nodes (37): From, detect_java_vendor(), discover_system_jdks(), find_system_jdk(), inspect_jdk_dir(), parse_java_major_version(), Option, Path (+29 more)

### Community 2 - "package_native_app"
Cohesion: 0.06
Nodes (37): Command, Box, Error, Option, Path, Result, Send, String (+29 more)

### Community 3 - "JoltManifest"
Cohesion: 0.15
Nodes (27): HashMap, GraalVmConfig, JoltManifest, PackageConfig, Project, Box, Error, Option (+19 more)

### Community 4 - "CacheManager"
Cohesion: 0.23
Nodes (13): CacheManager, Box, Default, Error, Option, Path, PathBuf, Result (+5 more)

### Community 5 - "init_project"
Cohesion: 0.11
Nodes (21): ensure_ide_configuration(), Box, Error, Option, Path, Result, Send, Sync (+13 more)

### Community 6 - "org.junit.jupiter.api.Test"
Cohesion: 0.17
Nodes (6): org.junit.jupiter.api.Test, MainTest, AppTest, SpringAppTest, SwingAppTest, MainTest

### Community 7 - "build_standalone_jar"
Cohesion: 0.15
Nodes (19): build_jar(), build_standalone_jar(), Box, Error, Option, Path, PathBuf, Result (+11 more)

### Community 8 - "App"
Cohesion: 0.24
Nodes (5): javafx.application.Application, javafx.stage.Stage, App, Override, Main

### Community 9 - "JoltLock"
Cohesion: 0.17
Nodes (13): JoltLock, LockedPackage, Box, Default, Error, Path, Result, Self (+5 more)

### Community 10 - "Spring Boot Templates"
Cohesion: 0.39
Nodes (4): org.springframework.boot.autoconfigure.SpringBootApplication, org.springframework.web.bind.annotation.GetMapping, org.springframework.web.bind.annotation.RestController, Main

### Community 11 - "Picocli Command Templates"
Cohesion: 0.40
Nodes (3): picocli.CommandLine.Command, Override, Main

### Community 12 - "3. Módulo Y: Formateador Integrado de Código Java (`jolt fmt`)"
Cohesion: 0.10
Nodes (20): 1.1 Objetivos y Cambios, 1. Módulo S+: Presets GraalVM y Enriquecimiento de Plantillas (`src/scaffold/`, `src/cli/`), 2. Módulo Arch+: Arquitectura Modular de `src/`, 3.1 Visión y Propósito, 3.2 CLI e Interfaz, 3.3 Opciones de Configuración en `jolt.toml`, 3.4 Arquitectura y Flujo de Ejecución, 3. Módulo Y: Formateador Integrado de Código Java (`jolt fmt`) (+12 more)

### Community 13 - "Modulo P: Comando `jolt sync` y Autoconfiguracion de VS Code e IDEs Java"
Cohesion: 0.50
Nodes (3): Modulo P: Comando `jolt sync` y Autoconfiguracion de VS Code e IDEs Java, Problema Resuelto, Solución Técnica Implementada

### Community 15 - "1. Modulo Q: Empaquetado Binario Nativo Multiplataforma (`jolt package`)"
Cohesion: 0.40
Nodes (4): 1. Modulo Q: Empaquetado Binario Nativo Multiplataforma (`jolt package`), Capacidades:, Jolt - Especificaciones Tecnicas Fase 4, Tareas implementadas:

### Community 47 - "install_in_dir"
Cohesion: 0.24
Nodes (14): install_in_dir(), main(), resolve_target_directories(), Box, Error, Option, Path, PathBuf (+6 more)

### Community 48 - "Jolt - Especificaciones Tecnicas Fase 5"
Cohesion: 0.25
Nodes (7): 1. Modulo R: Paquetes Java y Namespaces (`src/scaffold.rs`, `src/manifest.rs`), 2. Modulo S: Asistente Interactivo y Detección de Contexto (`jolt init`), 3. Modulo T: Workspaces Multimódulo y Dependencias Locales (`[workspace]`, `src/engine.rs`, `src/main.rs`), Capacidades:, Capacidades:, Capacidades:, Jolt - Especificaciones Tecnicas Fase 5

### Community 53 - "openspec-explore/SKILL.md"
Cohesion: 0.18
Nodes (10): Check for context, Ending Discovery, Guardrails, Handling Different Entry Points, OpenSpec Awareness, The Stance, What You Don't Have To Do, What You Might Do (+2 more)

### Community 54 - "opsx-explore.md"
Cohesion: 0.20
Nodes (9): Check for context, Ending Discovery, Guardrails, OpenSpec Awareness, The Stance, What You Don't Have To Do, What You Might Do, When a change exists (+1 more)

### Community 67 - "Requirements"
Cohesion: 0.20
Nodes (9): graalvm-config Specification, Purpose, Requirement: Compilación de binario nativo con GraalVM, Requirement: Detección del ejecutable native-image, Requirement: Esquema de configuración GraalVM en jolt.toml, Requirements, Scenario: Carga exitosa de configuración GraalVM desde jolt.toml, Scenario: Compilación nativa exitosa (+1 more)

### Community 68 - "Requirements"
Cohesion: 0.20
Nodes (9): Purpose, Requirement: Control y notificación antes de descargar JDK, Requirement: Detección multinivel de JDKs locales, Requirement: Evaluación de compatibilidad de versiones de JDK, Requirements, Scenario: Detección exitosa de GraalVM u OpenJDK en PATH o variables de entorno, Scenario: JDK local compatible con la versión solicitada, Scenario: Notificación al usuario por falta de JDK compatible (+1 more)

### Community 69 - "Decisions"
Cohesion: 0.22
Nodes (8): Context, Decisions, Decisión 1: Estructura `InstalledJdk` y detección semántica de versiones, Decisión 2: Compatibilidad de versiones mayores (>= requested), Decisión 3: Notificación de instalación en lugar de descarga silenciosa, Decisión 4: Esquema `[graalvm]` en `jolt.toml`, Goals / Non-Goals, Risks / Trade-offs

### Community 70 - "ADDED Requirements"
Cohesion: 0.22
Nodes (8): ADDED Requirements, Purpose, Requirement: Compilación de binario nativo con GraalVM, Requirement: Detección del ejecutable native-image, Requirement: Esquema de configuración GraalVM en jolt.toml, Scenario: Carga exitosa de configuración GraalVM desde jolt.toml, Scenario: Compilación nativa exitosa, Scenario: Detección del ejecutable native-image de GraalVM

### Community 71 - "ADDED Requirements"
Cohesion: 0.22
Nodes (8): ADDED Requirements, Purpose, Requirement: Control y notificación antes de descargar JDK, Requirement: Detección multinivel de JDKs locales, Requirement: Evaluación de compatibilidad de versiones de JDK, Scenario: Detección exitosa de GraalVM u OpenJDK en PATH o variables de entorno, Scenario: JDK local compatible con la versión solicitada, Scenario: Notificación al usuario por falta de JDK compatible

### Community 72 - "Jolt - Especificaciones Técnicas Fase 6"
Cohesion: 0.17
Nodes (11): 1. Módulo U: Detección Multinivel de JDKs (`src/toolchain.rs`), 2. Módulo V: Compatibilidad y Control de Descargas (`src/toolchain.rs`, `src/cli.rs`), 3. Módulo W: Configuración GraalVM en `jolt.toml` (`src/manifest.rs`), 4. Módulo X: Compilador Nativo GraalVM (`src/engine.rs`, `src/main.rs`), Detección Semántica y Extracción de Vendor:, Esquema de Configuración:, Flujo de Ejecución:, Fuentes de Escaneo (en orden de prioridad): (+3 more)

### Community 73 - "proposal.md"
Cohesion: 0.29
Nodes (6): Capabilities, Impact, Modified Capabilities, New Capabilities, What Changes, Why

### Community 74 - "Commands"
Cohesion: 0.40
Nodes (4): Cli, Commands, Option, String

### Community 75 - "tasks.md"
Cohesion: 0.40
Nodes (4): 1. Soporte de configuración GraalVM en jolt.toml, 2. Detección inteligente de JDKs locales en ToolchainManager, 3. Integración de GraalVM Native Image y comandos CLI, 4. Verificación y pruebas de integración

### Community 77 - "parse_pom_dependencies"
Cohesion: 0.17
Nodes (13): Dependency, DependencyNode, Option, String, Vec, SearchResultItem, parse_pom_dependencies(), Box (+5 more)

### Community 78 - "architecture-modularity Specification"
Cohesion: 0.25
Nodes (7): architecture-modularity Specification, Purpose, Requirement: Organización modular de la base de código en subdirectorios, Requirement: Punto de entrada unificado y despacho de comandos, Requirements, Scenario: Acceso coherente a APIs entre módulos, Scenario: Enrutamiento de subcomandos CLI

## Knowledge Gaps
- **126 isolated node(s):** `jolt`, `com.example:maven-test`, `run_benchmark.sh script`, `The Stance`, `What You Might Do` (+121 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 239 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **36 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Toolchain` connect `Toolchain` to `package_native_app`, `build_standalone_jar`?**
  _High betweenness centrality (0.103) - this node is a cross-community bridge._
- **Why does `ToolchainManager` connect `Toolchain` to `package_native_app`, `install_in_dir`?**
  _High betweenness centrality (0.101) - this node is a cross-community bridge._
- **Why does `package_native_app()` connect `package_native_app` to `Toolchain`, `JoltManifest`, `build_standalone_jar`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **Are the 4 inferred relationships involving `package_native_app()` (e.g. with `build_standalone_jar()` and `find_makensis_binary()`) actually correct?**
  _`package_native_app()` has 4 INFERRED edges - model-reasoned connections that need verification._
- **What connects `jolt`, `com.example:maven-test`, `run_benchmark.sh script` to the rest of the system?**
  _126 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Toolchain` be split into smaller, more focused modules?**
  _Cohesion score 0.0815686274509804 - nodes in this community are weakly interconnected._
- **Should `package_native_app` be split into smaller, more focused modules?**
  _Cohesion score 0.06105457909343201 - nodes in this community are weakly interconnected._
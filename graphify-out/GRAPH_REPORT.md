# Graph Report - jolt  (2026-10-11)

## Corpus Check
- 96 files · ~50,747 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 15 file(s) not represented in the graph (top: .toml 7, .properties 3, (none) 2)

## Summary
- 488 nodes · 646 edges · 60 communities (23 shown, 37 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 32 edges (avg confidence: 0.9)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `00a469cc`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- MavenClient
- Toolchain
- path
- manifest.rs
- CacheManager
- scaffold/mod.rs
- org.junit.jupiter.api.Test
- App.java
- lockfile.rs
- 3. Módulo Y: Formateador Integrado de Código Java (`jolt fmt`)
- Modulo P: Comando `jolt sync` y Autoconfiguracion de VS Code e IDEs Java
- Archivo de Modulo O: Gestion de Dev-Dependencies, Menu Interactivo y Soporte IDE (v0.2.0)
- 1. Modulo Q: Empaquetado Binario Nativo Multiplataforma (`jolt package`)
- Java Main Entrypoint 2
- swing/src/main/java/Main.java
- web/src/main/java/Main.java
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
- Benchmark Output Results
- Phase 1 Documentation
- Phase 2 Documentation
- Phase 3 Documentation
- Phase 4 Documentation
- Maven Test Project
- Jolt Configuration
- Jolt Root Repo
- Jolt - Especificaciones Tecnicas Fase 5
- Modulo Q: Empaquetado Binario Nativo Multiplataforma (jpackage)
- Modulo R: Paquetes Java y Namespaces Estructurados
- Modulo S: Asistente Interactivo de Inicializacion
- Modulo T: Workspaces Multimodulo y Dependencias Locales
- .status
- Módulo U: Empaquetado en Windows con UPX y NSIS (Windows Installer)
- Requirements
- Requirements
- Decisions
- ADDED Requirements
- ADDED Requirements
- Jolt - Especificaciones Técnicas Fase 6
- proposal.md
- Commands
- pom_parser.rs
- cli/src/main/java/Main.java

## God Nodes (most connected - your core abstractions)
1. `CacheManager` - 20 edges
2. `MavenClient` - 18 edges
3. `JoltManifest` - 17 edges
4. `Toolchain` - 17 edges
5. `Commands` - 14 edges
6. `ToolchainManager` - 12 edges
7. `parse_pom_dependencies()` - 10 edges
8. `JoltLock` - 8 edges
9. `package_native_app()` - 8 edges
10. `init_project()` - 7 edges

## Surprising Connections (you probably didn't know these)
- `build_jar()` --references--> `Toolchain`  [EXTRACTED]
  src/packaging/jar.rs → src/toolchain/mod.rs
- `package_native_app()` --calls--> `find_makensis_binary()`  [INFERRED]
  src/packaging/jpackage.rs → src/packaging/nsis.rs
- `init_project()` --calls--> `print_available_templates()`  [INFERRED]
  src/scaffold/mod.rs → src/scaffold/templates.rs
- `AGENTS Graphify Usage` --semantically_similar_to--> `GEMINI Graphify Usage`  [INFERRED] [semantically similar]
  AGENTS.md → GEMINI.md
- `install_in_dir()` --references--> `CacheManager`  [EXTRACTED]
  src/main.rs → src/core/cache.rs

## Import Cycles
- 2-file cycle: `src/toolchain/downloader.rs -> src/toolchain/mod.rs -> src/toolchain/downloader.rs`

## Hyperedges (group relationships)
- **Jolt Modules** — docs_archive_modulo_a_cli_scaffolding_modulo_a, docs_archive_modulo_b_maven_resolver_modulo_b, docs_archive_modulo_c_cache_storage_modulo_c, docs_archive_modulo_d_toolchain_provisioner_modulo_d, docs_archive_modulo_e_build_run_engine_modulo_e, docs_archive_modulo_f_i_fatjar_resources_modulo_f, docs_archive_modulo_f_i_fatjar_resources_modulo_i, docs_archive_modulo_g_unit_testing_modulo_g, docs_archive_modulo_h_watch_mode_modulo_h, docs_archive_modulo_j_system_project_check_modulo_j, docs_archive_modulo_k_lockfile_modulo_k, docs_archive_modulo_l_templates_modulo_l, docs_archive_modulo_m_remove_dependency_modulo_m, docs_archive_modulo_n_search_modulo_n [EXTRACTED 1.00]

## Communities (60 total, 37 thin omitted)

### Community 0 - "MavenClient"
Cohesion: 0.10
Nodes (14): install_in_dir(), install_one_dependency(), install_resolved_in_dir(), main(), resolve_target_directories(), sync_in_dir(), dependency(), MavenClient (+6 more)

### Community 1 - "Toolchain"
Cohesion: 0.13
Nodes (11): detect_java_vendor(), discover_system_jdks(), find_system_jdk(), inspect_jdk_dir(), parse_java_major_version(), download_and_extract_jdk(), find_binaries_in_dir(), InstalledJdk (+3 more)

### Community 2 - "path"
Cohesion: 0.10
Nodes (11): build_jar(), build_standalone_jar(), compress_to_tar_gz(), package_native_app(), test_compress_to_tar_gz(), test_generate_nsis_script_content(), build_native_image(), find_makensis_binary() (+3 more)

### Community 3 - "manifest.rs"
Cohesion: 0.15
Nodes (14): GraalVmConfig, JoltManifest, PackageConfig, Project, test_add_and_remove_dev_dependency(), test_parse_basic_manifest(), test_parse_dependency_with_path(), test_parse_graalvm_config() (+6 more)

### Community 4 - "CacheManager"
Cohesion: 0.17
Nodes (3): CacheManager, test_cache_save_and_link(), SystemChecker

### Community 5 - "scaffold/mod.rs"
Cohesion: 0.21
Nodes (9): ensure_ide_configuration(), init_project(), test_ensure_ide_configuration_creates_files(), test_init_project_web_template_has_test(), test_init_project_with_graalvm(), test_init_project_with_package(), test_init_project_workspace(), AVAILABLE_TEMPLATES (+1 more)

### Community 6 - "org.junit.jupiter.api.Test"
Cohesion: 0.16
Nodes (5): MainTest, AppTest, SpringAppTest, SwingAppTest, MainTest

### Community 9 - "lockfile.rs"
Cohesion: 0.22
Nodes (3): JoltLock, LockedPackage, test_lockfile_serialization()

### Community 12 - "3. Módulo Y: Formateador Integrado de Código Java (`jolt fmt`)"
Cohesion: 0.10
Nodes (20): 1.1 Objetivos y Cambios, 1. Módulo S+: Presets GraalVM y Enriquecimiento de Plantillas (`src/scaffold/`, `src/cli/`), 2. Módulo Arch+: Arquitectura Modular de `src/`, 3.1 Visión y Propósito, 3.2 CLI e Interfaz, 3.3 Opciones de Configuración en `jolt.toml`, 3.4 Arquitectura y Flujo de Ejecución, 3. Módulo Y: Formateador Integrado de Código Java (`jolt fmt`) (+12 more)

### Community 13 - "Modulo P: Comando `jolt sync` y Autoconfiguracion de VS Code e IDEs Java"
Cohesion: 0.50
Nodes (3): Modulo P: Comando `jolt sync` y Autoconfiguracion de VS Code e IDEs Java, Problema Resuelto, Solución Técnica Implementada

### Community 15 - "1. Modulo Q: Empaquetado Binario Nativo Multiplataforma (`jolt package`)"
Cohesion: 0.40
Nodes (4): 1. Modulo Q: Empaquetado Binario Nativo Multiplataforma (`jolt package`), Capacidades:, Jolt - Especificaciones Tecnicas Fase 4, Tareas implementadas:

### Community 48 - "Jolt - Especificaciones Tecnicas Fase 5"
Cohesion: 0.25
Nodes (7): 1. Modulo R: Paquetes Java y Namespaces (`src/scaffold.rs`, `src/manifest.rs`), 2. Modulo S: Asistente Interactivo y Detección de Contexto (`jolt init`), 3. Modulo T: Workspaces Multimódulo y Dependencias Locales (`[workspace]`, `src/engine.rs`, `src/main.rs`), Capacidades:, Capacidades:, Capacidades:, Jolt - Especificaciones Tecnicas Fase 5

### Community 53 - ".status"
Cohesion: 0.06
Nodes (20): Check for context, Ending Discovery, Guardrails, Handling Different Entry Points, OpenSpec Awareness, The Stance, What You Don't Have To Do, What You Might Do (+12 more)

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
Cohesion: 0.11
Nodes (14): Cli, Commands, Add, Build, Check, Init, Install, Package (+6 more)

### Community 77 - "pom_parser.rs"
Cohesion: 0.17
Nodes (11): Dependency, SearchResultItem, is_in_dependency(), is_in_project_dependencies(), is_in_properties(), is_parent_group(), is_parent_version(), is_project_group() (+3 more)

### Community 78 - "cli/src/main/java/Main.java"
Cohesion: 0.10
Nodes (12): 1. Soporte de configuración GraalVM en jolt.toml, 2. Detección inteligente de JDKs locales en ToolchainManager, 3. Integración de GraalVM Native Image y comandos CLI, 4. Verificación y pruebas de integración, architecture-modularity Specification, Purpose, Requirement: Organización modular de la base de código en subdirectorios, Requirement: Punto de entrada unificado y despacho de comandos (+4 more)

## Knowledge Gaps
- **50 isolated node(s):** `jolt`, `com.example:maven-test`, `Init`, `Add`, `Search` (+45 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 225 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **37 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `CacheManager` connect `CacheManager` to `MavenClient`, `path`?**
  _High betweenness centrality (0.024) - this node is a cross-community bridge._
- **What connects `jolt`, `com.example:maven-test`, `Init` to the rest of the system?**
  _50 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `MavenClient` be split into smaller, more focused modules?**
  _Cohesion score 0.1028225806451613 - nodes in this community are weakly interconnected._
- **Why does `MavenClient` connect `MavenClient` to `pom_parser.rs`?**
  _High betweenness centrality (0.022) - this node is a cross-community bridge._
- **Should `Toolchain` be split into smaller, more focused modules?**
  _Cohesion score 0.12903225806451613 - nodes in this community are weakly interconnected._
- **Should `path` be split into smaller, more focused modules?**
  _Cohesion score 0.09716599190283401 - nodes in this community are weakly interconnected._
- **Should `manifest.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1452991452991453 - nodes in this community are weakly interconnected._
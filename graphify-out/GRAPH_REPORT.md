# Graph Report - jolt  (2026-11-22)

## Corpus Check
- 43 files · ~66,532 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 18 file(s) not represented in the graph (top: .toml 7, (none) 3, .properties 3)

## Summary
- 565 nodes · 798 edges · 77 communities (26 shown, 51 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 20 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `5ffb03cf`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- MavenClient
- Joltv documentation
- ToolchainManager
- manifest.rs
- maven_client.rs
- scaffold/mod.rs
- org.junit.jupiter.api.Test
- DocsLayout.astro
- App.java
- scaffold/mod.rs
- AGENTS.md
- CLAUDE.md
- 3. Módulo Y: Formateador Integrado de Código Java (`jolt fmt`)
- Modulo P: Comando `jolt sync` y Autoconfiguracion de VS Code e IDEs Java
- config.yaml
- 1. Modulo Q: Empaquetado Binario Nativo Multiplataforma (`jolt package`)
- Java Main Entrypoint 2
- swing/src/main/java/Main.java
- web/src/main/java/Main.java
- Graphify Markdown Rules
- Benchmark Scripts
- lockfile.rs
- Module B Documentation
- Module C Documentation
- Module D Documentation
- Module E Documentation
- Module F Documentation
- Module I Documentation
- Module H Documentation
- Module J Documentation
- Module K Documentation
- Module L Documentation
- Module M Documentation
- Module N Documentation
- tsconfig.json
- Módulo U: Empaquetado en Windows con UPX y NSIS (Windows Installer)
- Jolt Architecture Vision
- Phase 1 Documentation
- Phase 2 Documentation
- Phase 3 Documentation
- Phase 4 Documentation
- Maven Test Project
- Jolt Configuration
- Jolt Root Repo
- Modulo F: Fat-JAR Bundler
- Jolt - Especificaciones Tecnicas Fase 5
- Modulo Q: Empaquetado Binario Nativo Multiplataforma (jpackage)
- Modulo R: Paquetes Java y Namespaces Estructurados
- Modulo S: Asistente Interactivo de Inicializacion
- Modulo T: Workspaces Multimodulo y Dependencias Locales
- Modulo L: Templates
- Modulo M: Remove Dependency
- Modulo N: Search
- Benchmark Results
- Decisions
- ADDED Requirements
- ADDED Requirements
- Jolt - Especificaciones Técnicas Fase 6
- proposal.md
- Commands
- Jolt

## God Nodes (most connected - your core abstractions)
1. `CacheManager` - 26 edges
2. `MavenClient` - 23 edges
3. `JoltManifest` - 19 edges
4. `Commands` - 15 edges
5. `parse_pom_dependencies()` - 14 edges
6. `Toolchain` - 12 edges
7. `JoltLock` - 11 edges
8. `init_project()` - 10 edges
9. `package_native_app()` - 9 edges
10. `Main` - 8 edges

## Surprising Connections (you probably didn't know these)
- `package_native_app()` --references--> `JoltManifest`  [INFERRED]
  src/packaging/jpackage.rs → src/core/manifest.rs
- `test_generate_nsis_script_content()` --calls--> `generate_nsis_script()`  [INFERRED]
  src/packaging/mod.rs → src/packaging/nsis.rs
- `test_parse_version_formats()` --calls--> `parse_java_major_version()`  [INFERRED]
  src/toolchain/mod.rs → src/toolchain/detector.rs
- `test_parse_pom_ignores_dependency_management_and_resolves_properties()` --calls--> `parse_pom_dependencies()`  [INFERRED]
  src/resolver/mod.rs → src/resolver/pom_parser.rs
- `test_parse_pom_xml()` --calls--> `parse_pom_dependencies()`  [INFERRED]
  src/resolver/mod.rs → src/resolver/pom_parser.rs

## Import Cycles
- 1-file cycle: `src/cli/args.rs -> src/cli/args.rs`
- 2-file cycle: `src/toolchain/detector.rs -> src/toolchain/mod.rs -> src/toolchain/detector.rs`
- 2-file cycle: `src/resolver/maven_client.rs -> src/resolver/mod.rs -> src/resolver/maven_client.rs`
- 2-file cycle: `src/resolver/mod.rs -> src/resolver/pom_parser.rs -> src/resolver/mod.rs`
- 3-file cycle: `src/resolver/maven_client.rs -> src/resolver/pom_parser.rs -> src/resolver/mod.rs -> src/resolver/maven_client.rs`

## Hyperedges (group relationships)
- **Jolt Modules** — docs_archive_modulo_a_cli_scaffolding_modulo_a, docs_archive_modulo_b_maven_resolver_modulo_b, docs_archive_modulo_c_cache_storage_modulo_c, docs_archive_modulo_d_toolchain_provisioner_modulo_d, docs_archive_modulo_e_build_run_engine_modulo_e, docs_archive_modulo_f_i_fatjar_resources_modulo_f, docs_archive_modulo_f_i_fatjar_resources_modulo_i, docs_archive_modulo_g_unit_testing_modulo_g, docs_archive_modulo_h_watch_mode_modulo_h, docs_archive_modulo_j_system_project_check_modulo_j, docs_archive_modulo_k_lockfile_modulo_k, docs_archive_modulo_l_templates_modulo_l, docs_archive_modulo_m_remove_dependency_modulo_m, docs_archive_modulo_n_search_modulo_n [EXTRACTED 1.00]

## Communities (77 total, 51 thin omitted)

### Community 0 - "MavenClient"
Cohesion: 0.10
Nodes (19): build_jar(), build_standalone_jar(), compress_to_tar_gz(), package_native_app(), test_compress_to_tar_gz(), test_generate_nsis_script_content(), find_makensis_binary(), generate_nsis_script() (+11 more)

### Community 1 - "Joltv documentation"
Cohesion: 0.07
Nodes (17): Archivo de Modulo O: Gestion de Dev-Dependencies, Menu Interactivo y Soporte IDE (v0.2.0), Resumen de Tareas Cumplidas, CacheManager, concurrent_publications_never_leave_partial_content(), publishes_jar_without_leaving_temporary_files(), refuses_to_link_an_incomplete_cached_jar(), test_cache_save_and_link(), configured_maven_client() (+9 more)

### Community 2 - "ToolchainManager"
Cohesion: 0.22
Nodes (4): SystemChecker, does_not_download_when_system_jdk_is_missing(), test_find_system_jdk(), ToolchainManager

### Community 3 - "manifest.rs"
Cohesion: 0.05
Nodes (38): defaults, runs-on, steps, concurrency, cancel-in-progress, group, run, environment (+30 more)

### Community 4 - "maven_client.rs"
Cohesion: 0.10
Nodes (22): caches_pom_metadata_between_requests(), dependency(), MavenDoc, MavenSearchDocs, MavenSearchResponse, pom(), reports_context_when_maven_server_does_not_respond(), resolves_transitive_dependencies_and_skips_non_runtime_scopes() (+14 more)

### Community 5 - "scaffold/mod.rs"
Cohesion: 0.14
Nodes (15): JoltManifest, PackageConfig, Project, RepositoryConfig, test_add_and_remove_dev_dependency(), test_moving_dependency_between_scopes_removes_stale_entry(), test_parse_basic_manifest(), test_parse_dependency_with_path() (+7 more)

### Community 6 - "org.junit.jupiter.api.Test"
Cohesion: 0.10
Nodes (14): Cli, Commands, Add, Build, Check, Init, Install, Package (+6 more)

### Community 7 - "DocsLayout.astro"
Cohesion: 0.10
Nodes (20): 1.1 Objetivos y Cambios, 1. Módulo S+: Presets GraalVM y Enriquecimiento de Plantillas (`src/scaffold/`, `src/cli/`), 2. Módulo Arch+: Arquitectura Modular de `src/`, 3.1 Visión y Propósito, 3.2 CLI e Interfaz, 3.3 Opciones de Configuración en `jolt.toml`, 3.4 Arquitectura y Flujo de Ejecución, 3. Módulo Y: Formateador Integrado de Código Java (`jolt fmt`) (+12 more)

### Community 8 - "App.java"
Cohesion: 0.16
Nodes (5): MainTest, AppTest, SpringAppTest, SwingAppTest, MainTest

### Community 9 - "scaffold/mod.rs"
Cohesion: 0.19
Nodes (12): ensure_ide_configuration(), init_project(), javafx_classifier(), test_cli_template_uses_tui4j_without_picocli(), test_ensure_ide_configuration_creates_files(), test_init_project_omits_graalvm_config(), test_init_project_web_template_has_test(), test_init_project_with_package() (+4 more)

### Community 12 - "3. Módulo Y: Formateador Integrado de Código Java (`jolt fmt`)"
Cohesion: 0.17
Nodes (11): 1. Módulo U: Detección Multinivel de JDKs (`src/toolchain.rs`), 2. Módulo V: Compatibilidad y Control de Descargas (`src/toolchain.rs`, `src/cli.rs`), 3. Módulo W: Configuración GraalVM en `jolt.toml` (`src/manifest.rs`), 4. Módulo X: Compilador Nativo GraalVM (`src/engine.rs`, `src/main.rs`), Detección Semántica y Extracción de Vendor:, Esquema de Configuración:, Flujo de Ejecución:, Fuentes de Escaneo (en orden de prioridad): (+3 more)

### Community 13 - "Modulo P: Comando `jolt sync` y Autoconfiguracion de VS Code e IDEs Java"
Cohesion: 0.18
Nodes (10): Check for context, Ending Discovery, Guardrails, Handling Different Entry Points, OpenSpec Awareness, The Stance, What You Don't Have To Do, What You Might Do (+2 more)

### Community 14 - "config.yaml"
Cohesion: 0.18
Nodes (10): guidance, guidance, context, operations, apply, archive, rules, proposal (+2 more)

### Community 15 - "1. Modulo Q: Empaquetado Binario Nativo Multiplataforma (`jolt package`)"
Cohesion: 0.20
Nodes (9): Check for context, Ending Discovery, Guardrails, OpenSpec Awareness, The Stance, What You Don't Have To Do, What You Might Do, When a change exists (+1 more)

### Community 16 - "Java Main Entrypoint 2"
Cohesion: 0.20
Nodes (9): Purpose, Requirement: Control y notificación antes de descargar JDK, Requirement: Detección multinivel de JDKs locales, Requirement: Evaluación de compatibilidad de versiones de JDK, Requirements, Scenario: Detección exitosa de GraalVM u OpenJDK en PATH o variables de entorno, Scenario: JDK local compatible con la versión solicitada, Scenario: Notificación al usuario por falta de JDK compatible (+1 more)

### Community 17 - "swing/src/main/java/Main.java"
Cohesion: 0.22
Nodes (8): Context, Decisions, Decisión 1: Estructura `InstalledJdk` y detección semántica de versiones, Decisión 2: Compatibilidad de versiones mayores (>= requested), Decisión 3: Notificación de instalación en lugar de descarga silenciosa, Decisión 4: Esquema `[graalvm]` en `jolt.toml`, Goals / Non-Goals, Risks / Trade-offs

### Community 18 - "web/src/main/java/Main.java"
Cohesion: 0.22
Nodes (8): ADDED Requirements, Purpose, Requirement: Compilación de binario nativo con GraalVM, Requirement: Detección del ejecutable native-image, Requirement: Esquema de configuración GraalVM en jolt.toml, Scenario: Carga exitosa de configuración GraalVM desde jolt.toml, Scenario: Compilación nativa exitosa, Scenario: Detección del ejecutable native-image de GraalVM

### Community 19 - "Graphify Markdown Rules"
Cohesion: 0.22
Nodes (8): ADDED Requirements, Purpose, Requirement: Control y notificación antes de descargar JDK, Requirement: Detección multinivel de JDKs locales, Requirement: Evaluación de compatibilidad de versiones de JDK, Scenario: Detección exitosa de GraalVM u OpenJDK en PATH o variables de entorno, Scenario: JDK local compatible con la versión solicitada, Scenario: Notificación al usuario por falta de JDK compatible

### Community 21 - "lockfile.rs"
Cohesion: 0.19
Nodes (7): accepts_manifest_dependency_with_matching_locked_version(), JoltLock, LockedPackage, rejects_manifest_dependency_with_different_locked_version(), save_replaces_lockfile_without_leaving_temporary_files(), temporary_path(), test_lockfile_serialization()

### Community 22 - "Module B Documentation"
Cohesion: 0.25
Nodes (7): 1. Modulo R: Paquetes Java y Namespaces (`src/scaffold.rs`, `src/manifest.rs`), 2. Modulo S: Asistente Interactivo y Detección de Contexto (`jolt init`), 3. Modulo T: Workspaces Multimódulo y Dependencias Locales (`[workspace]`, `src/engine.rs`, `src/main.rs`), Capacidades:, Capacidades:, Capacidades:, Jolt - Especificaciones Tecnicas Fase 5

### Community 23 - "Module C Documentation"
Cohesion: 0.25
Nodes (7): architecture-modularity Specification, Purpose, Requirement: Organización modular de la base de código en subdirectorios, Requirement: Punto de entrada unificado y despacho de comandos, Requirements, Scenario: Acceso coherente a APIs entre módulos, Scenario: Enrutamiento de subcomandos CLI

### Community 24 - "Module D Documentation"
Cohesion: 0.29
Nodes (6): Deployment base path, Documentation map, Joltv documentation, Local development, Production build, Project structure

### Community 25 - "Module E Documentation"
Cohesion: 0.29
Nodes (6): Capabilities, Impact, Modified Capabilities, New Capabilities, What Changes, Why

### Community 27 - "Module I Documentation"
Cohesion: 0.40
Nodes (4): 1. Modulo Q: Empaquetado Binario Nativo Multiplataforma (`jolt package`), Capacidades:, Jolt - Especificaciones Tecnicas Fase 4, Tareas implementadas:

### Community 29 - "Module H Documentation"
Cohesion: 0.40
Nodes (4): 1. Soporte de configuración GraalVM en jolt.toml, 2. Detección inteligente de JDKs locales en ToolchainManager, 3. Integración de GraalVM Native Image y comandos CLI, 4. Verificación y pruebas de integración

### Community 30 - "Module J Documentation"
Cohesion: 0.50
Nodes (3): Modulo P: Comando `jolt sync` y Autoconfiguracion de VS Code e IDEs Java, Problema Resuelto, Solución Técnica Implementada

## Knowledge Gaps
- **50 isolated node(s):** `Add`, `Build`, `Check`, `Init`, `Install` (+45 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 245 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **51 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `CacheManager` connect `Joltv documentation` to `ToolchainManager`, `scaffold/mod.rs`?**
  _High betweenness centrality (0.026) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `JoltManifest` (e.g. with `.validate_manifest_dependencies()` and `package_native_app()`) actually correct?**
  _`JoltManifest` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Add`, `Build`, `Check` to the rest of the system?**
  _50 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `MavenClient` be split into smaller, more focused modules?**
  _Cohesion score 0.1036077705827937 - nodes in this community are weakly interconnected._
- **Why does `MavenClient` connect `Joltv documentation` to `maven_client.rs`?**
  _High betweenness centrality (0.023) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `parse_pom_dependencies()` (e.g. with `test_parse_pom_ignores_dependency_management_and_resolves_properties()` and `test_parse_pom_xml()`) actually correct?**
  _`parse_pom_dependencies()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Should `Joltv documentation` be split into smaller, more focused modules?**
  _Cohesion score 0.07239819004524888 - nodes in this community are weakly interconnected._
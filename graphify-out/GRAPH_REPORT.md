# Graph Report - jolt  (2026-08-30)

## Corpus Check
- 71 files · ~43,225 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 378 nodes · 712 edges · 67 communities (19 shown, 36 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.95)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `b90041bf`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Maven Resolver Component
- BuildEngine
- Cache Storage Management
- JoltManifest
- JoltLock
- Toolchain Provisioning
- org.junit.jupiter.api.Test
- System Check Components
- App
- init_project
- Spring Boot Templates
- Picocli Command Templates
- CLI Interfaces
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

## God Nodes (most connected - your core abstractions)
1. `CacheManager` - 21 edges
2. `BuildEngine` - 19 edges
3. `JoltManifest` - 18 edges
4. `MavenClient` - 16 edges
5. `Toolchain` - 16 edges
6. `ToolchainManager` - 14 edges
7. `init_project()` - 11 edges
8. `JoltLock` - 10 edges
9. `install_in_dir()` - 10 edges
10. `sync_in_dir()` - 10 edges

## Surprising Connections (you probably didn't know these)
- `AGENTS Graphify Usage` --semantically_similar_to--> `GEMINI Graphify Usage`  [INFERRED] [semantically similar]
  AGENTS.md → GEMINI.md
- `install_in_dir()` --references--> `CacheManager`  [EXTRACTED]
  src/main.rs → src/cache.rs
- `sync_in_dir()` --references--> `CacheManager`  [EXTRACTED]
  src/main.rs → src/cache.rs
- `install_in_dir()` --references--> `MavenClient`  [EXTRACTED]
  src/main.rs → src/maven.rs
- `sync_in_dir()` --references--> `MavenClient`  [EXTRACTED]
  src/main.rs → src/maven.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Jolt Modules** — docs_archive_modulo_a_cli_scaffolding_modulo_a, docs_archive_modulo_b_maven_resolver_modulo_b, docs_archive_modulo_c_cache_storage_modulo_c, docs_archive_modulo_d_toolchain_provisioner_modulo_d, docs_archive_modulo_e_build_run_engine_modulo_e, docs_archive_modulo_f_i_fatjar_resources_modulo_f, docs_archive_modulo_f_i_fatjar_resources_modulo_i, docs_archive_modulo_g_unit_testing_modulo_g, docs_archive_modulo_h_watch_mode_modulo_h, docs_archive_modulo_j_system_project_check_modulo_j, docs_archive_modulo_k_lockfile_modulo_k, docs_archive_modulo_l_templates_modulo_l, docs_archive_modulo_m_remove_dependency_modulo_m, docs_archive_modulo_n_search_modulo_n [EXTRACTED 1.00]

## Communities (67 total, 36 thin omitted)

### Community 0 - "Maven Resolver Component"
Cohesion: 0.21
Nodes (19): Dependency, DependencyNode, MavenClient, MavenDoc, MavenSearchDocs, MavenSearchResponse, Box, Client (+11 more)

### Community 1 - "BuildEngine"
Cohesion: 0.24
Nodes (18): Child, BuildEngine, Box, Error, Option, Path, PathBuf, Result (+10 more)

### Community 2 - "Cache Storage Management"
Cohesion: 0.23
Nodes (13): CacheManager, Box, Default, Error, Option, Path, PathBuf, Result (+5 more)

### Community 3 - "JoltManifest"
Cohesion: 0.16
Nodes (25): HashMap, JoltManifest, PackageConfig, Project, Box, Error, Option, Path (+17 more)

### Community 4 - "JoltLock"
Cohesion: 0.18
Nodes (13): JoltLock, LockedPackage, Box, Default, Error, Path, Result, Self (+5 more)

### Community 5 - "Toolchain Provisioning"
Cohesion: 0.19
Nodes (13): Box, Client, Default, Error, Option, Path, PathBuf, Result (+5 more)

### Community 6 - "org.junit.jupiter.api.Test"
Cohesion: 0.21
Nodes (5): org.junit.jupiter.api.Test, MainTest, AppTest, SpringAppTest, SwingAppTest

### Community 7 - "System Check Components"
Cohesion: 0.17
Nodes (10): Command, Box, Error, Option, Path, Result, Send, String (+2 more)

### Community 8 - "App"
Cohesion: 0.24
Nodes (5): javafx.application.Application, javafx.stage.Stage, App, Override, Main

### Community 9 - "init_project"
Cohesion: 0.27
Nodes (13): ensure_ide_configuration(), init_project(), print_available_templates(), Box, Error, Option, Path, Result (+5 more)

### Community 10 - "Spring Boot Templates"
Cohesion: 0.39
Nodes (4): org.springframework.boot.autoconfigure.SpringBootApplication, org.springframework.web.bind.annotation.GetMapping, org.springframework.web.bind.annotation.RestController, Main

### Community 11 - "Picocli Command Templates"
Cohesion: 0.40
Nodes (3): picocli.CommandLine.Command, Override, Main

### Community 12 - "CLI Interfaces"
Cohesion: 0.50
Nodes (4): Cli, Commands, Option, String

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

## Knowledge Gaps
- **71 isolated node(s):** `jolt`, `com.example:maven-test`, `run_benchmark.sh script`, `The Stance`, `What You Might Do` (+66 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 129 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **36 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `ToolchainManager` connect `Toolchain Provisioning` to `install_in_dir`, `System Check Components`?**
  _High betweenness centrality (0.143) - this node is a cross-community bridge._
- **Why does `Toolchain` connect `BuildEngine` to `Toolchain Provisioning`?**
  _High betweenness centrality (0.119) - this node is a cross-community bridge._
- **Why does `JoltManifest` connect `JoltManifest` to `BuildEngine`?**
  _High betweenness centrality (0.083) - this node is a cross-community bridge._
- **What connects `jolt`, `com.example:maven-test`, `run_benchmark.sh script` to the rest of the system?**
  _71 weakly-connected nodes found - possible documentation gaps or missing edges._
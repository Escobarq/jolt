# Graph Report - jolt  (2026-08-18)

## Corpus Check
- Corpus is ~14,583 words - fits in a single context window. You may not need a graph.

## Summary
- 285 nodes · 524 edges · 47 communities (14 shown, 33 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 2 edges (avg confidence: 0.88)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Maven Resolver Component
- Build & Run Engine
- Cache Storage Management
- Manifest Configuration
- Lockfile Management
- Toolchain Provisioning
- Unit Test Components
- System Check Components
- JavaFX Demo Apps
- CLI Scaffolding Scaffold
- Spring Boot Templates
- Picocli Command Templates
- CLI Interfaces
- Main Class Entrypoint
- Graphify Plugin Settings
- Java Main Entrypoint 1
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

## God Nodes (most connected - your core abstractions)
1. `CacheManager` - 18 edges
2. `Toolchain` - 15 edges
3. `ToolchainManager` - 14 edges
4. `MavenClient` - 13 edges
5. `BuildEngine` - 12 edges
6. `JoltLock` - 10 edges
7. `JoltManifest` - 9 edges
8. `init_project()` - 8 edges
9. `Main` - 6 edges
10. `SystemChecker` - 5 edges

## Surprising Connections (you probably didn't know these)
- `AGENTS Graphify Usage` --semantically_similar_to--> `GEMINI Graphify Usage`  [INFERRED] [semantically similar]
  AGENTS.md → GEMINI.md
- `Modulo A Spec` --cites--> `Modulo A: CLI Scaffolding`  [EXTRACTED]
  docs/specs.md → docs/archive/modulo-a-cli-scaffolding.md
- `Modulo B Spec` --cites--> `Modulo B: Maven Resolver`  [EXTRACTED]
  docs/specs.md → docs/archive/modulo-b-maven-resolver.md
- `Modulo C Spec` --cites--> `Modulo C: Cache Storage`  [EXTRACTED]
  docs/specs.md → docs/archive/modulo-c-cache-storage.md
- `Modulo D Spec` --cites--> `Modulo D: Toolchain Provisioner`  [EXTRACTED]
  docs/specs.md → docs/archive/modulo-d-toolchain-provisioner.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Jolt Modules** — docs_archive_modulo_a_cli_scaffolding_modulo_a, docs_archive_modulo_b_maven_resolver_modulo_b, docs_archive_modulo_c_cache_storage_modulo_c, docs_archive_modulo_d_toolchain_provisioner_modulo_d, docs_archive_modulo_e_build_run_engine_modulo_e, docs_archive_modulo_f_i_fatjar_resources_modulo_f, docs_archive_modulo_f_i_fatjar_resources_modulo_i, docs_archive_modulo_g_unit_testing_modulo_g, docs_archive_modulo_h_watch_mode_modulo_h, docs_archive_modulo_j_system_project_check_modulo_j, docs_archive_modulo_k_lockfile_modulo_k, docs_archive_modulo_l_templates_modulo_l, docs_archive_modulo_m_remove_dependency_modulo_m, docs_archive_modulo_n_search_modulo_n [EXTRACTED 1.00]

## Communities (47 total, 33 thin omitted)

### Community 0 - "Maven Resolver Component"
Cohesion: 0.21
Nodes (19): Dependency, DependencyNode, MavenClient, MavenDoc, MavenSearchDocs, MavenSearchResponse, Box, Client (+11 more)

### Community 1 - "Build & Run Engine"
Cohesion: 0.30
Nodes (15): Child, BuildEngine, Box, Error, Option, Path, PathBuf, Result (+7 more)

### Community 2 - "Cache Storage Management"
Cohesion: 0.22
Nodes (13): CacheManager, Box, Default, Error, Option, Path, PathBuf, Result (+5 more)

### Community 3 - "Manifest Configuration"
Cohesion: 0.17
Nodes (16): Main, HashMap, java.util.HashMap, JoltManifest, Project, Box, Error, Option (+8 more)

### Community 4 - "Lockfile Management"
Cohesion: 0.16
Nodes (13): JoltLock, LockedPackage, Box, Default, Error, Path, Result, Self (+5 more)

### Community 5 - "Toolchain Provisioning"
Cohesion: 0.19
Nodes (13): Box, Client, Default, Error, Option, Path, PathBuf, Result (+5 more)

### Community 6 - "Unit Test Components"
Cohesion: 0.16
Nodes (6): CalculatorTest, org.junit.jupiter.api.Test, MainTest, AppTest, SpringAppTest, SwingAppTest

### Community 7 - "System Check Components"
Cohesion: 0.17
Nodes (10): Command, Box, Error, Option, Path, Result, Send, String (+2 more)

### Community 8 - "JavaFX Demo Apps"
Cohesion: 0.24
Nodes (6): javafx.application.Application, App, Override, javafx.stage.Stage, App, Override

### Community 9 - "CLI Scaffolding Scaffold"
Cohesion: 0.25
Nodes (8): init_project(), print_available_templates(), Box, Error, Option, Result, Send, Sync

### Community 10 - "Spring Boot Templates"
Cohesion: 0.39
Nodes (4): org.springframework.boot.autoconfigure.SpringBootApplication, org.springframework.web.bind.annotation.GetMapping, org.springframework.web.bind.annotation.RestController, Main

### Community 11 - "Picocli Command Templates"
Cohesion: 0.40
Nodes (3): picocli.CommandLine.Command, Override, Main

### Community 12 - "CLI Interfaces"
Cohesion: 0.50
Nodes (4): Cli, Commands, Option, String

## Knowledge Gaps
- **41 isolated node(s):** `jolt`, `com.example:maven-test`, `run_benchmark.sh script`, `Graphify Rules`, `Graphify Workflow` (+36 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **33 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `ToolchainManager` connect `Toolchain Provisioning` to `Lockfile Management`, `System Check Components`?**
  _High betweenness centrality (0.081) - this node is a cross-community bridge._
- **Why does `CacheManager` connect `Cache Storage Management` to `Lockfile Management`, `System Check Components`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **Why does `Toolchain` connect `Build & Run Engine` to `Toolchain Provisioning`?**
  _High betweenness centrality (0.050) - this node is a cross-community bridge._
- **What connects `jolt`, `com.example:maven-test`, `run_benchmark.sh script` to the rest of the system?**
  _41 weakly-connected nodes found - possible documentation gaps or missing edges._
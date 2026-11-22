# Jolt: ¿Cómo gestiona proyectos Java de forma moderna?

Este documento explora el diseño arquitectónico y funcional de un gestor de paquetes y proyectos para Java, inspirado en el rendimiento y la experiencia de desarrollo de herramientas modernas como **uv** (Python) y **bun** (JavaScript).

**Estado actual:** Jolt v0.7.3 - Todos los módulos funcionales están implementados y archivados. Consulte [`docs/plan.md`](plan.md) para el estado completo del proyecto.

El objetivo principal es simplificar radicalmente la creación de aplicaciones Java, reduciendo la verbosidad y multiplicando la velocidad operativa, manteniendo total compatibilidad con el ecosistema de Maven Central.

---

## 1. Visión y Motivación

Históricamente, los desarrolladores de Java han dependido de Maven y Gradle. Aunque son robustos, presentan limitaciones:
- **Tiempos de inicio lentos** (la JVM debe arrancar para ejecutar el gestor).
- **Curva de aprendizaje empinada** y configuración verbosa (XML complejo en Maven, Groovy/Kotlin intrincado en Gradle).
- **Resolución de dependencias pesada**.

**Jolt** es un binario único escrito en **Rust**, que se ejecuta instantáneamente y centraliza todo el ciclo de vida del desarrollo en Java: inicialización de proyectos, gestión de toolchains de JDK, resolución determinista de dependencias, compilación incremental, pruebas unitarias, ejecución con Hot Reload y empaquetado nativo/instaladores.

---

## ¿Qué características tiene Jolt?

### Velocidad extrema (Escrito en Rust)
Al igual que `uv` y `bun`, el CLI está construido en Rust. Esto permite:
- **Inicio en milisegundos** (cold start instantáneo).
- Resolución de dependencias y descargas concurrentes asíncronas con `tokio` y `reqwest`.
- Uso de **Hardlinks y Caché Global** (`~/.jolt/cache/`), evitando descargar o duplicar la misma dependencia JAR/POM en múltiples proyectos.

### Gestión de entorno y detección de JDKs
- **Detección Multinivel:** Escanea variables de entorno (`GRAALVM_HOME`, `JAVA_HOME`), ejecutables en `PATH` (`javac`, `java`) y directorios estándar del sistema operativo (Oracle GraalVM, Eclipse Temurin, Amazon Corretto, Azul Zulu, BellSoft Liberica, Microsoft OpenJDK, etc.).
- **Compatibilidad Semántica:** Si el proyecto define `java_version = "21"` y el sistema cuenta con un JDK compatible superior (ej. Java 25), Jolt lo aprovecha automáticamente sin descargas innecesarias.
- **Solo JDK del sistema:** Notifica claramente cuando falta un JDK compatible y requiere que el usuario lo instale o configure en el sistema.

### Manifiesto simplificado (`jolt.toml`) y lockfile determinista (`jolt.lock`)
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

```

---

## ¿Cómo está estructurado internamente?

### Arquitectura interna del proyecto (`src/`)

La base de código de Jolt está organizada de forma modular en 8 subsistemas especializados dentro de [`src/`](../src/):

| Módulo | Directorio | Responsabilidad Principal |
|---|---|---|
| **Core** | [`src/core/`](../src/core/) | Parser de `jolt.toml` ([`manifest.rs`](../src/core/manifest.rs)), gestión de `jolt.lock` ([`lockfile.rs`](../src/core/lockfile.rs)) y caché global direccionable ([`cache.rs`](../src/core/cache.rs)). |
| **CLI** | [`src/cli/`](../src/cli/) | Definición de argumentos, subcomandos y flags con `clap` ([`args.rs`](../src/cli/args.rs), [`mod.rs`](../src/cli/mod.rs)). |
| **Build & Run** | [`src/build/`](../src/build/) | Compilación con `javac` ([`compiler.rs`](../src/build/compiler.rs)), ejecución y Hot Reload ([`runner.rs`](../src/build/runner.rs)) y runner JUnit 5 ([`test_runner.rs`](../src/build/test_runner.rs)). |
| **Resolver** | [`src/resolver/`](../src/resolver/) | Cliente asíncrono Maven Central ([`maven_client.rs`](../src/resolver/maven_client.rs)) y parser XML de POMs ([`pom_parser.rs`](../src/resolver/pom_parser.rs)). |
| **Packaging** | [`src/packaging/`](../src/packaging/) | Fat-JAR ([`jar.rs`](../src/packaging/jar.rs)), jpackage ([`jpackage.rs`](../src/packaging/jpackage.rs)), NSIS ([`nsis.rs`](../src/packaging/nsis.rs)) y UPX ([`upx.rs`](../src/packaging/upx.rs)). |
| **Toolchain** | [`src/toolchain/`](../src/toolchain/) | Detección multinivel de JDKs instalados en el sistema ([`detector.rs`](../src/toolchain/detector.rs)). |
| **Scaffold** | [`src/scaffold/`](../src/scaffold/) | Generador de plantillas y workspaces ([`templates.rs`](../src/scaffold/templates.rs)) y configuración para VS Code / IntelliJ ([`ide_config.rs`](../src/scaffold/ide_config.rs)). |
| **Doctor** | [`src/doctor/`](../src/doctor/) | Diagnóstico y validación de herramientas instaladas y proyectos ([`diagnostics.rs`](../src/doctor/diagnostics.rs)). |

---

## ¿Cuál es el flujo de trabajo?

### Flujo de trabajo (UX / CLI)

- `jolt init` - Asistente interactivo para inicializar proyectos individuales o workspaces monorepo.
- `jolt add <groupId:artifactId>` - Busca en Maven Central y registra la versión en `jolt.toml`.
- `jolt install` - Resuelve dependencias y genera el `jolt.lock` determinista enlazando JARs a `.jolt/modules/`.
- `jolt run [--watch]` - Compila y ejecuta con soporte de Hot Reload instantáneo.
- `jolt test` - Ejecuta la suite de pruebas JUnit 5 de forma nativa.
- `jolt build [--standalone | --package | --installer]` - Empaqueta Fat-JARs e instaladores con jpackage.
- `jolt check` - Diagnóstica la salud del entorno, toolchains y dependencias.

La resolución de dependencias usa una caché local de metadatos para búsquedas,
versiones y POMs, además de la caché existente de JARs. La configuración
opcional `[repositories]` permite que un proyecto o workspace use un proxy o
repositorio Maven interno:

```toml
[repositories]
search_url = "https://nexus.example.com/service/rest/v1/search"
artifact_url = "https://nexus.example.com/repository/maven-public"
cache_ttl_seconds = 3600
```

`jolt add` busca el `jolt.toml` del directorio actual y autodetecta el módulo
de un workspace; `--member` sigue disponible para seleccionar un módulo de
forma explícita.

---

## ¿Qué está planeado para el futuro?

### Roadmap

- **v0.7.3 (Completada):**
  - Detección inteligente de JDKs locales y compatibilidad semántica.
  - Empaquetado en Windows con NSIS y UPX.
  - Workspaces monorepo y dependencias locales por ruta.
  - Reorganización modular del código en `src/` (8 subsistemas).
  - Todos los módulos funcionales (A-U) implementados y probados.
- **v0.8.0 (Especificado, pendiente de implementar):**
  - `jolt fmt`: Formateador nativo ultrarrápido de código Java. (Módulo Y)
  - `jolt doc`: Generador y servidor web embebido para Javadoc. (Módulo Z)
  - `jolt audit`: Auditoría de vulnerabilidades CVE (OSV.dev / Sonatype). (Módulo AA)

Las especificaciones detalladas de v0.8.0 viven en [`docs/specs-v7.md`](specs-v7.md) y los módulos implementados (A–U) están archivados en [`docs/archive/`](archive/).

---

## ¿Dónde está más documentación?

### Documentación relacionada

- [`docs/specs.md`](specs.md): índice maestro de todos los módulos (A–U) con enlaces a sus especificaciones archivadas y roadmap v0.8.0.
- [`docs/plan.md`](plan.md): plan de desarrollo actual, estado del proyecto y flujo de trabajo del CLI.
- [`docs/archive/`](archive/): especificaciones individuales de cada módulo implementado.
- [`openspec/`](../openspec/): especificaciones OpenSpec impulsadas por specs, contexto del proyecto y registro de cambios aplicados.
- [`graphify-out/`](../graphify-out/): grafo de conocimiento del proyecto (`GRAPH_REPORT.md`, `graph.json`, `graph.html`) con nodos clave, comunidades y relaciones entre archivos.

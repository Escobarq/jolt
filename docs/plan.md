# Jolt: ¿Cuál es el plan de desarrollo?

Estado actual: **Jolt v0.7.3** — gestor de paquetes, dependencias y herramientas para Java escrito en Rust, compatible con Maven Central.

Este documento describe el estado del proyecto, los módulos entregados, el roadmap vigente y cómo navegar la documentación técnica.

**Última actualización:** Noviembre 2026 - Todos los módulos funcionales (A-U) están implementados y archivados.

---

## 1. Estado Actual (v0.7.3)

Todos los módulos funcionales de Jolt están implementados y archivados:

| Area | Modulos | Estado |
|---|---|---|
| CLI y scaffolding | A | Implementado y archivado |
| Maven Central y resolver | B | Implementado y archivado |
| Cache global de JARs | C | Implementado y archivado |
| Toolchains (JDK) | D | Implementado y archivado |
| Motor de compilacion y ejecucion | E | Implementado y archivado |
| Fat-JAR y recursos estaticos | F, I | Implementados y archivados |
| Pruebas unitarias | G | Implementado y archivado |
| Modo Watch / Hot Reload | H | Implementado y archivado |
| Doctor / comprobacion de sistema | J | Implementado y archivado |
| Lockfile `jolt.lock` | K | Implementado y archivado |
| Plantillas `templates/` | L | Implementado y archivado |
| `jolt remove`, `jolt search` | M, N | Implementados y archivados |
| Dev-dependencias e init interactivo | O, S | Implementados y archivados |
| `jolt sync` e IDEs | P | Implementado y archivado |
| Empaquetado nativo (jpackage) | Q | Implementado y archivado |
| Paquetes y namespaces Java | R | Implementado y archivado |
| Workspaces multimodulo | T | Implementado y archivado |
| Instalador Windows (NSIS + UPX) | U | Implementado y archivado |

Los módulos **V, W y X** (Fase 6: detección de GraalVM, esquema `[graalvm]` y compilador nativo `native-image`) fueron especificados pero **no están en el CLI actual**. El empaquetado nativo disponible se realiza con `jpackage`. Historial: `openspec/changes/archive/2026-09-03-jdk-detection-and-graalvm/`.

---

## ¿Dónde está la documentación técnica?

### Documentación técnica por carpeta

- [`docs/specs.md`](specs.md) — Indice maestro de todos los modulos (A–U) con enlaces a sus especificaciones archivadas y roadmap v0.8.0.
- [`docs/specs-v7.md`](specs-v7.md) — Especificaciones tecnicas de v0.7.1 y roadmap detallado de v0.8.0 (modulos Y, Z, AA).
- [`docs/specs-v6.md`](specs-v6.md) — Especificacion historica de la Fase 6 (JDK detection, GraalVM, retirados).
- [`docs/design.md`](design.md) — Diseno arquitectonico y modulos internos de `src/`.
- [`docs/archive/`](archive/) — Especificaciones individuales de cada modulo implementado (A–U).
- [`openspec/`](../openspec/) — Especificaciones OpenSpec impulsadas por specs: contexto, estado de specs y registro de cambios aplicados.
- [`graphify-out/`](../graphify-out/) — Grafo de conocimiento del proyecto (nodos, comunidades, relaciones entre archivos): `GRAPH_REPORT.md`, `graph.json`, `graph.html`.

---

## ¿Qué está planeado para v0.8.0?

### Roadmap v0.8.0 (Especificado, no implementado)

Detalle completo en `docs/specs-v7.md`.

| Modulo | Capabilidad | Comando | Estado |
|---|---|---|---|
| **Y** | Formateador de codigo Java | `jolt fmt` | Especificado |
| **Z** | Generador y servidor local de Javadoc | `jolt doc` | Especificado |
| **AA** | Auditoria de seguridad y CVEs | `jolt audit` | Especificado |

Cualquiera de estos modulos puede tomarse como siguiente unidad de trabajo; sus especificaciones ya estan escritas y solo falta la implementacion en `src/`.

---

## ¿Cómo se usa Jolt en la práctica?

### Flujo de trabajo del usuario (CLI)

- `jolt init <nombre> [--template <t>] [--package <pkg>]` — inicializa proyecto o workspace monorepo.
- `jolt add <groupId:artifactId[:version]>` — busca en Maven Central y registra en `jolt.toml`.
- `jolt search <query>` — busca paquetes en Maven Central.
- `jolt remove <groupId:artifactId>` — elimina una dependencia.
- `jolt install [--locked] [--all | --member <m>]` — resuelve e instala dependencias, genera `jolt.lock`.
- `jolt run [--watch]` — compila y ejecuta (con Hot Reload opcional).
- `jolt test [--all | --member <m>]` — ejecuta pruebas JUnit 5.
- `jolt build [--standalone | --package | --installer]` — empaqueta JAR, imagen de app o instalador nativo.
- `jolt check` — diagnostica el entorno (Java, toolchains, cachés).

La configuracion opcional `[repositories]` en `jolt.toml` permite usar un proxy o repositorio Maven interno.

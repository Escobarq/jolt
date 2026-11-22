# Proposal

## Why

La instalación puede terminar con dependencias faltantes, un `jolt.lock` incompleto o artefactos no verificados y aun así informar éxito. Además, administrar o descargar un JDK dentro de Jolt oculta la configuración real del entorno y dificulta reproducir qué toolchain usa el proyecto.

## What Changes

- Hacer que `--locked` valide el manifiesto, versiones, clasificadores y SHA-256 de cada artefacto.
- Fallar `install` y `sync` cuando una dependencia requerida no pueda descargarse, guardarse o enlazarse.
- Escribir JARs y lockfiles de forma atómica para evitar archivos parciales bajo concurrencia.
- Añadir límites de tiempo a las peticiones HTTP de Maven.
- No descargar ni usar JDKs administrados por Jolt; detectar y usar únicamente un JDK instalado en el sistema.
- Evitar falsos fallos de `verify` causados por el límite fijo de 750 ms.
- Añadir pruebas de regresión para errores de instalación, integridad y concurrencia.

## Capabilities

### New Capabilities

- `dependency-installation-integrity`: instalación determinista, validación del lockfile y propagación de errores.
- `artifact-cache-integrity`: publicación atómica y validación de artefactos en la caché local.
- `toolchain-download-integrity`: política de no descargar ni usar toolchains administrados por Jolt.

### Modified Capabilities

- `toolchain-detection`: la selección debe limitarse a JDKs compatibles instalados en el sistema.

## Impact

Afecta `src/main.rs`, `src/core/cache.rs`, `src/core/lockfile.rs`, `src/resolver/maven_client.rs`, `src/toolchain/mod.rs`, `src/toolchain/detector.rs` y `src/build/runner.rs`. La descarga de JDK deja de ser un comportamiento soportado por la CLI. No requiere nuevas dependencias. Puede cambiar comandos que antes devolvían éxito después de una instalación incompleta: esos casos pasarán a devolver error.

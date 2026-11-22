# graalvm-config Specification

## Status

**Retirado** (`RETIRADO`). Esta especificación ya no forma parte del CLI actual de Jolt.

## Purpose (original)

Soportar la sección `[graalvm]` en `jolt.toml` y la compilación de binarios nativos AOT con GraalVM `native-image` (`jolt build --native`), incluyendo configuración de argumentos, clase principal, nombre de binario y archivos de reflexión/recursos.

## Motivo de Retiro

El CLI actual de Jolt empaqueta aplicaciones nativas exclusivamente con **jpackage** (`jolt build --package` / `jolt build --installer`, instaladores NSIS + UPX en Windows). La detección multinivel de JDKs y el control de descargas (`--download-jdk`) se mantienen, pero sin integración de `native-image`.

- Las pruebas del proyecto verifican explícitamente que `jolt init` **rechaza** el flag `--graalvm` y que los manifiestos generados **no contienen** `[graalvm]`.
- `docs/specs-v6.md` conserva la especificación histórica de la Fase 6.
- `docs/specs-v7.md` aclara que GraalVM Native Image ya no forma parte del CLI actual.

## Historial

- Especificado originalmente en: `docs/specs-v6.md` (Módulos V, W, X).
- Implementado como cambio OpenSpec: `openspec/changes/archive/2026-09-03-jdk-detection-and-graalvm/` (proposal, design, tasks; todas las tareas marcadas `[x]`).
- Especificación OpenSpec asociada: `openspec/specs/toolchain-detection/spec.md` (detección de JDKs, vigente).
- Retirado y documentado en: este archivo.

## Reemplazo Actual

Empaquetado nativo: [`docs/archive/modulo-q-jpackage-bundler.md`](../../docs/archive/modulo-q-jpackage-bundler.md) — `jolt build --package` / `--installer` con jpackage, opcionalmente `--upx` y `--add-to-path`.

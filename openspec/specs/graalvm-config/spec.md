# graalvm-config Specification

## Status

**Retirado** (`RETIRADO`). Esta especificación ya no forma parte del CLI actual de Jolt.

## Purpose

Documenta el retiro de la configuración `[graalvm]` y de la compilación nativa con GraalVM `native-image` del CLI actual de Jolt.

## Requirements

### Requirement: GraalVM Native Image permanece retirado
El CLI SHALL rechazar las opciones retiradas de GraalVM Native Image y SHALL usar `jpackage` para el empaquetado nativo soportado.

#### Scenario: Uso de una opción retirada
- **WHEN** se ejecuta `jolt init --graalvm` o `jolt build --native`
- **THEN** el CLI rechaza la opción y no genera configuración `[graalvm]`

## Motivo de Retiro

El CLI actual de Jolt empaqueta aplicaciones nativas exclusivamente con **jpackage** (`jolt build --package` / `jolt build --installer`, instaladores NSIS + UPX en Windows). La detección multinivel de JDKs del sistema se mantiene, sin integración de `native-image` ni administración de toolchains propios.

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

# security-audit Specification

## Status

Especificado (roadmap v0.8.0), **no implementado**.

## Purpose

Detectar vulnerabilidades de seguridad conocidas (CVEs) en las dependencias declaradas en `jolt.lock`, ofreciendo análisis preventivo antes de compilaciones o despliegues a producción.

## Requirements

### Requirement: Lectura del lockfile
El sistema SHALL leer `jolt.lock` extrayendo las coordenadas `groupId:artifactId:version` y sus checksums SHA-256.

### Requirement: Consulta de fuentes de vulnerabilidades
El sistema SHALL consultar en lote (batch query) la API pública de OSV.dev (`https://api.osv.dev/v1/querybatch`) en el ecosistema `Maven`, con consulta cruzada opcional en Sonatype OSS Index.

### Requirement: Evaluación de severidad
El sistema SHALL mapear el vector CVSS a categorías legibles:
- LOW (0.1 - 3.9)
- MEDIUM (4.0 - 6.9)
- HIGH (7.0 - 8.9)
- CRITICAL (9.0 - 10.0)

### Requirement: Salida y código de salida
El sistema SHALL mostrar una tabla formateada (ID del advisory, dependencia, versión vulnerable, severidad, versión mínima con parche) y retornar código de salida `1` si se encuentran vulnerabilidades con severidad >= la configurada.

## CLI

- `jolt audit` — audita todas las dependencias del proyecto
- `jolt audit --severity <nivel>` — filtra por severidad mínima (low, medium, high, critical)
- `jolt audit --json` — salida JSON para integración en CI/CD
- `jolt audit --fix` — sugiere versiones parcheadas seguras en `jolt.toml`

## Referencias

- Especificación detallada: `docs/specs-v7.md` (Módulo AA).
- Plan: `docs/plan.md` (Roadmap v0.8.0).
- Fuente de entrada: `jolt.lock` (ver `crate::core::JoltLock`, `crate::core::LockedPackage`).

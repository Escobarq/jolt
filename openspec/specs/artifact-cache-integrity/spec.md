# artifact-cache-integrity Specification

## Purpose

Garantiza que la caché local de artefactos solo publique archivos completos y verificables, incluso cuando varias operaciones de Jolt se ejecutan al mismo tiempo.

## Requirements

### Requirement: Publicación atómica de artefactos
La caché SHALL hacer visible un artefacto nuevo solo después de que su contenido completo haya sido escrito.

#### Scenario: Lectura concurrente durante una descarga
- **WHEN** una operación escribe un JAR mientras otra intenta usar la caché
- **THEN** la segunda operación observa el JAR anterior completo o el nuevo completo, nunca un archivo parcial

### Requirement: Integridad de artefactos cacheados
La caché SHALL conservar y permitir comprobar el SHA-256 del contenido descargado antes de considerarlo utilizable.

#### Scenario: Contenido cacheado corrupto
- **WHEN** el SHA-256 calculado de un JAR no coincide con el checksum esperado
- **THEN** Jolt rechaza el artefacto y lo reemplaza o devuelve un error antes de enlazarlo

### Requirement: Enlace solo de artefactos válidos
Jolt SHALL enlazar o copiar al proyecto únicamente artefactos que existan completos y hayan superado la validación de integridad aplicable.

#### Scenario: Archivo temporal en la caché
- **WHEN** la caché contiene un archivo temporal o una publicación incompleta
- **THEN** ese archivo no se usa como dependencia del proyecto

## Historial

- Implementado por `openspec/changes/archive/2026-11-22-harden-installation-integrity/`.

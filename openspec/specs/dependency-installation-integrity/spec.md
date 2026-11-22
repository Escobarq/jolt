# dependency-installation-integrity Specification

## Purpose

Garantiza que las instalaciones de dependencias Maven sean reproducibles y que los comandos de instalación no reporten éxito cuando falta un artefacto, el lockfile no coincide o la integridad del archivo no puede confirmarse.

## Requirements

### Requirement: Instalación bloqueada reproducible
Con `--locked`, el sistema SHALL exigir que el manifiesto y `jolt.lock` representen exactamente las mismas dependencias, versiones y clasificadores solicitados.

#### Scenario: Manifiesto incompatible con el lockfile
- **WHEN** el usuario ejecuta `jolt install --locked` después de cambiar una dependencia del manifiesto
- **THEN** el comando falla antes de instalar y explica que el lockfile debe regenerarse

### Requirement: Verificación de artefactos bloqueados
Con `--locked`, el sistema SHALL verificar el SHA-256 registrado de cada artefacto antes de enlazarlo al proyecto.

#### Scenario: Artefacto cacheado alterado
- **WHEN** un JAR cacheado no coincide con el checksum del lockfile
- **THEN** la instalación falla y no enlaza ese JAR al proyecto

### Requirement: Errores de instalación son fatales
El sistema SHALL devolver un error cuando una dependencia requerida no pueda resolverse, descargarse, guardarse o enlazarse.

#### Scenario: Dependencia transitiva no disponible
- **WHEN** Maven no puede descargar una dependencia transitiva requerida
- **THEN** `jolt install` y `jolt sync` terminan con error y no informan una instalación completa

### Requirement: Persistencia segura del lockfile
El sistema SHALL preservar el lockfile anterior si no puede completar y publicar el nuevo contenido.

#### Scenario: Fallo al guardar el lockfile
- **WHEN** el nuevo lockfile no puede escribirse completamente
- **THEN** el comando falla y el lockfile previo permanece legible e intacto

## Historial

- Implementado por `openspec/changes/archive/2026-11-22-harden-installation-integrity/`.

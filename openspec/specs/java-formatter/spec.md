# java-formatter Specification

## Status

Especificado (roadmap v0.8.0), **no implementado**.

## Purpose

Garantizar la consistencia estilística del código Java en proyectos individuales y monorepos, ejecutándose de forma instantánea sin la lentitud de herramientas tradicionales de Maven o Gradle.

## Requirements

### Requirement: Formateo de archivos Java
El sistema SHALL escanear recursivamente `src/main/java`, `src/test/java` (o rutas especificadas) y aplicar transformaciones de formato (indentación, espaciado, ordenamiento de imports) mediante un motor integrado en Rust.

#### Scenario: Formateo bajo demanda
- **WHEN** el usuario ejecuta `jolt fmt`
- **THEN** se formatean todos los archivos `.java` bajo `src/` in-place.

### Requirement: Modo de validación CI (`--check`)
El sistema SHALL validar el formato sin modificar archivos y retornar código de salida `0` si todo está bien formado, o `1` listando los archivos que requieren cambios.

#### Scenario: Validación en CI
- **WHEN** se ejecuta `jolt fmt --check` sobre un proyecto con archivos mal formateados
- **THEN** retorna código de salida `1` e imprime la lista de archivos con problemas.

### Requirement: Configuración en `jolt.toml`
El sistema SHALL leer la configuración de formato de la sección `[format]` de `jolt.toml`.

```toml
[format]
style = "google"          # "google" (2 espacios) o "aosp" (4 espacios)
max_width = 100
sort_imports = true
remove_unused_imports = true
```

#### Scenario: Configuración del formato
- **WHEN** el proyecto declara una sección `[format]` válida
- **THEN** `jolt fmt` aplica el estilo, ancho máximo y reglas de imports configurados

### Requirement: Diff de cambios
El sistema SHALL ofrecer `jolt fmt --diff` que muestre el diff unificado de los cambios necesarios sin escribirlos en disco.

#### Scenario: Consulta del diff
- **WHEN** se ejecuta `jolt fmt --diff` sobre archivos que requieren formato
- **THEN** el comando muestra un diff unificado y no modifica los archivos

## CLI

- `jolt fmt` — formatea todo `src/`
- `jolt fmt --check` — valida sin modificar
- `jolt fmt --diff` — muestra diff unificado
- `jolt fmt <ruta>` — formatea un archivo o directorio específico

## Referencias

- Especificación detallada: `docs/specs-v7.md` (Módulo Y).
- Plan: `docs/plan.md` (Roadmap v0.8.0).

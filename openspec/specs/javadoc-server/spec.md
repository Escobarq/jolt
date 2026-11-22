# javadoc-server Specification

## Status

Especificado (roadmap v0.8.0), **no implementado**.

## Purpose

Proporcionar una experiencia fluida de documentación para librerías y aplicaciones Java, generando HTML estándar de Javadoc e iniciando un servidor HTTP local para previsualización inmediata.

## Requirements

### Requirement: Generación de Javadoc
El sistema SHALL obtener el ejecutable `javadoc` a través de `ToolchainManager`, resolver el classpath automáticamente con `BuildEngine::build_classpath` y ejecutar `javadoc` depositando el resultado en `target/doc/`.

#### Scenario: Generación de documentación
- **WHEN** el usuario ejecuta `jolt doc`
- **THEN** se genera documentación Javadoc en `target/doc/` para los paquetes de `src/main/java`.

### Requirement: Servidor HTTP embebido
El sistema SHALL ofrecer un servidor web asíncrono ligero que sirva los archivos estáticos de `target/doc/` y notifique la URL activa en la terminal.

#### Scenario: Previsualización local
- **WHEN** se ejecuta `jolt doc --serve`
- **THEN** se inicia un servidor en `http://localhost:8080` (puerto configurable con `--port`) que sirve `target/doc/`.

### Requirement: Opciones de visibilidad
El sistema SHALL soportar `--open` (abrir en el navegador), `--private` (incluir miembros privados/protegidos) y salida a puerto personalizado.

## CLI

- `jolt doc` — genera Javadoc en `target/doc/`
- `jolt doc --open` — genera y abre en el navegador predeterminado
- `jolt doc --serve` — servidor embebido en `http://localhost:8080`
- `jolt doc --port <n>` — puerto personalizado
- `jolt doc --private` — incluye miembros privados y protegidos

## Referencias

- Especificación detallada: `docs/specs-v7.md` (Módulo Z).
- Plan: `docs/plan.md` (Roadmap v0.8.0).
- Componentes reutilizados: `crate::toolchain::ToolchainManager`, `crate::build::BuildEngine`.

# Jolt - Especificaciones Tecnicas Fase 3

Este documento detalla la arquitectura y diseno tecnico para las siguientes funcionalidades de la Fase 3:
1. **Generador de Lockfile (`jolt.lock`)**
2. **Plantillas de Inicializacion (`jolt init --template`)**
3. **Desinstalacion de Dependencias (`jolt remove`)**

---

## 1. Modulo K: Motor de Lockfile Determinista (`jolt.lock`) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-k-lockfile.md`](modulo-k-lockfile.md).

**Responsabilidad:** Proporcionar builds 100% reproducibles y deterministas para entornos de produccion y CI/CD, almacenando el arbol completo de dependencias resueltas y sus hashes criptograficos SHA-256.

### Tareas implementadas:
- [x] Estructura `JoltLock` y serializacion TOML en `src/lockfile.rs`.
- [x] Calculo de hashes SHA-256 en `src/cache.rs`.
- [x] Sincronizacion automatica en `add`, `install` y `remove`.
- [x] Flag `--locked` en `jolt install`.

---

## 2. Modulo L: Plantillas de Inicio de Proyectos (`jolt init --template`) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-l-templates.md`](modulo-l-templates.md).

**Responsabilidad:** Permitir al desarrollador inicializar proyectos preconfigurados para diferentes casos de uso mediante la bandera `--template` / `-t`.

### Tareas implementadas:
- [x] Flag `--template` / `-t` en `src/cli.rs`.
- [x] Plantillas soportadas en `src/scaffold.rs`:
  - `minimal`: Java estandar + JUnit 5.
  - `cli`: Picocli 4.7.6.
  - `javafx`: OpenJFX 21 + Launcher + CSS.
  - `swing`: Java Swing con Look and Feel FlatLaf Dark.
  - `web`: Javalin 6.1.3 + REST API.
  - `spring`: Spring Boot 3.2.3 Web REST API + Actuator.

---

## 3. Modulo M: Desinstalacion de Dependencias (`jolt remove`) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-m-remove-dependency.md`](modulo-m-remove-dependency.md).

**Responsabilidad:** Remover de forma segura librerias del proyecto, actualizando el manifiesto `jolt.toml`, regenerando el `jolt.lock` y limpiando los enlaces en `.jolt/modules/`.

### Tareas implementadas:
- [x] Mutacion in-place con `toml_edit` en `src/manifest.rs`.
- [x] Subcomando `Remove` (alias `rm`) en `src/cli.rs`.
- [x] Limpieza automatica de `.jar` en `.jolt/modules/`.
- [x] Sincronizacion del lockfile `jolt.lock`.

---

## 4. Modulo N: Buscador de Dependencias en Maven Central (`jolt search`) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-n-search.md`](modulo-n-search.md).

**Responsabilidad:** Consultar el API de Maven Central para descubrir librerias y dependencias por palabras clave, facilitando su instalacion con comandos preformateados.

### Tareas implementadas:
- [x] Consulta REST a `search.maven.org` en `src/maven.rs`.
- [x] Subcomando `Search` (alias `find`) con soporte para `--limit` en `src/cli.rs`.
- [x] Formateo de salida interactiva con comandos directos `jolt add`.


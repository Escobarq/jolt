# Archivo de Modulo J: Diagnostico de Entorno y Salud del Proyecto (`jolt check`)

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/checker.rs`](../../src/checker.rs): `SystemChecker` con deteccion de utilidades (`java`, `javac`, `jar`, `rustc`, `cargo`), estadisticas de almacenamiento en cache global y analisis del estado de dependencias locales.
  - [`src/cli.rs`](../../src/cli.rs): Subcomando `Check` para invocar `jolt check`.
  - [`src/main.rs`](../../src/main.rs): Integracion y despacho del comando de diagnostico.

---

## Resumen de Tareas Cumplidas

1. **Diagnostico Global del Entorno**:
   - Inspecciona la disponibilidad y versiones de `java`, `javac`, `jar`, `rustc` y `cargo`.
   - Reporta la ubicacion y el uso de espacio en disco de la cache global (`~/.jolt/cache/v1/`) y toolchains JDK aprovisionados.
2. **Diagnostico Contextual de Proyectos**:
   - Si no hay un proyecto activo, indica claramente que el entorno esta listo para crear uno con `jolt init`.
   - Si existe `jolt.toml`, valida la version de Java requerida, recuenta archivos en `src/main/java/`, `src/main/resources/` y `src/test/java/`, y comprueba una a una las dependencias sincronizadas en `.jolt/modules/` advirtiendo si hace falta ejecutar `jolt install`.

---

## Verificacion y Pruebas Realizadas

- Ejecucion fuera de un proyecto (diagnostico global puro).
- Ejecucion en `demo_app` (deteccion de dependencias y archivos de prueba).
- Ejecucion en `javafx_demo` (validacion de 4 modulos JavaFX con clasificadores de plataforma).

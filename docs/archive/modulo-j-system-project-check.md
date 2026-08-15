# Archivo de Módulo J: Diagnóstico de Entorno y Salud del Proyecto (`jolt check`)

- **Estado:** ✅ Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/checker.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/checker.rs): `SystemChecker` con detección de utilidades (`java`, `javac`, `jar`, `rustc`, `cargo`), estadísticas de almacenamiento en caché global y análisis del estado de dependencias locales.
  - [`src/cli.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/cli.rs): Subcomando `Check` para invocar `jolt check`.
  - [`src/main.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/main.rs): Integración y despacho del comando de diagnóstico.

---

## Resumen de Tareas Cumplidas

1. **Diagnóstico Global del Entorno**:
   - Inspecciona la disponibilidad y versiones de `java`, `javac`, `jar`, `rustc` y `cargo`.
   - Reporta la ubicación y el uso de espacio en disco de la caché global (`~/.jolt/cache/v1/`) y toolchains JDK aprovisionados.
2. **Diagnóstico Contextual de Proyectos**:
   - Si no hay un proyecto activo, indica claramente que el entorno está listo para crear uno con `jolt init`.
   - Si existe `jolt.toml`, valida la versión de Java requerida, recuenta archivos en `src/main/java/`, `src/main/resources/` y `src/test/java/`, y comprueba una a una las dependencias sincronizadas en `.jolt/modules/` advirtiendo si hace falta ejecutar `jolt install`.

---

## Verificación y Pruebas Realizadas

- Ejecución fuera de un proyecto (diagnóstico global puro).
- Ejecución en `demo_app` (detección de dependencias y archivos de prueba).
- Ejecución en `javafx_demo` (validación de 4 módulos JavaFX con clasificadores de plataforma).

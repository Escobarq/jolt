# Archivo de Modulo G: Motor de Pruebas Unitarias Integrado (`jolt test`)

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/engine.rs`](../../src/engine.rs): `compile_tests` y `run_tests` con ejecucion sobre JUnit Platform Console Standalone.
  - [`src/cli.rs`](../../src/cli.rs): Subcomando `Test`.
  - [`src/main.rs`](../../src/main.rs): Aprovisionamiento de `junit-platform-console-standalone-1.10.2.jar` a la cache global.

---

## Resumen de Tareas Cumplidas

1. **Aprovisionamiento Automatico de JUnit 5**:
   - Descarga bajo demanda del JUnit Platform Console Launcher oficial hacia `~/.jolt/cache/v1/`.
2. **Separacion de Classpath**:
   - Compilacion de `src/main/java/` hacia `target/classes/`.
   - Compilacion de `src/test/java/` hacia `target/test-classes/` con dependencias del proyecto y JUnit API.
3. **Ejecucion de Pruebas**:
   - Invocacion del Launcher con reporte de resultados en tiempo real.

---

## Verificacion y Pruebas Realizadas

- Suite de pruebas unitarias en `demo_app/src/test/java/CalculatorTest.java` ejecutada con exito en 192 milisegundos.

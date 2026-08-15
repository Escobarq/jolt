# Archivo de Módulo G: Motor de Pruebas Unitarias Integrado (`jolt test`)

- **Estado:** ✅ Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/engine.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/engine.rs): `BuildEngine::compile_tests` y `BuildEngine::run_tests` con separación de `src/main/java` y `src/test/java`.
  - [`src/cli.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/cli.rs): Subcomando `Test` para `jolt test`.
  - [`src/main.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/main.rs): Aprovisionamiento automático y caché del ejecutor oficial **JUnit Platform Console Standalone (1.10.2)**.

---

## Resumen de Tareas Cumplidas

1. **Aprovisionamiento Automático de JUnit 5**:
   - Descarga bajo demanda a `~/.jolt/cache/v1/` de `org.junit.platform:junit-platform-console-standalone:1.10.2` sin necesidad de configuraciones manuales de plugins.
2. **Compilación Aislada de Pruebas**:
   - `compile` ahora sólo compila el código de producción en `src/main/java/` a `target/classes/`.
   - `compile_tests` compila `src/test/java/` a `target/test-classes/` enlazando el código de producción, dependencias y la API de JUnit 5.
3. **Ejecución y Reporte en Consola**:
   - Invocación de JUnit Platform Console con subcomando `execute` y formato en árbol interactivo con tiempos de ejecución e indicadores visuales de éxito/fallo.

---

## Verificación y Pruebas Realizadas

- Suite de pruebas con JUnit Jupiter (`CalculatorTest.java`) ejecutada en `demo_app` en 192 ms:
```text
╷
├─ JUnit Jupiter ✔
│  └─ CalculatorTest ✔
│     ├─ testStringValidation() ✔
│     └─ testAddition() ✔
├─ JUnit Vintage ✔
└─ JUnit Platform Suite ✔

[ 2 tests successful ] [ 0 tests failed ]
```

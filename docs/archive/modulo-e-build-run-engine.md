# Archivo de Modulo E: Motor de Compilacion y Ejecucion (Build & Run Engine)

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/engine.rs`](../../src/engine.rs): Ensamblaje de classpath, compilador `javac`, runner `java` y empaquetador `jar`.
  - [`src/cli.rs`](../../src/cli.rs): Subcomandos `jolt build` y `jolt run`.
  - [`src/main.rs`](../../src/main.rs): Coordinacion de ejecucion con el manifiesto y toolchains.

---

## Resumen de Tareas Cumplidas

1. **Ensamblaje del Classpath**:
   - `BuildEngine::build_classpath`: Une clases compiladas en `target/classes` y todos los `.jar` de `.jolt/modules/` usando delimitadores de plataforma (`:` en Linux/macOS, `;` en Windows).
2. **Compilacion (`jolt build` / `jolt run`)**:
   - `BuildEngine::compile`: Invoca `javac` apuntando a `src/main/java/` y depositando bytecode en `target/classes/`.
3. **Ejecucion (`jolt run`)**:
   - `BuildEngine::run`: Ejecuta la clase principal (`Main`) con la JVM correcta.
4. **Empaquetado (`jolt build`)**:
   - `BuildEngine::build_jar`: Crea un JAR estandar con manifiesto de entrada en `target/<name>-<version>.jar`.

---

## Verificacion y Pruebas Realizadas

- Test unitario `test_collect_java_files` en `src/engine.rs`.
- Ejecucion en vivo con `demo_app`:
  - Compilacion de `Main.java` utilizando librerias externas de Gson.
  - Ejecucion con salida JSON formateada.
  - Empaquetado de `target/demo_app-0.1.0.jar`.
- Resultado: Ciclo completo de desarrollo en 47 milisegundos.

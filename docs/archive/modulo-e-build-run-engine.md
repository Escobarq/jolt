# Archivo de Módulo E: Motor de Compilación y Ejecución (Build & Run Engine)

- **Estado:** ✅ Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/engine.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/engine.rs): Motor de compilación, construcción de classpath, ejecución interactiva con `std::process::Command` y empaquetado de archivos `.jar`.
  - [`src/main.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/main.rs): Integración de los subcomandos `jolt run` y `jolt build`.

---

## Resumen de Tareas Cumplidas

1. **Constructor de Classpath Dinámico**:
   - `BuildEngine::build_classpath`: Escaneo automático de `.jolt/modules/*.jar` y clases locales respetando los separadores de sistema (`:` en Unix, `;` en Windows).
2. **Compilación al Vuelo (`jolt run` / `jolt build`)**:
   - `BuildEngine::compile`: Búsqueda recursiva de archivos `.java` en `src/` e invocación de `javac` dirigiendo los binarios compilados a `target/classes/`.
3. **Ejecución Interactiva**:
   - `BuildEngine::run`: Invocación de `java -cp <classpath> Main` con transmisión de flujos de entrada/salida estándar en tiempo real.
4. **Empaquetador JAR**:
   - `BuildEngine::build_jar`: Creación de archivos ejecutables autónomos en `target/<nombre>-<version>.jar` mediante la herramienta `jar`.

---

## Verificación y Pruebas Realizadas

- Test unitario de recolección de archivos Java en `src/engine.rs` (`test_collect_java_files`) ejecutado con `cargo test`.
- Prueba en vivo en `demo_app`:
  - Modificación de `Main.java` usando la librería `com.google.gson.Gson`.
  - `jolt run` compiló y ejecutó imprimiendo la salida JSON formateada por Gson.
  - `jolt build` empaquetó `target/demo_app-0.1.0.jar` exitosamente.

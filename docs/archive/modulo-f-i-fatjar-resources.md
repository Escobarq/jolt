# Archivo de Modulos F e I: Fat-JAR Bundler y Gestor de Recursos Estaticos

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/engine.rs`](../../src/engine.rs): `BuildEngine::copy_resources` y `BuildEngine::build_standalone_jar`.
  - [`src/cli.rs`](../../src/cli.rs): Flag `--standalone` / `-s` en `jolt build`.
  - [`src/main.rs`](../../src/main.rs): Despacho de empaquetado Fat-JAR autonomo.

---

## Resumen de Tareas Cumplidas

1. **Gestor de Recursos Estaticos (Modulo I)**:
   - Copia recursiva de archivos desde `src/main/resources/` hacia `target/classes/` antes de la compilacion y empaquetado.
2. **Empaquetador Fat-JAR / Uber-JAR (Modulo F)**:
   - Desempaqueta y combina todas las dependencias `.jar` de `.jolt/modules/` junto al bytecode del proyecto.
   - Filtra firmas de seguridad digitales (`META-INF/*.SF`, `*.DSA`, `*.RSA`) para evitar excepciones de seguridad en runtime.
   - Genera `target/<name>-<version>-standalone.jar` ejecutable directamente con `java -jar app.jar`.

---

## Verificacion y Pruebas Realizadas

- Creacion y ejecucion de un Fat-JAR autonomo de 7.8 MB para `javafx_demo` con modulos de controles, graficos y estilos CSS embebidos.
- Ejecucion exitosa de `java -jar target/javafx_demo-0.1.0-standalone.jar` en entorno Linux.

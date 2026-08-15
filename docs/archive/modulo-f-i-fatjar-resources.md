# Archivo de Módulos F e I: Fat-JAR Bundler y Gestor de Recursos Estáticos

- **Estado:** ✅ Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/engine.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/engine.rs): `BuildEngine::copy_resources` y `BuildEngine::build_standalone_jar` con soporte para descompresión ZIP y filtrado de firmas digitales `META-INF/*.SF`, `.DSA`, `.RSA`.
  - [`src/cli.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/cli.rs): Flag `--standalone` / `-s` en el comando `jolt build`.
  - [`src/scaffold.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/scaffold.rs): Creación automática de directorios `src/main/resources/` y `src/test/java/`.

---

## Resumen de Tareas Cumplidas

1. **Gestor de Recursos Estáticos (Módulo I)**:
   - Copia recursiva de archivos estáticos (`.properties`, `.fxml`, `.json`, `.css`, `.png`, etc.) desde `src/main/resources/` hacia `target/classes/` durante la compilación.
2. **Empaquetador Fat-JAR Autónomo (Módulo F)**:
   - Fusión de todas las clases de la aplicación con los `.jar` de dependencias de `.jolt/modules/` en un único archivo ejecutable: `target/<nombre>-<version>-standalone.jar`.
   - Generación de `META-INF/MANIFEST.MF` con `Main-Class`.
   - Remoción de firmas de seguridad de librerías para prevenir errores `SecurityException: SHA-256 digest error`.

---

## Verificación y Pruebas Realizadas

- Compilación y ejecución de `demo_app` con `com.google.code.gson:gson` y lectura del archivo de configuración `app.properties`.
- Ejecución autónoma verificada: `java -jar target/demo_app-0.1.0-standalone.jar`.

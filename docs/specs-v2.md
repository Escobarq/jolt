# Jolt ⚡️ - Especificaciones Técnicas Fase 2 (Advanced Features)

Este documento detalla la arquitectura y diseño técnico para las siguientes funcionalidades avanzadas de Jolt:
1. **Fat-JAR / Standalone Bundler (`jolt build --standalone`)**
2. **Motor de Pruebas Unitarias Integrado (`jolt test`)**
3. **Modo Watch / Hot Reload (`jolt run --watch`)**
4. **Gestor de Recursos Estáticos (`src/main/resources/`)**

---

## 1. Módulo F: Fat-JAR / Standalone Bundler (`jolt build --standalone`) `[COMPLETADO Y ARCHIVADO ✅]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-f-i-fatjar-resources.md`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/docs/archive/modulo-f-i-fatjar-resources.md).

**Responsabilidad:** Generar un único archivo `.jar` ejecutable que contenga todas las clases del proyecto y todas las dependencias desempaquetadas, listo para ejecutarse en cualquier máquina con `java -jar app.jar`.

### Tareas implementadas:
- [x] Extracción recursiva de dependencias ZIP y copiado de clases.
- [x] Filtrado de firmas de seguridad `META-INF/*.SF`, `*.DSA`, `*.RSA`.
- [x] Generación de `META-INF/MANIFEST.MF` con `Main-Class`.

---

## 2. Módulo G: Motor de Pruebas Unitarias Integrado (`jolt test`)
**Responsabilidad:** Ejecutar pruebas automatizadas en `src/test/java/` usando JUnit 5 de forma nativa sin plugins pesados.

### Diseño e Implementación:
- **Flujo de Ejecución:**
  1. Verificar si existen archivos de prueba en `src/test/java/` (ej. `*Test.java`).
  2. Descargar a la caché global `~/.jolt/cache/v1/` el runner oficial **JUnit Platform Console Standalone** (`org.junit.platform:junit-platform-console-standalone:1.10.2`) si no está presente.
  3. Compilar el código principal (`src/main/java/`) a `target/classes/`.
  4. Compilar los tests (`src/test/java/`) a `target/test-classes/` incluyendo en el classpath:
     - `target/classes/`
     - `.jolt/modules/*.jar`
     - El JAR de JUnit Platform Console Standalone.
  5. Invocar:
     ```bash
     java -jar junit-platform-console-standalone.jar \
       --class-path target/test-classes:target/classes:<dependencies> \
       --scan-class-path
     ```
  6. Transmitir el reporte de tests pasados/fallidos con colores y tiempos de ejecución.

---

## 3. Módulo H: Modo Watch / Hot Reload (`jolt run --watch`)
**Responsabilidad:** Recompilar y reiniciar la aplicación Java automáticamente cada vez que el desarrollador guarde cambios en su editor de código.

### Diseño e Implementación:
- **Crates requeridos:** `notify` (v6+), `tokio`.
- **Flujo de Ejecución:**
  1. Iniciar un observador recursivo sobre el directorio `src/` y el archivo `jolt.toml`.
  2. Iniciar el proceso de Java (`Command::new("java").spawn()`) almacenando el manejador del proceso (`Child`).
  3. Al recibir un evento de modificación en disco (`EventKind::Modify` / `Create` / `Remove`):
     - Terminar (`child.kill()`) el proceso anterior inmediatamente.
     - Limpiar pantalla (opcional) y mostrar: `⚡ Cambio detectado. Recompilando...`.
     - Ejecutar `BuildEngine::compile` y lanzar el nuevo proceso.

---

## 4. Módulo I: Gestor de Recursos Estáticos (`src/main/resources/`) `[COMPLETADO Y ARCHIVADO ✅]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-f-i-fatjar-resources.md`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/docs/archive/modulo-f-i-fatjar-resources.md).

**Responsabilidad:** Copiar automáticamente recursos estáticos al classpath para aplicaciones con interfaces gráficas (JavaFX con archivos `.fxml`), servidores web (Spring Boot / Micronaut / Javalin con `.properties`, `.yaml`), y assets de juegos o utilidades.

### Tareas implementadas:
- [x] Copia recursiva de `src/main/resources/` y `src/resources/` a `target/classes/`.
- [x] Generación de carpeta en `jolt init`.

---

## Plan de Ejecución Propuesto:
Podemos implementar estas funcionalidades en el siguiente orden:
1. **Fase 2.1:** Gestor de Recursos (`src/main/resources/`) y Fat-JAR Bundler (`jolt build --standalone`).
2. **Fase 2.2:** Soporte para Pruebas Unitarias (`jolt test`).
3. **Fase 2.3:** Modo Watch (`jolt run --watch`).

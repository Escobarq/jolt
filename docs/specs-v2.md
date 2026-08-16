# Jolt - Especificaciones Tecnicas Fase 2 (Advanced Features)

Este documento detalla la arquitectura y diseno tecnico para las siguientes funcionalidades avanzadas de Jolt:
1. **Fat-JAR / Standalone Bundler (`jolt build --standalone`)**
2. **Motor de Pruebas Unitarias Integrado (`jolt test`)**
3. **Modo Watch / Hot Reload (`jolt run --watch`)**
4. **Gestor de Recursos Estaticos (`src/main/resources/`)**
5. **Diagnostico de Entorno y Proyecto (`jolt check`)**

---

## 1. Modulo F: Fat-JAR / Standalone Bundler (`jolt build --standalone`) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-f-i-fatjar-resources.md`](archive/modulo-f-i-fatjar-resources.md).

**Responsabilidad:** Generar un unico archivo `.jar` ejecutable que contenga todas las clases del proyecto y todas las dependencias desempaquetadas, listo para ejecutarse en cualquier maquina con `java -jar app.jar`.

### Tareas implementadas:
- [x] Extraccion recursiva de dependencias ZIP y copiado de clases.
- [x] Filtrado de firmas de seguridad `META-INF/*.SF`, `*.DSA`, `*.RSA`.
- [x] Generacion de `META-INF/MANIFEST.MF` con `Main-Class`.

---

## 2. Modulo G: Motor de Pruebas Unitarias Integrado (`jolt test`) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-g-unit-testing.md`](archive/modulo-g-unit-testing.md).

**Responsabilidad:** Ejecutar pruebas automatizadas en `src/test/java/` usando JUnit 5 de forma nativa sin plugins pesados.

### Tareas implementadas:
- [x] Subcomando `jolt test` en `src/cli.rs`.
- [x] Aprovisionamiento y almacenamiento en cache de `junit-platform-console-standalone-1.10.2.jar`.
- [x] Separacion de classpath de produccion (`src/main/java/` a `target/classes/`) y pruebas (`src/test/java/` a `target/test-classes/`).
- [x] Invocacion de JUnit Platform Console con reporte estructurado.

---

## 3. Modulo H: Modo Watch / Hot Reload (`jolt run --watch`) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-h-watch-mode.md`](archive/modulo-h-watch-mode.md).

**Responsabilidad:** Recompilar y reiniciar la aplicacion Java automaticamente cada vez que el desarrollador guarde cambios en su editor de codigo.

### Tareas implementadas:
- [x] Flag `--watch` / `-w` en `jolt run`.
- [x] Observador de eventos con `notify` sobre `src/` y `jolt.toml`.
- [x] Debounce de 300ms y ciclo de vida de procesos con Hot Reload instantaneo.
- [x] Manejo resiliente ante errores de compilacion durante la edicion.

---

## 4. Modulo I: Gestor de Recursos Estaticos (`src/main/resources/`) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-f-i-fatjar-resources.md`](archive/modulo-f-i-fatjar-resources.md).

**Responsabilidad:** Copiar automaticamente recursos estaticos al classpath para aplicaciones con interfaces graficas (JavaFX con archivos `.fxml`), servidores web (Spring Boot / Micronaut / Javalin con `.properties`, `.yaml`), y assets de juegos o utilidades.

### Tareas implementadas:
- [x] Copia recursiva de `src/main/resources/` y `src/resources/` a `target/classes/`.
- [x] Generacion de carpeta en `jolt init`.

---

## 5. Modulo J: Diagnostico de Entorno y Proyecto (`jolt check`) `[COMPLETADO Y ARCHIVADO]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-j-system-project-check.md`](archive/modulo-j-system-project-check.md).

**Responsabilidad:** Diagnosticar la disponibilidad de herramientas del sistema (`java`, `javac`, `jar`, `rustc`, `cargo`), tamano de cache global y validar la salud e integridad de dependencias de proyectos locales.

### Tareas implementadas:
- [x] Subcomando `jolt check` en `src/cli.rs`.
- [x] Motor de inspeccion del sistema `src/checker.rs`.
- [x] Validacion contextual fuera y dentro de proyectos Jolt con consejos de sincronizacion.

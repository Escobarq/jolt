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

## 2. Módulo G: Motor de Pruebas Unitarias Integrado (`jolt test`) `[COMPLETADO Y ARCHIVADO ✅]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-g-unit-testing.md`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/docs/archive/modulo-g-unit-testing.md).

**Responsabilidad:** Ejecutar pruebas automatizadas en `src/test/java/` usando JUnit 5 de forma nativa sin plugins pesados.

### Tareas implementadas:
- [x] Subcomando `jolt test` en `src/cli.rs`.
- [x] Aprovisionamiento y almacenamiento en caché de `junit-platform-console-standalone-1.10.2.jar`.
- [x] Separación de classpath de producción (`src/main/java/` a `target/classes/`) y pruebas (`src/test/java/` a `target/test-classes/`).
- [x] Invocación de JUnit Platform Console con reporte estructurado.

---

## 3. Módulo H: Modo Watch / Hot Reload (`jolt run --watch`) `[COMPLETADO Y ARCHIVADO ✅]`
> **Estado:** Implementado y archivado en [`docs/archive/modulo-h-watch-mode.md`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/docs/archive/modulo-h-watch-mode.md).

**Responsabilidad:** Recompilar y reiniciar la aplicación Java automáticamente cada vez que el desarrollador guarde cambios en su editor de código.

### Tareas implementadas:
- [x] Flag `--watch` / `-w` en `jolt run`.
- [x] Observador de eventos con `notify` sobre `src/` y `jolt.toml`.
- [x] Debounce de 300ms y ciclo de vida de procesos con Hot Reload instantáneo.
- [x] Manejo resiliente ante errores de compilación durante la edición.

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

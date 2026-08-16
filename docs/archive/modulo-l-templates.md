# Archivo de Modulo L: Plantillas de Inicializacion (`jolt init --template`)

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/scaffold.rs`](../../src/scaffold.rs): Generador de proyectos con plantillas `minimal`, `cli`, `javafx`, `web`, `spring`.
  - [`src/cli.rs`](../../src/cli.rs): Flags `--template` / `-t` y `--list-templates` / `-l` en `jolt init`.
  - [`src/main.rs`](../../src/main.rs): Despacho de templates y listado interactivo.

---

## Resumen de Tareas Cumplidas

1. **Plantilla `minimal`:** Proyecto estandar Java con JUnit 5 configurado.
2. **Plantilla `cli`:** Preconfigurado con `info.picocli:picocli` para desarrollo rapido de utilidades de terminal.
3. **Plantilla `javafx`:** Preconfigurado con OpenJFX, launcher y estilos CSS.
4. **Plantilla `web`:** Preconfigurado con `io.javalin:javalin` para microservicios y APIs REST.
5. **Plantilla `spring`:** Preconfigurado con Spring Boot 3.2.3 Web, REST Controller y `application.properties`.
6. **Listado de Plantillas (`jolt init --list-templates`):** Despliega la descripcion de cada plantilla soportada.

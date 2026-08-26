# Jolt - Especificaciones Tecnicas Fase 5

Este documento detalla la arquitectura y diseño técnico para las funcionalidades de la Fase 5:
1. **Paquetes Java y Namespaces Estructurados (`package org.equipo.proyecto;`)**
2. **Asistente Interactivo de Inicialización (`jolt init`) para Monorepos y Proyectos Individuales**
3. **Workspaces Multimódulo (Monorepos estilo Cargo / Bun / Maven Modules)**
4. **Dependencias Locales Inter-Módulo (`path dependencies`) y Aislamiento de Librerías**

---

## 1. Modulo R: Paquetes Java y Namespaces (`src/scaffold.rs`, `src/manifest.rs`)

**Responsabilidad:** Generar estructuras de directorios reales según la convención de paquetes de Java y registrar namespaces canónicos.

### Capacidades:
- Creación de rutas de paquetes anidadas: `src/main/java/org/equipo/proyecto/Main.java` y `src/test/java/org/equipo/proyecto/MainTest.java`.
- Inyección dinámica de cabeceras `package org.equipo.proyecto;` en todas las plantillas Java (`minimal`, `cli`, `javafx`, `swing`, `web`, `spring`).
- Campos `package = "org.equipo.proyecto"`, `group_id = "org.equipo"` y `main_class = "org.equipo.proyecto.Main"` en `jolt.toml`.

---

## 2. Modulo S: Asistente Interactivo y Detección de Contexto (`jolt init`)

**Responsabilidad:** Proveer una experiencia de inicialización interactiva guiada.

### Capacidades:
- Menú interactivo con flechas para seleccionar entre **Proyecto Individual** y **Workspace Monorepo**.
- Solicitud interactiva del namespace / paquete Java con sugerencias inteligentes.
- Detección automática de workspaces padre para registrar nuevos submódulos en `[workspace].members`.

---

## 3. Modulo T: Workspaces Multimódulo y Dependencias Locales (`[workspace]`, `src/engine.rs`, `src/main.rs`)

**Responsabilidad:** Permitir que múltiples proyectos independientes (ej. JavaFX, Spring Boot, Core) coexistan en un mismo repositorio con dependencias aisladas y compartidas.

### Capacidades:
- **Estructura raíz con tabla `[workspace]`**:
  ```toml
  [workspace]
  members = ["core", "desktop_app", "web_api"]
  ```
- **Dependencias locales inter-módulos**:
  ```toml
  [dependencies]
  "core" = { path = "../core" }
  ```
- **Resolución de Classpath y Compilación Automática**:
  - `BuildEngine::compile` compila automáticamente módulos dependientes referenciados por `path`.
  - `BuildEngine::build_standalone_jar` y `package_native_app` integran las clases y recursos de los módulos locales referenciados.
- **Comandos conscientes de Workspaces**:
  - `jolt install` / `jolt sync`: instala/sincroniza todos los submódulos.
  - `jolt run -p <modulo>`: ejecuta un submódulo específico.
  - `jolt build --all` / `jolt build -p <modulo>`.
  - `jolt test --all` / `jolt test -p <modulo>`.
  - `jolt package -p <modulo>`: empaqueta un binario nativo de cualquier submódulo.

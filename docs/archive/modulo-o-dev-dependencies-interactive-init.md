# Archivo de Modulo O: Gestion de Dev-Dependencies, Menu Interactivo y Soporte IDE (v0.2.0)

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-18
- **Archivos Entregables:**
  - [`src/manifest.rs`](../../src/manifest.rs): Soporte de escritura y remocion para `[dev-dependencies]` y `[dependencies]`.
  - [`src/cache.rs`](../../src/cache.rs): Metodo `link_to_project_dir_with_classifier` para enlazar a subdirectorios `modules/` o `dev-modules/`.
  - [`src/engine.rs`](../../src/engine.rs): Separacion estricta de classpath de produccion (`.jolt/modules/`) y classpath de testing (`build_test_classpath`). Aislamiento de Fat-JAR (`build_standalone_jar`).
  - [`src/scaffold.rs`](../../src/scaffold.rs): Menu interactivo terminal con `dialoguer` para seleccion de plantilla y nombre, mas generacion automatica de `.vscode/settings.json`.
  - [`src/cli.rs`](../../src/cli.rs): Flag `--dev` (`-D`) en `jolt add` y actualizacion de version a `0.2.0`.
  - [`src/checker.rs`](../../src/checker.rs): Diagnostico diferenciado de dependencias de produccion y dependencias de desarrollo.
  - [`src/main.rs`](../../src/main.rs): Enrutamiento para instalacion, remocion y adicion con soporte de ambitos de dependencias.

---

## Resumen de Tareas Cumplidas

1. **Soporte Completo de `[dev-dependencies]`**:
   - `jolt install` sincroniza y descarga tanto `dependencies` (en `.jolt/modules/`) como `dev-dependencies` (en `.jolt/dev-modules/`).
   - `jolt add <dep> --dev` (`-D`) permite anadir librerias de testing directamente a `[dev-dependencies]`.
   - `jolt remove <dep>` remueve del manifiesto y limpia los archivos `.jar` de ambos directorios.
2. **Aislamiento y Autocompletado**:
   - `jolt build --standalone` solo incluye dependencias de produccion en el Fat-JAR.
   - Generacion de `.vscode/settings.json` con `java.project.referencedLibraries` apuntando a `modules` y `dev-modules` para reconocimiento instantaneo en editores (VS Code, Cursor, Eclipse LSP).
3. **Menu Interactivo en `jolt init`**:
   - Selector interactivo con flechas y prompt guiado utilizando el crate `dialoguer`.

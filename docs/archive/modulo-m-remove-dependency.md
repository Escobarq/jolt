# Archivo de Modulo M: Desinstalacion de Dependencias (`jolt remove`)

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/manifest.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/manifest.rs): `remove_dependency_from_file` con mutacion in-place via `toml_edit`.
  - [`src/cli.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/cli.rs): Subcomando `Remove` (alias `rm`).
  - [`src/main.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/main.rs): Eliminacion de `.jar` en `.jolt/modules/` y sincronizacion automatica de `jolt.lock`.

---

## Resumen de Tareas Cumplidas

1. **Remocion de Manifiesto:** Elimina claves de `dependencies` y `dev-dependencies` preservando formato y comentarios del archivo.
2. **Limpieza del Sistema de Archivos:** Remueve los enlaces hardlinks correspondientes de `.jolt/modules/`.
3. **Sincronizacion de Lockfile:** Actualiza `jolt.lock` de forma automatica.

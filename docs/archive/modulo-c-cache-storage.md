# Archivo de Módulo C: Gestor de Caché Global (Storage Engine)

- **Estado:** ✅ Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/cache.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/cache.rs): Motor de almacenamiento persistente `CacheManager`, hashing SHA-256 (`sha2`, `hex`), y creador de *hardlinks*.
  - [`src/main.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/main.rs): Integración del subcomando `jolt install` e instalación automática al hacer `jolt add`.

---

## Resumen de Tareas Cumplidas

1. **Directorio de Caché Global (`~/.jolt/cache/v1/`)**:
   - `CacheManager::get_jar_path` y `CacheManager::has_jar`: Estructura jerárquica por group/artifact/version.
2. **Descarga e Integridad SHA-256**:
   - `CacheManager::save_jar`: Almacenamiento seguro tras verificación de hash.
3. **Enlaces Duros (Hardlinks)**:
   - `CacheManager::link_to_project`: Vinculación instantánea hacia `.jolt/modules/` compartiendo inodos para ocupar 0 bytes duplicados en disco (con fallback a copia si hay múltiples sistemas de archivos).
4. **Comando `jolt install`**:
   - Lectura de `jolt.toml` y sincronización masiva de dependencias.

---

## Verificación y Pruebas Realizadas

- Test unitario de guardado y enlazado en `src/cache.rs` (`test_cache_save_and_link`) ejecutado con `cargo test`.
- Pruebas en el proyecto `demo_app`:
  - `jolt install` descargó y vinculó dependencias correctamente.
  - Verificación de conteo de inodos (`hard link count = 2`).
  - Cache hits instantáneos verificados en `jolt add`.

# Archivo de Modulo C: Gestor de Cache Global (Storage Engine)

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/cache.rs`](../../src/cache.rs): Gestor de almacenamiento con Content-Addressable Storage, calculo SHA-256 e integracion con hardlinks.
  - [`src/main.rs`](../../src/main.rs): Subcomando `jolt install` y flujo de almacenamiento en `jolt add`.

---

## Resumen de Tareas Cumplidas

1. **Jerarquia de Directorios Global**:
   - Inicializacion automatica de `~/.jolt/cache/v1/jars/` con aislamiento por version.
2. **Almacenamiento e Integridad SHA-256**:
   - `CacheManager::save_jar`: Escritura segura calculando sumas de verificacion.
3. **Hardlinks a Nivel de Sistema de Archivos**:
   - `CacheManager::link_to_project`: Creacion de enlaces duros en `.jolt/modules/` para zero-copy duplication.
4. **Comando `jolt install`**:
   - Lectura de `jolt.toml` y sincronizacion en masa de dependencias.

---

## Verificacion y Pruebas Realizadas

- Test unitario `test_cache_save_and_link` en `src/cache.rs` con validacion de inodos identicos (`stat`).
- Verificacion en vivo con `demo_app`:
  - `jolt install` creo correctamente los archivos en `.jolt/modules/`.
- Resultado: Deduplicacion efectiva sin duplicar espacio en disco.

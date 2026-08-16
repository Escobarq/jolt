# Archivo de Modulo H: Modo Observador / Hot Reload (`jolt run --watch`)

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/engine.rs`](../../src/engine.rs): `run_watch` y `spawn_process` con integracion del crate `notify`.
  - [`src/cli.rs`](../../src/cli.rs): Flag `--watch` / `-w` en `jolt run`.
  - [`src/main.rs`](../../src/main.rs): Despacho del subcomando de observador.

---

## Resumen de Tareas Cumplidas

1. **Observacion Recursiva del Sistema de Archivos**:
   - Monitoreo continuo sobre `src/` y `jolt.toml`.
2. **Debounce de Eventos (300 ms)**:
   - Evita multiples compilaciones simultaneas al guardar archivos en editores de codigo.
3. **Ciclo de Vida de Procesos y Hot Reload**:
   - Detencion controlada del subproceso Java anterior, recompilacion instantanea y relanzamiento automatico.

---

## Verificacion y Pruebas Realizadas

- Ejecucion de `jolt run --watch` en `demo_app` con modificacion dinamica de codigo fuente Java y reinicio automatico.

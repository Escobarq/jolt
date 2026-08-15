# Archivo de Módulo H: Modo Watch / Hot Reload (`jolt run --watch`)

- **Estado:** ✅ Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/engine.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/engine.rs): `BuildEngine::spawn_process` y `BuildEngine::run_watch` usando `notify` y canales `mpsc` con debounce de 300ms.
  - [`src/cli.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/cli.rs): Flag `--watch` / `-w` en `jolt run`.
  - [`src/main.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/main.rs): Coordinación y despacho del modo observador.

---

## Resumen de Tareas Cumplidas

1. **Observador de Archivos en Tiempo Real**:
   - Monitoreo recursivo de eventos de creación, modificación y eliminación en `src/` y `jolt.toml`.
2. **Ciclo de Vida de Procesos y Hot Reload**:
   - Manejo de procesos concurrentes: ante cambios detectados, se termina el proceso Java previo, se recompila el código modificado y se relanza la aplicación de forma instantánea.
   - Manejo resiliente de errores de compilación: si el usuario introduce un error de sintaxis mientras escribe, Jolt muestra el error con claridad y permanece en escucha esperando la corrección.

---

## Verificación y Pruebas Realizadas

- Prueba en vivo en `demo_app`:
```text
$ jolt run --watch
👀 Modo Watch activado. Observando cambios en 'src/' y 'jolt.toml'...
Salida JSON (Fat-JAR + Recursos): {"mode":"standalone-fat-jar","version":"0.1.0","tool":"Jolt"}

⚡ Cambio detectado en archivos. Recompilando...
🚀 Reiniciando aplicación...
🔥 ¡HOT RELOAD FUNCIONANDO AL INSTANTE!: {"mode":"standalone-fat-jar","version":"0.1.0","tool":"Jolt"}
```

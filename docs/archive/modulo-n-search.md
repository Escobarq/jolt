# Archivo de Modulo N: Buscador de Dependencias en Maven Central (`jolt search`)

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/maven.rs`](../../src/maven.rs): Metodo `search_packages` que consulta `search.maven.org` y extrae coordenadas, versiones, tipos y metadatos.
  - [`src/cli.rs`](../../src/cli.rs): Subcomando `Search` (alias `find`) con flags `--limit` / `-l`.
  - [`src/main.rs`](../../src/main.rs): Despacho con formateo de resultados y sugerencias de comandos directos `jolt add`.

---

## Resumen de Tareas Cumplidas

1. **Busqueda de Texto Libre**:
   - Permite encontrar librerias en Maven Central sin conocer previamente el `groupId` exacto o la version.
2. **Formateo de Comandos Listos para Usar**:
   - Genera automaticamente la linea exacta `jolt add <groupId:artifactId:version>` para copiar y pegar.
3. **Paginacion y Limites**:
   - Soporta `--limit <numero>` para controlar la cantidad de resultados devueltos (por defecto: 10).

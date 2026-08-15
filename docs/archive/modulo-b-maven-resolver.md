# Archivo de Módulo B: Cliente de Maven Central y Resolver de Dependencias

- **Estado:** ✅ Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/maven.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/maven.rs): Cliente HTTP asíncrono con `reqwest`, parser XML con `quick-xml`, y constructor de árboles de dependencias.
  - [`src/main.rs`](file:///home/juandavidescobarquezada/Escritorio/project/jolt/src/main.rs): Integración en el comando `jolt add` con resolución en tiempo real.

---

## Resumen de Tareas Cumplidas

1. **Búsqueda en Maven Central (`search.maven.org`)**:
   - `MavenClient::fetch_latest_version`: Consulta dinámica del último artefacto estable en caso de no especificarse la versión.
2. **Descarga de Metadatos (`repo1.maven.org`)**:
   - `MavenClient::fetch_pom`: Descarga de especificaciones POM vía HTTP.
3. **Parseo XML Ultrarrápido (`quick-xml`)**:
   - `MavenClient::parse_pom_dependencies`: Extracción de tags `<dependency>` sin requerir la JVM.
4. **Árbol de Dependencias**:
   - `MavenClient::fetch_dependency_tree`: Construcción y visualización jerárquica de dependencias transitivas en consola.

---

## Verificación y Pruebas Realizadas

- Test unitario de parseo POM en `src/maven.rs` (`test_parse_pom_xml`) ejecutado con `cargo test`.
- Pruebas en vivo con dependencias reales de Maven Central:
  - `jolt add com.google.code.gson:gson` -> Detectó v2.14.0 y dependencias transitivas.
  - `jolt add org.slf4j:slf4j-api` -> Detectó v2.1.0-alpha1.
- Resultado: 100% funcional y determinista.

# Archivo de Modulo B: Cliente de Maven Central y Resolver de Dependencias

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/maven.rs`](../../src/maven.rs): Cliente HTTP asincrono con `reqwest`, parser XML con `quick-xml`, y constructor de arboles de dependencias.
  - [`src/main.rs`](../../src/main.rs): Integracion en el comando `jolt add` con resolucion en tiempo real.

---

## Resumen de Tareas Cumplidas

1. **Busqueda en Maven Central (`search.maven.org`)**:
   - `MavenClient::fetch_latest_version`: Consulta dinamica del ultimo artefacto estable en caso de no especificarse la version.
2. **Descarga de Metadatos (`repo1.maven.org`)**:
   - `MavenClient::fetch_pom`: Descarga de especificaciones POM via HTTP.
3. **Parseo XML Ultrarrapido (`quick-xml`)**:
   - `MavenClient::parse_pom_dependencies`: Extraccion de tags `<dependency>` sin requerir la JVM.
4. **Arbol de Dependencias**:
   - `MavenClient::fetch_dependency_tree`: Construccion y visualizacion jerarquica de dependencias transitivas en consola.

---

## Verificacion y Pruebas Realizadas

- Test unitario de parseo POM en `src/maven.rs` (`test_parse_pom_xml`) ejecutado con `cargo test`.
- Pruebas en vivo con dependencias reales de Maven Central:
  - `jolt add com.google.code.gson:gson` -> Detecto v2.14.0 y dependencias transitivas.
  - `jolt add org.slf4j:slf4j-api` -> Detecto v2.1.0-alpha1.
- Resultado: 100% funcional y determinista.

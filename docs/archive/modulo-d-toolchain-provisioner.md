# Archivo de Modulo D: Aprovisionador de Toolchains (Gestion de JDK)

- **Estado:** Completado y Verificado
- **Fecha de Archivo:** 2026-08-15
- **Archivos Entregables:**
  - [`src/toolchain.rs`](../../src/toolchain.rs): Deteccion de JDK local del sistema y cliente de descarga Adoptium / Temurin API.
  - [`src/main.rs`](../../src/main.rs): Resolucion automatica de Toolchain basada en el campo `java_version` de `jolt.toml`.

---

## Resumen de Tareas Cumplidas

1. **Deteccion de JDK del Sistema**:
   - `ToolchainManager::find_system_jdk`: Inspecciona el `PATH` para detectar ejecutables existentes de `javac` y `java` y parsear la version mayor.
2. **Cliente API de Eclipse Adoptium**:
   - `ToolchainManager::download_and_extract_jdk`: Descarga el release LTS correspondiente segun la arquitectura del procesador y sistema operativo.
3. **Descompresion en Tarball (.tar.gz)**:
   - Desempaquetado automatico hacia `~/.jolt/toolchains/jdk-<version>/`.

---

## Verificacion y Pruebas Realizadas

- Test unitario `test_find_system_jdk` en `src/toolchain.rs` comprobando compatibilidad con OpenJDK 21.
- Resultado: Soporte transparente para toolchains locales y provisionadas.

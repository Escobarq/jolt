## 1. Soporte de configuración GraalVM en jolt.toml

- [x] 1.1 Definir la estructura `GraalVmConfig` en `src/manifest.rs` y enlazarla a `JoltManifest` y `PackageConfig`, verificando con tests unitarios en `manifest.rs`.
- [x] 1.2 Añadir soporte para deserializar argumentos (`args`), clase principal (`main_class`), nombre de binario (`name`), y rutas de reflexión/recursos, verificando con pruebas de parsing TOML.

## 2. Detección inteligente de JDKs locales en ToolchainManager

- [x] 2.1 Implementar el parser semántico de versiones de Java (`major: u32`) y detección de vendor (GraalVM, Temurin, Corretto, etc.) en `src/toolchain.rs`, verificado con tests unitarios.
- [x] 2.2 Implementar el escaneo multinivel en `src/toolchain.rs`: inspeccionar `GRAALVM_HOME`, `JAVA_HOME`, ejecutables en `PATH` y directorios estándar del sistema operativo (Windows, Linux, macOS), identificando ejecutables (`java`, `javac`, `jar`, `jpackage`, `native-image`).
- [x] 2.3 Implementar la lógica de compatibilidad (`>= requested_version` o coincidencia exacta) y prevenir la descarga silenciosa de Temurin, emitiendo un diagnóstico claro con instrucciones al usuario si no se encuentra un JDK adecuado.

## 3. Integración de GraalVM Native Image y comandos CLI

- [x] 3.1 Añadir la opción `--native` en el comando de compilación (`jolt build --native`) en `src/cli.rs`.
- [x] 3.2 Implementar en `src/engine.rs` la invocación de `native-image` consumiendo los parámetros de `GraalVmConfig` desde `jolt.toml` cuando se active la compilación nativa.
- [x] 3.3 Integrar la resolución del nuevo ToolchainManager en `src/main.rs` en los comandos `build`, `run`, `test` y `check`.

## 4. Verificación y pruebas de integración

- [x] 4.1 Ejecutar la suite completa de pruebas unitarias (`cargo test`) y verificar que todos los tests pasen exitosamente.
- [x] 4.2 Probar la detección en el entorno real del usuario (verificando que detecte Oracle GraalVM 25.0.4 sin intentar descargar Temurin).

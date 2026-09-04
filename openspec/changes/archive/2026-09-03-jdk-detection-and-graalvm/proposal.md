## Why

Jolt actualmente asume una versión fija de Java por defecto ("21") y utiliza una comprobación de versiones muy estricta y limitada (`javac -version.contains(requested_version)`). Cuando el desarrollador tiene un JDK alternativo instalado en el sistema (como Oracle GraalVM 25, Eclipse Temurin, Corretto o Zulu en `JAVA_HOME`, `GRAALVM_HOME` o rutas estándar), Jolt falla en detectarlo y descarga automáticamente OpenJDK Temurin sin previo aviso ni consentimiento explícito.

Adicionalmente, el manifiesto `jolt.toml` no cuenta con soporte sintáctico para configurar GraalVM Native Image, impidiendo que los usuarios definan argumentos de compilación nativa (`native-image`), clases principales o banderas de optimización AOT directamente en su proyecto.

## What Changes

- **Detección inteligente de JDKs instalados**:
  - Escaneo proactivo de variables de entorno (`JAVA_HOME`, `GRAALVM_HOME`) y rutas del sistema (`PATH`, `C:\Program Files\Java`, `C:\Program Files\GraalVm`, `/usr/lib/jvm`, `/Library/Java/JavaVirtualMachines`).
  - Extracción robusta de versiones mayores de Java (ej. parsing semántico de `javac -version` y `java -version` para 17, 21, 25, etc.) e identificación del vendor (GraalVM, Temurin, Corretto, Zulu, OpenJDK).
  - Reutilización de JDKs instalados compatibles si cumplen con la versión requerida o superior compatible (`>= requested_version` o coincidencia exacta según configuración).
- **Control de descarga de JDK**:
  - Si no se encuentra un JDK compatible instalado, Jolt no descargará un JDK en segundo plano silenciosamente; en su lugar, informará al usuario detallando el JDK requerido y cómo instalarlo o habilitar la descarga automática (o bandera explícita).
- **Sintaxis de configuración GraalVM en `jolt.toml`**:
  - Soporte para la sección `[graalvm]` (y `[package.graalvm]`) en `jolt.toml`, permitiendo configurar:
    - `enabled`: booleano para habilitar compilación nativa.
    - `args`: lista de argumentos para `native-image` (ej. `["--no-fallback", "-Ob"]`).
    - `main_class`: clase principal para el binario nativo.
    - `name` / `binary_name`: nombre del binario generado.
    - `reflection_config` / `resources_config`: rutas a archivos de metadata GraalVM.
- **Integración del compilador nativo**:
  - Soporte para compilación de binarios nativos vía GraalVM `native-image` desde Jolt.

## Capabilities

### New Capabilities
- `toolchain-detection`: Detección avanzada de JDKs locales, resolución de vendors (GraalVM, HotSpot), compatibilidad de versiones y prevención de descargas no solicitadas.
- `graalvm-config`: Soporte en el esquema de `jolt.toml` y CLI para configuración y ejecución de compilación nativa con GraalVM `native-image`.

### Modified Capabilities
<!-- No existing OpenSpec specs were previously tracked in openspec/specs/ -->

## Impact

- `src/toolchain.rs`: Nueva lógica de escaneo multinivel (PATH, JAVA_HOME, GRAALVM_HOME, directorios estándar del sistema), parser de versión de JDK y política de aviso/descarga controlada.
- `src/manifest.rs`: Nueva estructura `GraalVmConfig` agregada a `JoltManifest` y `PackageConfig`.
- `src/engine.rs`: Detección y ejecución de `native-image` integrando las opciones de `jolt.toml`.
- `src/cli.rs` & `src/main.rs`: Manejo de banderas (`--native` en build, etc.) y mensajes amigables cuando no se encuentra un JDK adecuado.

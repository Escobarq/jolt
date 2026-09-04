# Jolt - Especificaciones Técnicas Fase 6

Este documento detalla la arquitectura y diseño técnico para las funcionalidades de la Fase 6:
1. **Detección Inteligente Multinivel de JDKs Locales**
2. **Compatibilidad Semántica de Versiones Mayores (`major >= requested`)**
3. **Control Estricto de Auto-Descarga y Prevención de Descargas Silenciosas**
4. **Sintaxis de Configuración GraalVM en `jolt.toml` (`[graalvm]`)**
5. **Compilación de Binarios Nativos AOT con GraalVM Native Image (`jolt build --native`)**

---

## 1. Módulo U: Detección Multinivel de JDKs (`src/toolchain.rs`)

**Responsabilidad:** Descubrir y verificar todas las instalaciones locales de Java en el sistema anfitrión sin depender de descargas redundantes.

### Fuentes de Escaneo (en orden de prioridad):
1. **Variables de entorno:** `GRAALVM_HOME` y `JAVA_HOME`.
2. **Ejecutables en PATH:** Detección de `javac` y `java` y resolución de su directorio raíz.
3. **Directorios estándar del sistema operativo:**
   - **Windows:** `C:\Program Files\GraalVm`, `Java`, `Eclipse Adoptium`, `Amazon Corretto`, `Zulu`, `BellSoft`, `Microsoft`.
   - **Linux:** `/usr/lib/jvm`, `/opt/java`, `/opt/graalvm`, `~/.sdkman/candidates/java`.
   - **macOS:** `/Library/Java/JavaVirtualMachines`, `~/Library/Java/JavaVirtualMachines`.

### Detección Semántica y Extracción de Vendor:
- **`parse_java_major_version`**: Extrae la versión mayor numérica (`u32`) desde formatos modernos (ej. `25.0.4` → `25`, `21.0.2` → `21`) y legados (ej. `1.8.0_351` → `8`).
- **`detect_java_vendor`**: Identifica el distribuidor de la JVM (*Oracle GraalVM*, *Eclipse Temurin*, *Amazon Corretto*, *Azul Zulu*, *BellSoft Liberica*, *Microsoft OpenJDK*, *Oracle JDK*, *OpenJDK*).
- **Detección de `native-image`**: Identifica binarios `native-image`, `native-image.cmd` y `native-image.exe`.

---

## 2. Módulo V: Compatibilidad y Control de Descargas (`src/toolchain.rs`, `src/cli.rs`)

**Responsabilidad:** Evaluar compatibilidad de versiones y garantizar consentimiento explícito del usuario.

### Reglas de Compatibilidad:
- Si el proyecto solicita `java_version = "21"`, cualquier JDK instalado con `major_version >= 21` (como Java 25) es aceptado como compatible y reutilizado.
- Criterio de selección: Coincidencia exacta > GraalVM > versión compatible más cercana.

### Prevención de Descargas No Solicitadas:
- Si no existe un JDK compatible en el sistema:
  - **Sin flag `--download-jdk`:** Jolt finaliza con un error descriptivo detallando los JDKs instalados en el entorno y proporcionando instrucciones para instalarlo o autorizar la descarga.
  - **Con flag `--download-jdk`:** Jolt descarga y aprovisiona automáticamente Eclipse Temurin desde Adoptium API.

---

## 3. Módulo W: Configuración GraalVM en `jolt.toml` (`src/manifest.rs`)

**Responsabilidad:** Permitir a los proyectos configurar parámetros de compilación nativa AOT directamente en su manifiesto.

### Esquema de Configuración:
```toml
[graalvm]
enabled = false                       # Habilita compilación nativa por defecto
name = "mi-binario"                   # Nombre del ejecutable generado
main_class = "com.empresa.app.Main"   # Clase principal para el entrypoint nativo
args = [                              # Argumentos para native-image
    "--no-fallback",
    "-H:+ReportExceptionStackTraces"
]
reflection_config = "reflect.json"    # Metadata de reflexión
resources_config = "resources.json"   # Metadata de recursos
```

---

## 4. Módulo X: Compilador Nativo GraalVM (`src/engine.rs`, `src/main.rs`)

**Responsabilidad:** Coordinar el pipeline de compilación a binario nativo.

### Flujo de Ejecución:
1. `jolt build --native` detecta el toolchain y el ejecutable `native-image`.
2. Compila el código fuente Java a `.class`.
3. Empaqueta un Fat-JAR autónomo con todas las dependencias.
4. Invoca `native-image` pasando el Fat-JAR, el nombre de salida y los argumentos configurados en `jolt.toml`.
5. Deposita el binario ejecutable final en el directorio `dist/`.

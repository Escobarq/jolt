# ⚡ Jolt

**Gestor de paquetes, dependencias y herramientas para Java de nueva generación, ultrarrápido y desarrollado en Rust.**

Inspirado en la velocidad y ergonomía de herramientas modernas como `uv` (Python) y `bun` (JavaScript), Jolt elimina el XML verboso de Maven, los tiempos de arranque pesados de la JVM y la complejidad innecesaria en el desarrollo con Java.

---

## 🚀 Benchmark

En pruebas de resolución e instalación de dependencias en un árbol estándar con `hyperfine`:

| Gestor | Comando | Tiempo Promedio | Relación |
|---|---|---|---|
| **Jolt** | `jolt install` | **25.0 ms** | **1.0x (🚀 El más rápido)** |
| **Gradle** | `gradle dependencies` | 1.20 s | 48x más lento |
| **Maven** | `mvn dependency:resolve` | 8.73 s | 349x más lento |

---

## 📦 Instalación

### Con Cargo (Rust)
```bash
git clone https://github.com/Escobarq/jolt.git
cd jolt
cargo install --path .
```

### Instalador Oficial para Windows (.exe con NSIS y UPX)
```powershell
pwsh scripts/build-windows-installer.ps1
```
*Genera `dist/jolt-setup.exe` (o `dist/jolt-v0.7.0-setup.exe`) configurando automáticamente `jolt` en el `PATH` del usuario con desinstalador limpio.*

---

## ⚡ Inicio Rápido

```bash
# 1. Crear nuevo proyecto o Workspace Monorepo
jolt init mi-proyecto --template javafx --package com.empresa.app
cd mi-proyecto

# 2. Añadir dependencias desde Maven Central
jolt add com.google.code.gson:gson

# 3. Ejecutar la aplicación con Hot Reload
jolt run --watch

# 4. Ejecutar pruebas unitarias integradas (JUnit 5)
jolt test

# 5. Compilar Fat-JAR autónomo, Binario Nativo GraalVM o Instalador
jolt build --standalone
jolt build --native            # 🚀 Binario nativo instantáneo con GraalVM Native Image
jolt build --installer         # 📦 Instalador nativo (.msi / NSIS .exe)
```

> 💡 **Ayuda interactiva en CLI:**
> Toda la referencia completa y actualizada de opciones se consulta directamente en la terminal:
> ```bash
> jolt --help
> jolt <comando> --help    # Ej: jolt build --help, jolt package --help, jolt init --help
> jolt check               # Diagnostica tu entorno (Java, GraalVM, Rust, NSIS, UPX, Caché)
> ```

---

## ☕ Detección Inteligente de JDKs y Control de Descargas

Jolt incorpora un motor de resolución multinivel para Java:
- **Escaneo inteligente de tu entorno**: Inspecciona variables `GRAALVM_HOME` y `JAVA_HOME`, ejecutables en `PATH` (`javac`, `java`) y directorios estándar del sistema operativo (Oracle GraalVM, Eclipse Temurin, Amazon Corretto, Azul Zulu, BellSoft, Microsoft OpenJDK, etc.).
- **Compatibilidad semántica**: Si tu proyecto define `java_version = "21"` y cuentas con un JDK más moderno instalado (por ejemplo, **Oracle GraalVM 25**), Jolt lo detecta y lo utiliza automáticamente sin necesidad de descargar versiones redundantes.
- **Sin descargas silenciosas**: Jolt jamás descargará paquetes pesados en segundo plano sin consentimiento. Si no se encuentra un JDK adecuado, te indicará claramente los JDKs detectados y cómo instalar uno compatible, o puedes autorizar la auto-descarga de Eclipse Temurin usando el flag `--download-jdk`.

---

## 📄 Ejemplo de Manifiesto (`jolt.toml`)

```toml
[project]
name = "mi-aplicacion"
version = "1.0.0"
java_version = "21"
package = "com.empresa.app"

[dependencies]
"com.google.code.gson:gson" = "2.14.0"
"org.openjfx:javafx-controls" = "21.0.2:linux"

[dev-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.10.2"

# Configuración opcional para GraalVM Native Image
[graalvm]
enabled = false # o activar en CLI con: jolt build --native
name = "mi-app-bin"
args = [
    "--no-fallback",
    "-H:+ReportExceptionStackTraces"
]
# reflection_config = "reflect-config.json"
# resources_config = "resource-config.json"

[package]
type = "nsis"
name = "MiAplicacion"
dest = "dist"

[package.windows]
upx = true
add_to_path = true
desktop_shortcut = true
```

---

## 📚 Documentación Técnica y Módulos
Para especificaciones técnicas detalladas, diseño de arquitectura y especificaciones de módulos, consulta la carpeta [`docs/`](docs/).

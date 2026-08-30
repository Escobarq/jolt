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
*Genera `dist/jolt-setup.exe` (o `dist/jolt-v0.6.0-setup.exe`) configurando automáticamente `jolt` en el `PATH` del usuario con desinstalador limpio.*

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

# 5. Compilar Fat-JAR autónomo o Instalador Nativo
jolt build --standalone
jolt build --installer
```

> 💡 **Ayuda interactiva en CLI:**
> Toda la referencia completa y actualizada de opciones se consulta directamente en la terminal:
> ```bash
> jolt --help
> jolt <comando> --help    # Ej: jolt build --help, jolt package --help, jolt init --help
> jolt check               # Diagnostica tu entorno (Java, Rust, NSIS, UPX, Caché)
> ```

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

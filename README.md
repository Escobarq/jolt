# ⚡ Jolt

**El gestor de paquetes, dependencias y herramientas para Java de nueva generación, ultrarrápido y escrito en Rust.**

Inspirado en la velocidad y ergonomía de herramientas modernas como **`uv`** (Python) y **`bun`** (JavaScript), **Jolt** elimina la sobrecarga del XML, los tiempos de arranque pesados de la JVM y la complejidad innecesaria en el desarrollo con Java.

---

## 🚀 Características Principales

- ⚡ **Extremadamente Rápido:** Escrito en **Rust nativo** sin tiempo de arranque de la JVM para comandos de CLI.
- 📦 **Manifiesto Simple (`jolt.toml`):** Configuración limpia y legible en formato TOML en lugar de miles de líneas en `pom.xml`.
- 🌐 **Resolución Asíncrona con Maven Central:** Búsqueda en tiempo real de versiones y árbol de dependencias transitivas con parsers XML ultrarrápidos.
- 🔗 **Caché Global y Deduplicación con Hardlinks:** Las librerías se descargan una sola vez a `~/.jolt/cache/v1/` y se comparten entre proyectos mediante *hardlinks* a nivel de inodo en el sistema de archivos (ahorrando gigabytes de almacenamiento y tiempo de descarga).
- 🛠️ **Aprovisionamiento Automático de Toolchains:** Detección de JDKs locales del sistema o descarga bajo demanda de distribuciones OpenJDK Temurin (LTS).
- 📦 **Fat-JAR / Standalone Bundler (`jolt build --standalone`):** Empaqueta todo tu código y librerías en un único archivo `.jar` autónomo con filtrado automático de firmas digitales para distribución directa.
- 🧪 **Suite de Pruebas Unitarias Integrada (`jolt test`):** Ejecución nativa de pruebas con JUnit 5 Platform Console Launcher sin plugins externos.
- 🔥 **Modo Observador / Hot Reload (`jolt run --watch`):** Recompilación y reinicio instantáneo de la aplicación Java al guardar cambios en el código.
- 🎨 **Gestión de Recursos Estáticos:** Copia automática de activos desde `src/main/resources/` (archivos `.fxml` de JavaFX, `.properties`, `.css`, imágenes, etc.).
- 🩺 **Diagnóstico de Entorno y Salud (`jolt check`):** Auditoría en tiempo real del estado de tus herramientas (`java`, `javac`, `jar`, `rustc`, `cargo`), tamaño de caché e integridad de dependencias locales.

---

## 📦 Instalación

### Compilar e Instalar desde el Código Fuente
```bash
git clone https://github.com/juandavidescobarquezada/jolt.git
cd jolt
cargo install --path .
```

Verifica la instalación:
```bash
jolt --version
jolt check
```

---

## 💻 Guía Rápida de Comandos

| Comando | Descripción |
|---|---|
| `jolt init [nombre]` | Inicializa un nuevo proyecto Java con estructura estándar y `jolt.toml` |
| `jolt add <groupId:artifactId[:version]>` | Añade una dependencia desde Maven Central y la enlaza |
| `jolt install` | Sincroniza e instala todas las dependencias declaradas en `jolt.toml` |
| `jolt run` | Compila y ejecuta el proyecto en tiempo real |
| `jolt run --watch` (`-w`) | Ejecuta la aplicación con **Hot Reload** al detectar cambios de archivos |
| `jolt build` | Compila el proyecto y genera un `.jar` estándar en `target/` |
| `jolt build --standalone` (`-s`) | Genera un **Fat-JAR autónomo** con todas las dependencias embebidas |
| `jolt test` | Ejecuta las pruebas unitarias en `src/test/java/` con **JUnit 5** |
| `jolt check` | Diagnostica el entorno del sistema y la salud de las dependencias |

---

## 📝 Ejemplo de `jolt.toml`

```toml
[project]
name = "mi_aplicacion"
version = "0.1.0"
java_version = "21"

[dependencies]
"com.google.code.gson:gson" = "2.14.0"
"org.slf4j:slf4j-api" = "2.1.0-alpha1"
"org.openjfx:javafx-controls" = "21.0.2:linux"

[dev-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.10.2"
```

---

## 🏛️ Estructura del Proyecto

```text
mi_proyecto/
├── jolt.toml                  # Configuración del proyecto y dependencias
├── .jolt/
│   └── modules/               # Enlaces a los JARs en la caché global
├── src/
│   ├── main/
│   │   ├── java/              # Código fuente Java principal (Main.java, etc.)
│   │   └── resources/         # Archivos estáticos (.properties, .fxml, .css)
│   └── test/
│       └── java/              # Pruebas unitarias JUnit 5 (*Test.java)
└── target/
    ├── classes/               # Bytecode compilado
    ├── test-classes/          # Bytecode de pruebas compilado
    └── mi_proyecto-0.1.0.jar  # JAR resultante
```

---

## 📚 Documentación y Especificaciones
- [Especificaciones Fase 1 (Core)](docs/specs.md)
- [Especificaciones Fase 2 (Advanced)](docs/specs-v2.md)
- [Registro de Módulos Archivados](docs/archive/)

---

## 📄 Licencia
Este proyecto está bajo la Licencia MIT.

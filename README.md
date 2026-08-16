# Jolt

**Gestor de paquetes, dependencias y herramientas para Java de nueva generacion, ultrarrapido y desarrollado en Rust.**

Inspirado en la velocidad y ergonomia de herramientas modernas como `uv` (Python) y `bun` (JavaScript), Jolt elimina la sobrecarga del XML, los tiempos de arranque pesados de la JVM y la complejidad innecesaria en el desarrollo con Java.

---

## Caracteristicas Principales

- **Rendimiento Nativo:** Desarrollado en Rust sin tiempo de arranque de JVM para operaciones CLI.
- **Manifiesto Simple (`jolt.toml`):** Configuracion limpia y legible en formato TOML en reemplazo de `pom.xml`.
- **Resolucion Asincrona con Maven Central:** Busqueda en tiempo real de versiones y arbol de dependencias transitivas con parseo XML optimizado.
- **Cache Global y Deduplicacion con Hardlinks:** Almacenamiento unico en `~/.jolt/cache/v1/` y enlaces a nivel de inodo en el sistema de archivos (`.jolt/modules/`).
- **Aprovisionamiento Automatico de Toolchains:** Deteccion de JDKs instalados y descarga bajo demanda de distribuciones OpenJDK Temurin (LTS).
- **Fat-JAR / Standalone Bundler (`jolt build --standalone`):** Empaquetado de aplicacion y dependencias en un unico archivo `.jar` ejecutable con filtrado de firmas de seguridad.
- **Suite de Pruebas Unitarias Integrada (`jolt test`):** Ejecucion nativa de pruebas con JUnit 5 Platform Console Launcher.
- **Modo Observador / Hot Reload (`jolt run --watch`):** Recompilacion y reinicio automatico de la aplicacion al detectar cambios en el codigo.
- **Gestion de Recursos Estaticos:** Copia automatica de archivos desde `src/main/resources/` (`.fxml` de JavaFX, `.properties`, `.yaml`, `.json`, `.css`).
- **Diagnostico de Entorno y Proyecto (`jolt check`):** Auditoria del estado de herramientas (`java`, `javac`, `jar`, `rustc`, `cargo`), cache y dependencias.

---

## Instalacion

### Compilar e Instalar desde el Codigo Fuente
```bash
git clone https://github.com/Escobarq/jolt.git
cd jolt
cargo install --path .
```

Verificar la instalacion:
```bash
jolt --version
jolt check
```

---

## Guia Rapida de Comandos

| Comando | Descripcion |
|---|---|
| `jolt init [nombre]` | Inicializa un nuevo proyecto Java (`minimal` por defecto) |
| `jolt init [nombre] --template <cli\|javafx\|web\|spring>` | Inicializa un proyecto preconfigurado con plantillas de inicio |
| `jolt init --list-templates` (`-l`) | Muestra la lista de todas las plantillas disponibles con descripcion |
| `jolt add <groupId:artifactId[:version]>` | Anade una dependencia desde Maven Central, la enlaza y actualiza `jolt.lock` |
| `jolt remove <groupId:artifactId>` (`rm`) | Elimina una dependencia de `jolt.toml`, remueve el `.jar` y actualiza `jolt.lock` |
| `jolt install` | Sincroniza e instala todas las dependencias declaradas en `jolt.toml` |
| `jolt install --locked` | Instalacion determinista y estricta para entornos CI/CD usando `jolt.lock` |
| `jolt run` | Compila y ejecuta el proyecto en tiempo real |
| `jolt run --watch` (`-w`) | Ejecuta la aplicacion con **Hot Reload** continuo al editar archivos |
| `jolt build` | Compila el proyecto y genera un `.jar` estandar en `target/` |
| `jolt build --standalone` (`-s`) | Genera un **Fat-JAR autonomo** con todas las dependencias embebidas |
| `jolt test` | Ejecuta las pruebas unitarias en `src/test/java/` con **JUnit 5** |
| `jolt check` | Diagnostica el entorno del sistema y la salud de las dependencias |

---

## Ejemplo de `jolt.toml`

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

## Estructura del Proyecto

```text
mi_proyecto/
├── jolt.toml                  # Configuracion del proyecto y dependencias
├── .jolt/
│   └── modules/               # Enlaces a los JARs en la cache global
├── src/
│   ├── main/
│   │   ├── java/              # Codigo fuente Java principal (Main.java, etc.)
│   │   └── resources/         # Archivos estaticos (.properties, .fxml, .css)
│   └── test/
│       └── java/              # Pruebas unitarias JUnit 5 (*Test.java)
└── target/
    ├── classes/               # Bytecode compilado
    ├── test-classes/          # Bytecode de pruebas compilado
    └── mi_proyecto-0.1.0.jar  # JAR resultante
```

---

## Documentacion y Especificaciones
- [Especificaciones Fase 1 (Core)](docs/specs.md)
- [Especificaciones Fase 2 (Advanced)](docs/specs-v2.md)
- [Especificaciones Fase 3 (Lockfile, Templates, Remove)](docs/specs-v3.md)
- [Registro de Modulos Archivados](docs/archive/)

---

## Licencia
Este proyecto esta bajo la Licencia MIT.

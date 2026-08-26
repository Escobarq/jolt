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
- **Empaquetado Binario Nativo Multiplataforma (`jolt package`):** Creacion directa de lanzadores binarios autonomos ejecutables e instaladores de sistema (`app-image`, `deb`, `rpm`, `msi`, `exe`, `dmg`, `pkg`) con runtime optimizado mediante `jpackage`.
- **Paquetes Java y Namespaces Estructurados (`package org.equipo.proyecto;`):** Generación automática de árboles de directorios canónicos e inyección de namespaces en todas las plantillas.
- **Workspaces Multimódulo (Monorepos):** Gestión de múltiples submódulos heterogéneos (ej. módulo de escritorio con JavaFX, backend con Spring Boot y librerías compartidas Core) en un único repositorio con dependencias locales (`path = "../core"`).
- **Suite de Pruebas Unitarias Integrada (`jolt test`):** Ejecucion nativa de pruebas con JUnit 5 Platform Console Launcher.
- **Modo Observador / Hot Reload (`jolt run --watch`):** Recompilacion y reinicio automatico de la aplicacion al detectar cambios en el codigo.
- **Gestion de Recursos Estaticos:** Copia automatica de archivos desde `src/main/resources/` (`.fxml` de JavaFX, `.properties`, `.yaml`, `.json`, `.css`).
- **Diagnostico de Entorno y Proyecto (`jolt check`):** Auditoria del estado de herramientas (`java`, `javac`, `jar`, `jpackage`, `rustc`, `cargo`), cache y dependencias de proyectos o workspaces.

---

## 🚀 Rendimiento Insuperable (Benchmarks)

Jolt ha sido diseñado desde cero en Rust para eliminar los tiempos muertos en el desarrollo de Java. Mientras Maven y Gradle sufren de la sobrecarga de inicialización de la JVM, Jolt actúa de manera casi instantánea.

En pruebas de resolución e instalación de dependencias en un proyecto con un árbol estándar, **Jolt destroza a las alternativas tradicionales**:

| Gestor | Comando | Tiempo Promedio | Relación |
|---|---|---|---|
| **Jolt** | `jolt install` | **25.0 ms** | **1.0x (🚀 El más rápido)** |
| **Gradle** | `gradle dependencies` | 1.20 s | 48x más lento |
| **Maven** | `mvn dependency:resolve` | 8.73 s | 349x más lento |

> *Nota: Benchmark automatizado realizado con `hyperfine`. Puedes reproducir estas métricas usando el script disponible en la carpeta `benchmark/` del repositorio.*

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
| `jolt init` | Asistente interactivo: elige entre **Proyecto Individual** o **Workspace Monorepo**, define el paquete Java y la plantilla |
| `jolt init [nombre] --package <org.equipo.app>` | Inicializa un proyecto con paquete / namespace Java canónico |
| `jolt init [nombre] --workspace` | Inicializa un nuevo **Workspace Monorepo** vacío con `[workspace]` |
| `jolt init [nombre] --template <cli\|javafx\|swing\|web\|spring>` | Inicializa un proyecto preconfigurado con plantillas de inicio |
| `jolt init --list-templates` (`-l`) | Muestra la lista de todas las plantillas disponibles con descripcion |
| `jolt search <query>` (`find`) | Busca librerias en Maven Central y genera el comando para anadirlas |
| `jolt add <groupId:artifactId[:version]>` | Anade una dependencia a `dependencies` (o usa `--member <nombre>` desde la raíz del workspace) |
| `jolt add <groupId:artifactId[:version]> --dev` (`-D`) | Anade una libreria a las dependencias de desarrollo (`dev-dependencies`) |
| `jolt remove <groupId:artifactId>` (`rm`) | Elimina una dependencia de `jolt.toml`, remueve el `.jar` y actualiza `jolt.lock` |
| `jolt install` | Sincroniza e instala dependencias (`modules/` y `dev-modules/`) de proyectos o todos los miembros del workspace |
| `jolt install --locked` | Instalacion determinista y estricta para entornos CI/CD usando `jolt.lock` |
| `jolt sync` | Sincroniza dependencias declaradas y regenera autoconfiguración de IDE (VS Code / Eclipse / Cursor) |
| `jolt run [-p <modulo>]` | Compila y ejecuta el proyecto o módulo especificado en tiempo real |
| `jolt run --watch` (`-w`) | Ejecuta la aplicacion con **Hot Reload** continuo al editar archivos |
| `jolt build [--all\|-p <modulo>]` | Compila el proyecto o módulos del workspace y genera `.jar` en `target/` |
| `jolt build --standalone` (`-s`) | Genera un **Fat-JAR autonomo** (fusionando dependencias de producción y módulos locales) |
| `jolt package [-p <modulo>]` (`pkg`, `bundle`) | Empaqueta la app en binario nativo o instalador (`app-image`, `deb`, `rpm`, `msi`, `exe`, `dmg`, `pkg`) |
| `jolt test [--all\|-p <modulo>]` | Ejecuta las pruebas unitarias en `src/test/java/` con **JUnit 5** |
| `jolt check` | Diagnostica el entorno del sistema y la salud de dependencias o submódulos del workspace |

---

## 🏢 Workspaces Multimódulo (Monorepos) y Paquetes Java

Jolt permite organizar arquitecturas complejas en un solo repositorio, aislando dependencias heterogéneas (como JavaFX en un módulo desktop y Spring Boot en un backend web) compartiendo lógica común mediante dependencias locales:

### 1. Crear el Workspace Monorepo
```bash
jolt init mi-monorepo --workspace
cd mi-monorepo
```

### 2. Crear los Submódulos con Paquetes
```bash
# Módulo Core compartido
jolt init core --package org.equipo.core --template minimal

# Módulo Desktop con JavaFX
jolt init desktop_app --package org.equipo.desktop --template javafx

# Módulo Backend con Spring Boot
jolt init web_api --package org.equipo.web --template spring
```

Jolt registrará automáticamente los submódulos en el `jolt.toml` raíz del workspace:
```toml
[workspace]
members = ["core", "desktop_app", "web_api"]
```

### 3. Enlazar Dependencias Locales Inter-Módulo (`path dependencies`)
En `desktop_app/jolt.toml` o `web_api/jolt.toml`:
```toml
[dependencies]
"org.openjfx:javafx-controls" = "21.0.2:linux"
"core" = { path = "../core" } # Dependencia local inter-módulo
```

### 4. Operar desde la Raíz del Monorepo
```bash
# Instalar dependencias de todos los submódulos
jolt install

# Ejecutar el módulo de escritorio JavaFX
jolt run -p desktop_app

# Ejecutar el servicio Spring Boot
jolt run -p web_api

# Empaquetar un binario nativo del módulo desktop
jolt package -p desktop_app
```

---

## Empaquetado Binario Nativo con `jpackage`

Jolt permite generar binarios ejecutables e instaladores nativos listos para producción para Linux, Windows y macOS sin requerir Java instalado en la máquina del usuario final:

```bash
# Crear un lanzador binario portable con runtime integrado (app-image en dist/)
jolt package

# Ejecutar directamente el binario generado
./dist/mi_aplicacion/bin/mi_aplicacion       # Linux
./dist/mi_aplicacion/mi_aplicacion.exe       # Windows
open ./dist/mi_aplicacion.app                # macOS

# Generar instalador nativo (.deb, .rpm, .msi, .exe, .dmg, .pkg)
jolt package --type deb
jolt package --type msi
jolt package --type dmg

# Personalizar el empaquetado por CLI
jolt package --name "MiApp" --dest "release" --icon "assets/icon.png" --java-options "-Xmx512m"
```

---

## Ejemplo de `jolt.toml`

```toml
[project]
name = "mi_aplicacion"
version = "0.1.0"
java_version = "21"
package = "org.equipo.miapp"
group_id = "org.equipo"
main_class = "org.equipo.miapp.Main" # Opcional (Jolt la auto-detecta si se omite)

[package]
type = "app-image"              # app-image, deb, rpm, msi, exe, dmg, pkg
name = "mi-lanzador"            # Nombre opcional personalizado del binario
vendor = "Mi Empresa"
description = "Lanzador nativo de alta velocidad"
icon = "src/main/resources/icon.png"
dest = "dist"
java_options = ["-Xmx512m", "-Dfile.encoding=UTF-8"]

[dependencies]
"com.google.code.gson:gson" = "2.14.0"
"org.slf4j:slf4j-api" = "2.1.0-alpha1"
"org.openjfx:javafx-controls" = "21.0.2:linux"
"core" = { path = "../core" }

[dev-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.10.2"
```

---

## Estructuras del Proyecto

### 📦 Proyecto Individual
```text
mi_proyecto/
├── jolt.toml                  # Configuracion del proyecto y dependencias
├── jolt.lock                  # Arbol determinista de dependencias con hashes SHA-256
├── .vscode/
│   ├── settings.json          # Configuracion de classpath y libraries para VS Code / Cursor
│   └── extensions.json        # Extensiones recomendadas para Java y TOML
├── .project                   # Descriptor de proyecto para Eclipse / Java Language Server
├── .classpath                 # Enlace directo de fuentes y bibliotecas JAR al Language Server
├── .gitignore                 # Exclusion de target/, dist/ y binarios generados
├── .jolt/
│   ├── modules/               # Enlaces simbolicos/hardlinks a JARs de produccion
│   └── dev-modules/           # Enlaces a librerias de desarrollo y testing (JUnit, etc.)
├── src/
│   ├── main/
│   │   ├── java/              # Codigo fuente (ej. org/equipo/miapp/Main.java)
│   │   └── resources/         # Archivos estaticos (.properties, .fxml, .css, .json)
│   └── test/
│       └── java/              # Pruebas unitarias JUnit 5 (ej. org/equipo/miapp/AppTest.java)
├── target/
│   ├── classes/               # Bytecode compilado de la aplicacion
│   ├── test-classes/          # Bytecode compilado de las pruebas unitarias
│   └── mi_proyecto-0.1.0.jar  # JAR estandar o Fat-JAR generado
└── dist/
    └── mi_proyecto/
        ├── bin/mi_proyecto    # Lanzador binario nativo autonomo
        └── lib/               # Runtime JVM minimalista y librerias embebidas
```

### 🏢 Workspace Multimódulo (Monorepo)
```text
mi_monorepo/
├── jolt.toml                  # [workspace] members = ["core", "desktop_app", "web_api"]
├── core/
│   ├── jolt.toml              # [project] name = "core" package = "org.equipo.core"
│   └── src/main/java/org/equipo/core/
│       └── CoreService.java
├── desktop_app/
│   ├── jolt.toml              # Depende de core: "core" = { path = "../core" } + JavaFX
│   └── src/main/java/org/equipo/desktop/
│       └── DesktopApp.java
└── web_api/
    ├── jolt.toml              # Depende de core: "core" = { path = "../core" } + Spring Boot
    └── src/main/java/org/equipo/web/
        └── WebApiApp.java
```

---

## Documentacion y Especificaciones
- [Especificaciones Fase 1 (Core CLI y Maven)](docs/specs.md)
- [Especificaciones Fase 2 (Avanzado, Toolchains y Fat-JARs)](docs/specs-v2.md)
- [Especificaciones Fase 3 (Lockfile, Plantillas y Remove)](docs/specs-v3.md)
- [Especificaciones Fase 4 (Empaquetado Nativo con jpackage)](docs/specs-v4.md)
- [Especificaciones Fase 5 (Paquetes FQCN y Workspaces Monorepo)](docs/specs-v5.md)
- [Registro de Modulos Archivados](docs/archive/)

---

## Licencia
Este proyecto esta bajo la Licencia MIT.



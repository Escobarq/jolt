# Jolt: ¿Cómo gestiona dependencias Java de forma rápida?

Jolt gestiona paquetes, dependencias y herramientas para Java con un único binario escrito en Rust. Elimina el XML verboso de Maven, los tiempos de arranque pesados de la JVM y la complejidad innecesaria en el desarrollo con Java.

Inspirado en herramientas modernas como `uv` (Python) y `bun` (JavaScript), Jolt ofrece una experiencia de desarrollo rápida y sencilla.

---

## ¿Qué tan rápido es Jolt?

Pruebas de resolución e instalación de dependencias en un árbol estándar con `hyperfine`:

| Gestor | Comando | Tiempo Promedio | Relación |
|---|---|---|---|
| **Jolt** | `jolt install` | **25.0 ms** | **1.0x (🚀 El más rápido)** |
| **Gradle** | `gradle dependencies` | 1.20 s | 48x más lento |
| **Maven** | `mvn dependency:resolve` | 8.73 s | 349x más lento |

---

## ¿Cómo instalo Jolt?

### Instalación con Cargo (Rust)
```bash
git clone https://github.com/Escobarq/jolt.git
cd jolt
cargo install --path .
```

### Instalador oficial para Windows (.exe con NSIS y UPX)
```powershell
pwsh scripts/build-windows-installer.ps1
```
*Genera `dist/jolt-setup.exe` (o `dist/jolt-v0.7.3-windows-x86_64-setup.exe`) compilando el binario actual, validando su arranque y configurando automáticamente `jolt` en el `PATH` del usuario con desinstalador limpio.*

**Nota:** La versión actual es **v0.7.3**. Consulte [`docs/plan.md`](docs/plan.md) para ver el estado completo del proyecto y todos los módulos funcionales implementados.

---

## ¿Cómo empiezo con Jolt?

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

# 5. Verificar dependencias, compilación, pruebas y arranque
jolt verify

# 6. Compilar Fat-JAR autónomo o instalador nativo
jolt build --standalone
jolt build --installer         # 📦 Instalador nativo (.msi / NSIS .exe)
```

Jolt detecta automáticamente el módulo cuyo `jolt.toml` corresponde al
directorio actual. En la raíz de un workspace con varios módulos puedes
seleccionar uno explícitamente:

```bash
jolt add org.slf4j:slf4j-api --member web-api
```

Las respuestas de búsqueda, versiones y POMs se almacenan en
`~/.jolt/cache/v1/metadata` para evitar consultas repetidas. Equipos que
usen un proxy o repositorio Maven interno pueden configurarlo en el
`jolt.toml` del proyecto o workspace:

```toml
[repositories]
search_url = "https://nexus.example.com/service/rest/v1/search"
artifact_url = "https://nexus.example.com/repository/maven-public"
cache_ttl_seconds = 3600
```

Maven Central se usa por defecto cuando esta sección no existe.
La configuración del proyecto tiene prioridad sobre la del workspace.

### Lockfile e instalaciones reproducibles

Usa `jolt install --locked` en CI para exigir que `jolt.toml`, `jolt.lock` y
los checksums de los artefactos coincidan. Si el comando rechaza un lockfile
desactualizado, regénéralo explícitamente y revisa el cambio antes de volver al
modo bloqueado:

```bash
jolt install
git diff -- jolt.lock
jolt install --locked
```

Los fallos al resolver, descargar, guardar o enlazar una dependencia son
errores fatales en `jolt install` y `jolt sync`; Jolt ya no continúa mostrando
una instalación incompleta como exitosa. Corrige la dependencia o el acceso al
repositorio y vuelve a ejecutar el comando.

**Obtén ayuda en la terminal:**
Consulta la referencia completa y actualizada de opciones directamente:
```bash
jolt --help
jolt <comando> --help    # Ej: jolt build --help, jolt package --help, jolt init --help
jolt check               # Diagnostica tu entorno (Java, jpackage, Rust, NSIS, UPX, Caché)
```

---

## ¿Cómo gestiona Jolt las versiones de Java?

Jolt incluye un motor de resolución multinivel para Java:
- **Escaneo del entorno**: Inspecciona variables `GRAALVM_HOME` y `JAVA_HOME`, ejecutables en `PATH` (`javac`, `java`) y directorios estándar del sistema (Oracle GraalVM, Eclipse Temurin, Amazon Corretto, Azul Zulu, BellSoft, Microsoft OpenJDK, etc.).
- **Compatibilidad semántica**: Si tu proyecto define `java_version = "21"` y tienes un JDK más moderno instalado (por ejemplo, **Oracle GraalVM 25**), Jolt lo detecta y lo usa automáticamente sin descargar versiones redundantes.
- **Solo JDK del sistema**: Jolt no descarga ni administra JDKs. Si no encuentra uno adecuado, te muestra los JDKs detectados y cómo instalar o configurar uno compatible.

---

## ¿Cómo se configura un proyecto en Jolt?

### Ejemplo de manifiesto (`jolt.toml`)

```toml
[project]
name = "mi-aplicacion"
version = "1.0.0"
java_version = "21"
package = "com.empresa.app"

[dependencies]
"com.google.code.gson:gson" = "2.14.0"
"org.openjfx:javafx-controls" = "21.0.2:<linux|win|mac>"

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

## ¿Dónde está la documentación técnica?

Para especificaciones técnicas detalladas, diseño de arquitectura y especificaciones de módulos, consulta la carpeta [`docs/`](docs/).

### Documentación técnica principal

| Recurso | Contenido |
|---|---|
| [`docs/specs.md`](docs/specs.md) | Índice maestro de módulos (A–U) y roadmap v0.8.0 (`jolt fmt`, `jolt doc`, `jolt audit`). |
| [`docs/design.md`](docs/design.md) | Arquitectura interna de `src/` (8 subsistemas) y flujo de trabajo del CLI. |
| [`docs/plan.md`](docs/plan.md) | Plan de desarrollo actual, estado del proyecto y comandos del CLI. |
| [`docs/archive/`](docs/archive/) | Especificaciones archivadas de cada módulo implementado (A–U). |
| [`openspec/`](openspec/) | Especificaciones OpenSpec, contexto del proyecto y registro de cambios aplicados. |
| [`graphify-out/`](graphify-out/) | Grafo de conocimiento del proyecto: `GRAPH_REPORT.md`, `graph.json`, `graph.html`. |

### Documentación web interactiva

La documentación web está disponible en dos idiomas:

| Idioma | Página Principal | Guía Rápida |
|---|---|---|
| **Español** | [`frontend-docs/es/index.html`](frontend-docs/es/index.html) | [`frontend-docs/es/docs/quickstart/index.html`](frontend-docs/es/docs/quickstart/index.html) |
| **English** | [`frontend-docs/en/index.html`](frontend-docs/en/index.html) | [`frontend-docs/en/docs/quickstart/index.html`](frontend-docs/en/docs/quickstart/index.html) |

La documentación web incluye:
- Guía de inicio rápido
- Referencia completa del CLI
- Configuración avanzada
- Arquitectura del sistema
- Guía de toolchains (JDK)
- Contribución al proyecto

# Jolt - Especificaciones Técnicas: v0.7.1 & Roadmap v0.8.0

Este documento detalla las mejoras y refactorizaciones de arquitectura implementadas en la versión **0.7.1** y establece las especificaciones técnicas completas para los módulos que se desarrollarán en la **versión 0.8.0**:

1. **Implementado en v0.7.1:**
   - **Módulo S+:** Presets preconfigurados de **GraalVM Native Image** en todas las plantillas.
   - **Módulo CLI+:** Soporte del flag `--graalvm` / `--native` en `jolt init` y en el asistente interactivo.
   - **Módulo Test+:** Suite de pruebas unitarias para la plantilla `web`.
   - **Módulo Arch+:** Refactorización y modularización completa de `src/` en 8 submódulos temáticos (`core`, `cli`, `build`, `resolver`, `packaging`, `toolchain`, `scaffold`, `doctor`).
   - Sincronización completa del directorio `templates/`.
2. **Especificado para v0.8.0 (Roadmap Futuro):**
   - **Módulo Y:** Formateador de Código Java (`jolt fmt`).
   - **Módulo Z:** Generación y Servidor de Documentación Javadoc (`jolt doc`).
   - **Módulo AA:** Auditoría de Seguridad y Detección de Vulnerabilidades CVE (`jolt audit`).

---

# PARTE 1: Implementación en Jolt v0.7.1

## 1. Módulo S+: Presets GraalVM y Enriquecimiento de Plantillas (`src/scaffold/`, `src/cli/`)

### 1.1 Objetivos y Cambios
- Todas las plantillas (`minimal`, `cli`, `javafx`, `swing`, `web`, `spring`) incluyen de fábrica un bloque `[graalvm]` optimizado según el tipo de aplicación:
  - **`minimal`**: Configurado para compilación nativa estándar con `--no-fallback`.
  - **`cli` (Picocli)**: Configurado con `--initialize-at-build-time` y `-H:+ReportExceptionStackTraces` para optimizar el arranque en 5 ms de utilidades de terminal.
  - **`web` (Javalin)**: Configurado con `--enable-http` y soporte de excepciones para microservicios ligeros.
  - **`spring` (Spring Boot 3)**: Preparado con argumentos AOT de Spring.
  - **`javafx` / `swing`**: Preparado para interfaces gráficas de usuario.
- **Flag `--graalvm` (alias `--native`):** Permite inicializar un proyecto con `enabled = true` directamente desde la terminal (`jolt init mi-app --template cli --graalvm`), permitiendo compilar inmediatamente con `jolt build`.
- **Asistente interactivo:** En modo interactivo (`jolt init`), Jolt consulta al usuario si desea activar GraalVM Native Image por defecto.

## 2. Módulo Arch+: Arquitectura Modular de `src/`

Para preparar la base de código para el crecimiento de la versión 0.8.0, la estructura plana de archivos en `src/` se modularizó en 8 paquetes cohesivos:
- `src/core/`: Parser de manifiestos, lockfiles y caché de almacenamiento.
- `src/cli/`: Definición de comandos, argumentos y parseo con `clap`.
- `src/build/`: Compilador (`javac`), runner con Hot Reload y ejecutor JUnit 5.
- `src/resolver/`: Cliente Maven Central y parser de POMs XML.
- `src/packaging/`: Fat-JAR, GraalVM AOT, jpackage, NSIS y compresión UPX.
- `src/toolchain/`: Detección multinivel del sistema y descargador de JDKs.
- `src/scaffold/`: Generador de plantillas, workspaces y configs de IDE.
- `src/doctor/`: Diagnóstico integral del sistema y proyectos.

---

# PARTE 2: Especificaciones Técnicas para Jolt v0.8.0

---

## 3. Módulo Y: Formateador Integrado de Código Java (`jolt fmt`)

### 3.1 Visión y Propósito
Garantizar la consistencia estilística del código Java en proyectos individuales y monorepos, ejecutándose de forma instantánea sin la lentitud de herramientas tradicionales de Maven o Gradle.

### 3.2 CLI e Interfaz
```bash
jolt fmt                     # Formatea todos los archivos .java en src/
jolt fmt --check             # Valida el formato sin modificar archivos (ideal para CI/CD)
jolt fmt --diff              # Muestra el diff unificado de los cambios necesarios
jolt fmt src/main/Main.java  # Formatea un archivo o directorio específico
```

### 3.3 Opciones de Configuración en `jolt.toml`
```toml
[format]
style = "google"          # "google" (2 espacios) o "aosp" (4 espacios)
max_width = 100           # Longitud máxima de línea (por defecto: 100)
sort_imports = true       # Reordena alfabéticamente los imports
remove_unused_imports = true
```

### 3.4 Arquitectura y Flujo de Ejecución
1. **Detección de Archivos:** Escanea recursivamente `src/main/java`, `src/test/java` o rutas especificadas vía `walkdir`.
2. **Motor de Formateo:**
   - Ejecución mediante un parser léxico y AST de Java integrado en Rust para transformaciones rápidas (indentación, espaciado, ordenamiento de imports).
   - Opcionalmente con fallback o integración nativa con el motor estándar Google Java Format empaquetado y aprovisionado en la caché global de Jolt.
3. **Modo CI (`--check`):** Retorna código de salida `0` si el código está correctamente formateado, o código `1` con la lista de archivos que requieren cambios.

---

## 4. Módulo Z: Generador y Servidor Local de Javadoc (`jolt doc`)

### 4.1 Visión y Propósito
Proporcionar una experiencia fluida de documentación para librerías y aplicaciones Java, generando HTML estándar de Javadoc e iniciando un servidor HTTP local para previsualización inmediata.

### 4.2 CLI e Interfaz
```bash
jolt doc                     # Genera la documentación Javadoc en target/doc/
jolt doc --open              # Genera la documentación y la abre en el navegador predeterminado
jolt doc --serve             # Inicia un servidor web embebido en http://localhost:8080
jolt doc --port 3000         # Especifica el puerto del servidor web
jolt doc --private           # Incluye miembros privados y protegidos en la documentación
```

### 4.3 Arquitectura y Flujo de Ejecución
1. **Resolución de Toolchain:** Obtiene el ejecutable `javadoc` a través de `ToolchainManager`.
2. **Resolución del Classpath:** Utiliza `BuildEngine::build_classpath` para resolver automáticamente todas las dependencias en `.jolt/modules/` y módulos locales, evitando errores de símbolos no resueltos durante la generación de Javadoc.
3. **Invocación:** Ejecuta `javadoc` con los flags correspondientes:
   ```bash
   javadoc -d target/doc -cp <classpath> -sourcepath src/main/java -subpackages <paquetes>
   ```
4. **Micro-servidor HTTP Embebido:** Servidor asíncrono ultraligero (`tokio` / `hyper`) que sirve los archivos estáticos de `target/doc/` y notifica la URL activa en la terminal.

---

## 5. Módulo AA: Auditoría de Seguridad y CVEs (`jolt audit`)

### 5.1 Visión y Propósito
Detectar vulnerabilidades de seguridad conocidas (CVEs) en las dependencias declaradas en `jolt.lock`, ofreciendo análisis preventivo antes de compilaciones o despliegues a producción.

### 5.2 CLI e Interfaz
```bash
jolt audit                               # Audita todas las dependencias del proyecto
jolt audit --severity high               # Filtra alertas por severidad mínima (low, medium, high, critical)
jolt audit --json                        # Salida en formato JSON para integración en pipelines CI/CD
jolt audit --fix                         # Sugiere versiones parcheadas seguras en jolt.toml
```

### 5.3 Arquitectura y Fuentes de Datos
1. **Extracción de Artefactos:** Lee el archivo determinista `jolt.lock` extrayendo el listado de coordenadas `groupId:artifactId:version` y sus checksums SHA-256.
2. **Consulta de Vulnerabilidades:**
   - Realiza consultas en lote (Batch Query) a la API pública de **OSV.dev** (`https://api.osv.dev/v1/querybatch`) en el ecosistema `Maven`.
   - Consulta cruzada opcional con la API de Sonatype OSS Index.
3. **Evaluación de Severidad:** Mapea el vector CVSS a categorías legibles:
   - 🟢 **LOW** (0.1 - 3.9)
   - 🟡 **MEDIUM** (4.0 - 6.9)
   - 🟠 **HIGH** (7.0 - 8.9)
   - 🔴 **CRITICAL** (9.0 - 10.0)
4. **Formato de Salida:**
   - Muestra tabla formateada con ID del advisory (ej. `GHSA-xxxx-xxxx` o `CVE-2024-xxxx`), dependencia afectada, versión vulnerable, severidad y versión mínima recomendada con el parche.
   - Retorna código de salida `1` si se encuentran vulnerabilidades de severidad mayor o igual a la configurada.

---

## 6. Matriz de Versiones y Plan de Entrega

| Versión | Alcance / Módulos | Estado |
|---|---|:---:|
| **v0.7.0** | Detección Multinivel de JDKs, GraalVM AOT nativo, NSIS + UPX, Monorepos | ✅ Publicada |
| **v0.7.1** | Presets GraalVM en plantillas, flag `--graalvm`, tests en template web, arquitectura modular `src/` | 🚀 **Implementada** |
| **v0.8.0** | `jolt fmt` (Formateador), `jolt doc` (Javadoc + Web), `jolt audit` (CVEs) | 📋 **Especificada** |

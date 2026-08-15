# Jolt ⚡️: Gestor de Proyectos Java de Nueva Generación

Este documento explora el diseño arquitectónico y funcional de un nuevo gestor de paquetes y proyectos para Java, inspirado en el rendimiento y la experiencia de desarrollo (DX) de herramientas modernas como **uv** (Python) y **bun** (JavaScript). 

El objetivo principal es simplificar radicalmente la creación de aplicaciones Java, reduciendo la verbosidad y multiplicando la velocidad operativa, manteniendo total compatibilidad con el ecosistema de Maven Central.

---

## 1. Visión y Motivación

Históricamente, los desarrolladores de Java han dependido de Maven y Gradle. Aunque son robustos, sufren de:
- **Tiempos de inicio lentos** (la JVM debe arrancar para ejecutar el gestor).
- **Curva de aprendizaje empinada** y configuración verbosa (XML complejo en Maven, Groovy/Kotlin intrincado en Gradle).
- **Resolución de dependencias pesada**.

**Jolt** busca ser un binario único, escrito en un lenguaje de sistemas de alto rendimiento (como **Rust** o **Zig**), que se ejecute instantáneamente y centralice todo el ciclo de vida del desarrollo en Java: inicialización de proyectos, gestión de la versión de la JDK, resolución de dependencias, compilación y ejecución.

## 2. Características Clave

### 🚀 Velocidad Extrema (Escrito en Rust)
Al igual que `uv` y `bun`, el CLI estará construido en Rust. Esto permite:
- **Inicio en milisegundos** (cold start instantáneo).
- Resolución de dependencias y descargas concurrentes agresivas.
- Uso de **Hardlinks y Caché Global** (similar a `pnpm` o `uv`), evitando descargar la misma dependencia JAR/POM en múltiples proyectos.

### 📦 Gestión de Entorno y JDK Integrada
No es necesario instalar Java manualmente. Jolt administrará versiones de la JDK por proyecto.
- `jolt run app.java` detectará la versión requerida, descargará la JDK correspondiente (ej. Eclipse Temurin) en caché y ejecutará el código automáticamente.

### 📄 Manifiesto Simplificado (`jolt.toml`)
Adiós al verboso `pom.xml`. Jolt usará un formato moderno y legible, inspirado en `Cargo.toml` o `pyproject.toml`.

```toml
[project]
name = "mi-app"
version = "1.0.0"
java_version = "21"

[dependencies]
"org.springframework.boot:spring-boot-starter-web" = "3.2.0"
"com.google.guava:guava" = "33.0.0-jre"

[dev-dependencies]
"org.junit.jupiter:junit-jupiter" = "5.10.1"
```

### 🤝 Compatibilidad Total con Maven Central
Jolt funcionará como un cliente ultrarrápido para los repositorios Maven.
- Podrá leer archivos `pom.xml` existentes para proyectos heredados.
- Publicará artefactos compatibles con repositorios Maven.
- Implementará un archivo `jolt.lock` determinista para builds reproducibles.

---

## 3. Arquitectura Interna

La arquitectura de Jolt se dividirá en varios motores modulares:

1. **PubGrub Resolver**: Un motor de resolución de versiones eficiente basado en el algoritmo PubGrub (usado por Cargo y uv) adaptado a las reglas de resolución transitiva y exclusiones de Maven.
2. **Pom Parser en Rust**: Un analizador XML ultrarrápido capaz de parsear el árbol de dependencias de Maven Central sin inicializar una JVM.
3. **Caché Direccionable por Contenido**: Almacén global `~/.jolt/cache` para JARs, POMs, y JDKs.
4. **Daemon de Compilación (Opcional)**: Integración con el compilador de Java (`javac`) mediante un servidor persistente en segundo plano (estilo Gradle Daemon, pero gestionado transparentemente por Rust) o usando compilación incremental en Rust + JNI.

---

## 4. Flujo de Trabajo (UX / CLI)

La interfaz de línea de comandos será intuitiva y directa:

- `jolt init` - Crea un proyecto con `jolt.toml` y `src/main/java/Main.java`.
- `jolt add <groupId:artifactId>` - Busca en Maven Central, resuelve la última versión y la añade al `.toml`.
- `jolt install` o `jolt sync` - Resuelve dependencias y genera el `jolt.lock`.
- `jolt run src/Main.java` - Compila al vuelo (en memoria o en una carpeta de build transparente) y ejecuta.
- `jolt build` - Empaqueta el proyecto en un fat-JAR o una imagen nativa (integración con GraalVM).

---

## 5. Roadmap 2026: Siguientes Pasos

Para lograr tener una primera versión utilizable este año, la estrategia de ejecución es:

1. **Trimestre 1: Core y Resolución Maven**
   - Implementar el analizador de POMs en Rust.
   - Algoritmo de resolución de dependencias desde Maven Central.
   - Creación del formato `jolt.lock` y descargas concurrentes.
2. **Trimestre 2: Entorno y Configuración**
   - Gestión automática de toolchains de Java (descarga de JDKs).
   - CLI `jolt init`, `jolt add` y soporte para `jolt.toml`.
3. **Trimestre 3: Integración de Build y Ejecución**
   - Envolver las llamadas a `javac` y `java`.
   - Soporte para ejecutar archivos y gestionar el CLASSPATH automáticamente.
4. **Trimestre 4: Integración Avanzada**
   - Soporte para ejecución de pruebas (JUnit integration sin plugins pesados).
   - Beta pública inicial.

---

¿Qué te parece este enfoque inicial? Podemos refinar partes específicas como la **sintaxis del archivo de configuración**, el **algoritmo de resolución de conflictos** (que en Maven a veces es caótico), o cómo estructuraríamos **el proyecto en Rust**.

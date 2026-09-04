## Context

Ver `proposal.md` para la motivación. Actualmente, Jolt delega la provisión del toolchain a `ToolchainManager::get_or_download_toolchain`, el cual sólo evalúa si `javac -version` contiene textualmente `requested_version`. Al fallar (como cuando el usuario tiene instalado Oracle GraalVM 25.0.4 y el proyecto pide Java 21 por defecto), descarga silenciosamente OpenJDK Temurin desde Adoptium API. Además, `jolt.toml` carece de soporte estructurado para configuración de GraalVM Native Image.

## Goals / Non-Goals

**Goals:**
- Implementar un motor de descubrimiento multinivel para JDKs instalados (`GRAALVM_HOME`, `JAVA_HOME`, `PATH` y directorios típicos del sistema operativo).
- Parsear versiones mayores de Java semánticamente (`u32`) y detectar el proveedor/vendor (GraalVM, Temurin, Corretto, Zulu, OpenJDK).
- Reutilizar JDKs instalados compatibles cuando la versión mayor instalada sea compatible (>= versión requerida por el proyecto).
- Evitar descargas no consentidas: emitir diagnósticos claros indicando al usuario que instale el JDK o que pase una bandera explícita (`--download-jdk`) si desea que Jolt lo descargue.
- Añadir la sección `[graalvm]` en `jolt.toml` para configurar opciones de `native-image` (argumentos, main_class, binario, reflection/resources).
- Proveer soporte en CLI (`jolt build --native`) para compilar a binario nativo usando GraalVM.

**Non-Goals:**
- Implementar un gestor de instalación completa de GraalVM desde cero (sólo detección del existente e invocación de `native-image`).
- Soporte para versiones heredadas de Java pre-Java 8.

## Decisions

### Decisión 1: Estructura `InstalledJdk` y detección semántica de versiones
- **Enfoque**: Representar cada JDK descubierto mediante una estructura `InstalledJdk` que incluye versión mayor, vendor, rutas a ejecutables (`java`, `javac`, `jar`, `jpackage`, `native-image`).
- **Parsing**: Ejecutar `javac -version` y `java -version` para extraer la versión mayor numérica (ej. `25` de `25.0.4`, `21` de `21.0.2`, `8` de `1.8.0_351`).
- **Alternativas descartadas**:
  - Comparación de cadenas (`contains`): Es la causa del bug actual donde `25.0.4` no contiene `21`.
  - Depender únicamente de `PATH`: Incompleto si el usuario tiene `JAVA_HOME` o `GRAALVM_HOME` apuntando a otra versión o instalado en directorios estándar sin agregar a PATH.

### Decisión 2: Compatibilidad de versiones mayores (>= requested)
- **Enfoque**: En el ecosistema Java, un JDK de versión superior (ej. Java 25) compila y ejecuta código de versiones objetivo anteriores (ej. Java 21) mediante `--release 21` o compatibilidad nativa. Si no hay un JDK idéntico, se utiliza el JDK instalado más cercano que cumpla `>= requested_version`.
- **Alternativas descartadas**:
  - Exigir coincidencia exacta obligatoria: Forzaría a descargar Temurin 21 a pesar de que el desarrollador ya tiene un JDK 25 moderno listo para usar.

### Decisión 3: Notificación de instalación en lugar de descarga silenciosa
- **Enfoque**: Si no hay ningún JDK compatible instalado, `ToolchainManager` emitirá un error amigable explicando qué JDK se requiere y recomendando su instalación o el uso de `--download-jdk` para autorizar la auto-descarga.
- **Alternativas descartadas**:
  - Descargar automáticamente sin preguntar: Causa sorpresas y consume ancho de banda y disco innecesariamente cuando el usuario prefiere su propia instalación.

### Decisión 4: Esquema `[graalvm]` en `jolt.toml`
- **Enfoque**: Agregar `pub graalvm: Option<GraalVmConfig>` en `JoltManifest` y `pub graalvm: Option<GraalVmConfig>` en `PackageConfig`.
  Campos:
  - `enabled`: `Option<bool>`
  - `args`: `Option<Vec<String>>`
  - `main_class`: `Option<String>`
  - `name`: `Option<String>`
  - `reflection_config`: `Option<String>`
  - `resources_config`: `Option<String>`

## Risks / Trade-offs

- **[Riesgo]** Múltiples JDKs instalados en el sistema pueden generar ambigüedad.
  - **Mitigación**: Prioridad determinista: 1) `GRAALVM_HOME` / `JAVA_HOME`, 2) `PATH`, 3) Directorios estándar del sistema. Si hay varios compatibles, se prioriza el que tenga coincidencia exacta de versión o la versión más cercana.
- **[Riesgo]** El comando `native-image` puede no estar instalado en algunas distribuciones de GraalVM básicas.
  - **Mitigación**: Detectar la presencia del binario `native-image` (o `native-image.cmd` en Windows) e informar instrucciones claras si falta el componente.

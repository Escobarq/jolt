# Modulo P: Comando `jolt sync` y Autoconfiguracion de VS Code e IDEs Java

**Responsabilidad:** Proporcionar autoconfiguración integral para Visual Studio Code, Cursor, Eclipse y Java Language Servers (JDTLS), asegurando soporte sintáctico TOML, indexación de bibliotecas JAR y autocompletado/sugerencias de código Java en tiempo real mediante el nuevo comando `jolt sync`.

## Problema Resuelto
1. **Detección de TOML:** Los editores como VS Code no asociaban `jolt.toml` ni `jolt.lock` automáticamente con el lenguaje TOML sin extensiones o configuraciones explícitas de asociación de archivos.
2. **Autocompletado de Java en VS Code:** La extensión oficial de Java para VS Code (Red Hat JDTLS) requiere descriptores de proyecto (`.project`, `.classpath`) y configuraciones de `sourcePaths` y `referencedLibraries` para indexar correctamente los `.jar` de `.jolt/modules/` y `.jolt/dev-modules/` y ofrecer sugerencias de código completas e inmediatas.

## Solución Técnica Implementada
1. **Comando `jolt sync` (`src/cli.rs` y `src/main.rs`):**
   - Descarga a la caché global (`~/.jolt/cache/v1/`) cualquier dependencia faltante de producción y desarrollo.
   - Enlaza los JARs a `.jolt/modules/` y `.jolt/dev-modules/`.
   - Limpia JARs huérfanos que ya no existan en `jolt.toml`.
   - Sincroniza el lockfile `jolt.lock`.
   - Regenera todos los archivos de configuración de IDE.

2. **Generador de Entorno IDE (`src/scaffold.rs` -> `ensure_ide_configuration`):**
   - `.vscode/settings.json`: Configura `files.associations` para `jolt.toml` y `jolt.lock`, define `java.project.sourcePaths`, `java.project.referencedLibraries` y `java.project.outputPath`.
   - `.vscode/extensions.json`: Recomienda el Java Extension Pack de Microsoft/Red Hat y Even Better TOML.
   - `.project`: Descriptor Eclipse con nature `org.eclipse.jdt.core.javanature`.
   - `.classpath`: Descriptor Classpath con rutas fuente, contenedor JRE y entradas explícitas para todos los `.jar` del proyecto.
   - `.gitignore`: Configuración por defecto para excluir artefactos compilados.

3. **Auditoría con `jolt check` (`src/checker.rs`):**
   - Verifica la presencia y consistencia de los archivos `.vscode/settings.json`, `.classpath` y `.project`.

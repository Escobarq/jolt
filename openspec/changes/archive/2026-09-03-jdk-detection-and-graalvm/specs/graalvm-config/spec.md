## Purpose

Permite configurar parámetros de compilación de binarios nativos con GraalVM Native Image dentro del manifiesto jolt.toml y ejecutarlos mediante Jolt.

## ADDED Requirements

### Requirement: Esquema de configuración GraalVM en jolt.toml
El sistema SHALL permitir la definición de una tabla `[graalvm]` (y opcionalmente `[package.graalvm]`) en `jolt.toml` con opciones para el compilador `native-image`, incluyendo: `enabled` (booleano), `args` (lista de argumentos para native-image), `main_class` (opcional), `name` / `binary_name` (opcional), `reflection_config` (opcional) y `resources_config` (opcional).

#### Scenario: Carga exitosa de configuración GraalVM desde jolt.toml
- **WHEN** un manifiesto `jolt.toml` define `[graalvm]` con `args = ["--no-fallback"]`
- **THEN** el parser de Jolt deserializa correctamente las opciones en la estructura de manifiesto del proyecto

### Requirement: Detección del ejecutable native-image
El sistema SHALL buscar el binario `native-image` en las siguientes ubicaciones prioritarias:
1. Dentro del directorio `bin` del JDK/GraalVM detectado por el ToolchainManager.
2. En la variable de entorno `GRAALVM_HOME` / `JAVA_HOME`.
3. En el `PATH` del sistema.

#### Scenario: Detección del ejecutable native-image de GraalVM
- **WHEN** el usuario solicita una compilación nativa y `native-image` está disponible en el entorno o en el toolchain detectado
- **THEN** el sistema localiza la ruta absoluta al ejecutable y lo utiliza para invocar la compilación

### Requirement: Compilación de binario nativo con GraalVM
El sistema SHALL proveer la opción de compilar el proyecto a binario nativo invocando `native-image` con el fat JAR o classpath del proyecto y los argumentos configurados en `jolt.toml`.

#### Scenario: Compilación nativa exitosa
- **WHEN** el usuario ejecuta `jolt build --native`
- **THEN** el sistema compila las clases, empaqueta el jar y ejecuta `native-image` produciendo el ejecutable nativo en el directorio `dist/` o `target/`

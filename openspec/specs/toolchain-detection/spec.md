# toolchain-detection Specification

## Purpose
Proporciona detección inteligente de JDKs instalados en el sistema anfitrión (variables de entorno, rutas estándar y PATH), compatibilidad semántica de versiones y control explícito sobre la descarga de toolchains externas.

## Requirements

### Requirement: Detección multinivel de JDKs locales
El sistema SHALL inspeccionar ordenadamente las siguientes fuentes para descubrir instalaciones de Java locales:
1. Variables de entorno `GRAALVM_HOME` y `JAVA_HOME`.
2. Ejecutables `java` y `javac` accesibles en el `PATH` del sistema.
3. Directorios de instalación estándar del sistema operativo según la plataforma (en Windows: `C:\Program Files\Java`, `C:\Program Files\GraalVm`, `C:\Program Files\Eclipse Adoptium`, etc.; en Linux: `/usr/lib/jvm`, `/opt/java`; en macOS: `/Library/Java/JavaVirtualMachines`).

#### Scenario: Detección exitosa de GraalVM u OpenJDK en PATH o variables de entorno
- **WHEN** el usuario ejecuta un comando de compilación o ejecución (`jolt build`, `jolt run`) y cuenta con un JDK instalado en `PATH` o `JAVA_HOME`
- **THEN** el sistema detecta la ruta del binario `javac` y `java` y su versión mayor sin requerir descargas adicionales

### Requirement: Evaluación de compatibilidad de versiones de JDK
El sistema SHALL analizar la versión numérica mayor del JDK instalado contra la versión requerida (`java_version` en `jolt.toml` o por defecto) para determinar compatibilidad. Si la versión instalada es igual o compatible (superior y binariamente compatible), el sistema SHALL reutilizar el JDK instalado.

#### Scenario: JDK local compatible con la versión solicitada
- **WHEN** el proyecto solicita `java_version = "21"` y el sistema cuenta con Oracle GraalVM o OpenJDK 25 instalado
- **THEN** el sistema SHALL aceptar y utilizar el JDK 25 local indicando en consola la versión y vendor detectados

### Requirement: Control y notificación antes de descargar JDK
Si no se encuentra un JDK compatible en el sistema anfitrión ni en la caché local de Jolt, el sistema NO SHALL descargar un JDK automáticamente de forma silenciosa. El sistema SHALL notificar al usuario que no se encontró un JDK compatible y brindarle instrucciones para instalarlo o una opción/flag explícita para descargarlo.

#### Scenario: Notificación al usuario por falta de JDK compatible
- **WHEN** no existe un JDK instalado que satisfaga los requisitos del proyecto
- **THEN** el sistema finaliza con un mensaje de error claro indicando la versión requerida, las rutas inspeccionadas y la recomendación de instalación

# toolchain-detection Specification

## Purpose
Proporciona detección inteligente de JDKs instalados en el sistema anfitrión (variables de entorno, rutas estándar y PATH) y compatibilidad semántica de versiones sin administrar toolchains propios.

## Requirements

### Requirement: Detección multinivel de JDKs locales
El sistema SHALL inspeccionar fuentes locales conocidas para descubrir instalaciones de Java.

#### Scenario: Fuentes de detección local
- **WHEN** Jolt busca instalaciones de Java en el sistema anfitrión
- **THEN** inspecciona estas fuentes en orden:

1. Variables de entorno `GRAALVM_HOME` y `JAVA_HOME`.
2. Ejecutables `javac` accesibles en el `PATH` del sistema, validando también la presencia de `java`.
3. Directorios de instalación estándar del sistema operativo según la plataforma (en Windows: `C:\Program Files\Java`, `C:\Program Files\GraalVm`, `C:\Program Files\Eclipse Adoptium`, etc.; en Linux: `/usr/lib/jvm`, `/opt/java`; en macOS: `/Library/Java/JavaVirtualMachines`).

#### Scenario: Detección exitosa de GraalVM u OpenJDK en PATH o variables de entorno
- **WHEN** el usuario ejecuta un comando de compilación o ejecución (`jolt build`, `jolt run`) y cuenta con un JDK instalado en `PATH` o `JAVA_HOME`
- **THEN** el sistema detecta la ruta del binario `javac` y `java` y su versión mayor sin requerir descargas adicionales

### Requirement: Evaluación de compatibilidad de versiones de JDK
El sistema SHALL analizar la versión numérica mayor del JDK instalado contra la versión requerida (`java_version` en `jolt.toml` o por defecto) para determinar compatibilidad. Si la versión instalada es igual o compatible (superior y binariamente compatible), el sistema SHALL reutilizar el JDK instalado.

#### Scenario: JDK local compatible con la versión solicitada
- **WHEN** el proyecto solicita `java_version = "21"` y el sistema cuenta con Oracle GraalVM o OpenJDK 25 instalado
- **THEN** el sistema SHALL aceptar y utilizar el JDK 25 local indicando en consola la versión y vendor detectados

### Requirement: GraalVM como toolchain Java estándar
Una instalación GraalVM compatible SHALL poder utilizarse como cualquier otro JDK, mediante sus herramientas Java estándar (`java`, `javac`, `jar` y `jpackage`). Su detección no habilita compilación con `native-image`.

#### Scenario: Uso de GraalVM para compilar y empaquetar con jpackage
- **WHEN** el proyecto utiliza una instalación GraalVM compatible y solicita compilación o empaquetado con `jpackage`
- **THEN** Jolt utiliza las herramientas Java estándar del JDK detectado y no invoca `native-image`

### Requirement: Uso exclusivo de JDKs del sistema
Si no se encuentra un JDK compatible instalado en el sistema anfitrión, el sistema SHALL fallar con instrucciones para instalarlo o configurarlo. Jolt SHALL no descargar, extraer ni usar JDKs administrados por Jolt.

#### Scenario: Notificación al usuario por falta de JDK compatible
- **WHEN** no existe un JDK instalado que satisfaga los requisitos del proyecto
- **THEN** el sistema finaliza con un mensaje de error claro indicando la versión requerida, las rutas inspeccionadas y la recomendación de instalación

#### Scenario: No existe un JDK compatible
- **WHEN** no existe un JDK instalado que satisfaga los requisitos del proyecto
- **THEN** Jolt falla, no inicia una descarga y muestra cómo instalar o configurar un JDK del sistema

## Historial

- Actualizado por `openspec/changes/archive/2026-11-22-harden-installation-integrity/`.

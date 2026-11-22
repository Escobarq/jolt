# Spec Delta

## Purpose

Garantiza que Jolt no introduzca un JDK propio como sustituto silencioso del entorno del usuario y que la compilación use únicamente un JDK instalado y detectado en el sistema.

## ADDED Requirements

### Requirement: No descargar toolchains
Jolt SHALL no iniciar descargas de JDK ni extraer archivos de toolchains administrados por Jolt.

#### Scenario: Solicitud de descarga
- **WHEN** una operación intenta resolver un JDK que no está instalado en el sistema
- **THEN** Jolt informa que debe instalarse o configurarse un JDK del sistema y no realiza una petición de descarga

### Requirement: No usar toolchains administrados por Jolt
Jolt SHALL no seleccionar un JDK almacenado en la caché de Jolt para compilar, ejecutar o empaquetar proyectos.

#### Scenario: Caché de JDK existente
- **WHEN** existe un JDK bajo la caché local de Jolt pero también se resuelve el entorno de compilación
- **THEN** Jolt ignora la caché y usa el JDK compatible detectado en el sistema

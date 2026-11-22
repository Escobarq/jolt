# Spec Delta

## MODIFIED Requirements

### Requirement: Uso exclusivo de JDKs del sistema
Jolt SHALL usar únicamente un JDK compatible instalado en el sistema anfitrión. Si no se encuentra, SHALL finalizar con un error claro y no SHALL descargar, extraer ni usar un JDK administrado por Jolt.

#### Scenario: Notificación al usuario por falta de JDK compatible
- **WHEN** no existe un JDK instalado que satisfaga los requisitos del proyecto
- **THEN** el sistema finaliza con un mensaje de error claro indicando la versión requerida, las rutas inspeccionadas y la recomendación de instalación

#### Scenario: No existe un JDK compatible
- **WHEN** no existe un JDK instalado que satisfaga los requisitos del proyecto
- **THEN** Jolt falla, no inicia una descarga y muestra cómo instalar o configurar un JDK del sistema

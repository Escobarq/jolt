# Design

## Context

El flujo actual mezcla resolución, descarga, publicación en caché, enlace al proyecto y persistencia del lockfile en `src/main.rs`. Algunas rutas convierten errores en advertencias y varias escrituras publican directamente sobre el destino. Las descargas Maven y de JDK comparten la necesidad de timeout, validación y limpieza ante fallos. Véanse `proposal.md` y los cuatro deltas de especificación para el contrato observable.

## Goals / Non-Goals

**Goals:**

- Centralizar la validación de una dependencia bloqueada antes de enlazarla.
- Hacer que las publicaciones de caché y lockfile sean reemplazos completos, no escrituras parciales.
- Propagar errores de instalación y sincronización hasta la CLI.
- Usar exclusivamente un JDK instalado en el sistema y evitar descargas o toolchains administrados por Jolt.
- Permitir que `verify` espere un proceso legítimamente lento sin convertirlo en un fallo falso.
- Cubrir cada frontera con pruebas deterministas sin depender de Maven Central ni Adoptium.

**Non-Goals:**

- Implementar mediación completa de versiones Maven.
- Refactorizar `MavenClient` en varios servicios.
- Cambiar el formato público del manifiesto o del lockfile.
- Añadir firmas criptográficas propias cuando la fuente solo exponga checksum verificable.

## Decisions

### Validar el lockfile como contrato de entrada

En modo `--locked`, derivar las coordenadas requeridas del manifiesto y compararlas con el conjunto bloqueado, incluidos versión y clasificador. Para cada entrada, exigir checksum válido y comprobar el archivo antes de enlazarlo. Se rechaza la operación completa ante cualquier diferencia.

**Alternativa descartada:** confiar en que la presencia de una entrada en `jolt.lock` implica que sigue correspondiendo al manifiesto; no detecta cambios ni corrupción.

### Publicar mediante archivo temporal y reemplazo

Escribir el JAR o lockfile en un temporal del mismo directorio, finalizar y luego reemplazar el destino. Los temporales incompletos no serán nombres de artefacto válidos. La operación conservará el destino anterior si la escritura o el reemplazo falla.

**Alternativa descartada:** escribir directamente y borrar después; una terminación del proceso puede dejar un archivo que parezca válido.

### Propagar errores, sin recuperación silenciosa

Las operaciones de instalación y sync devolverán el primer error contextualizado y no guardarán un estado que describa una instalación completa. Las tareas no críticas de configuración IDE seguirán siendo separables, pero no ocultarán fallos de dependencias o lockfile.

**Alternativa descartada:** continuar tras advertencias; es la causa del falso éxito que este cambio corrige.

### Usar solamente el JDK del sistema

Resolver el toolchain únicamente mediante la detección del entorno anfitrión. La caché de Jolt y cualquier flujo de descarga quedan fuera de la selección; si no existe un JDK compatible, el comando falla con instrucciones para instalarlo o configurar `JAVA_HOME`/`PATH`.

**Alternativa descartada:** descargar o usar un JDK propio, porque oculta la configuración real del entorno y puede producir compilaciones con una herramienta distinta de la que el usuario administra.

### Sustituir la espera fija de `verify`

Mantener el proceso vivo hasta observar una señal de arranque válida o agotar un timeout configurable por el flujo actual, en vez de matar siempre a los 750 ms. El valor predeterminado debe conservar compatibilidad razonable, pero permitir aplicaciones lentas.

**Alternativa descartada:** aumentar arbitrariamente 750 ms; reduce falsos positivos solo para una clase de máquinas y mantiene la regla incorrecta.

### Pruebas aisladas de red y concurrencia

Usar directorios temporales, respuestas/archivos controlados y pruebas de fallo de escritura para validar checksum, preservación del destino anterior y ausencia de publicación parcial. Las pruebas de CLI comprobarán códigos de salida y mensajes, no servicios externos reales.

## Risks / Trade-offs

- [Cambio observable] Instalaciones que antes terminaban con advertencias ahora fallarán → documentar el cambio y comprobar que el error identifica la dependencia afectada.
- [Compatibilidad] Un lockfile antiguo o manual sin checksum válido puede dejar de aceptarse → exigir regeneración explícita en modo normal y rechazarlo en `--locked`.
- [Compatibilidad] Usuarios que dependían de la descarga administrada deberán instalar o configurar un JDK del sistema antes de usar Jolt.
- [Concurrencia] El reemplazo atómico puede fallar en plataformas o antivirus que bloqueen el destino → conservar el archivo anterior y devolver un error accionable.
- [Diagnóstico] Fallar al primer artefacto oculta errores posteriores → incluir coordenada, operación y causa en el mensaje; la siguiente ejecución puede descubrir el siguiente error.

## Migration Plan

1. Implementar validación y escritura segura detrás del comportamiento actual.
2. Ejecutar pruebas unitarias e integración de instalación, caché, timeout y toolchain.
3. Publicar la versión con la nueva semántica de error; no se requiere migración de formato.
4. Si una instalación existente contiene un lockfile inválido, ejecutar `jolt install` sin `--locked` para regenerarlo y revisar los cambios antes de usar `--locked`.
5. Rollback: volver al binario anterior; no se aplican migraciones destructivas y los temporales fallidos pueden eliminarse con seguridad.

# Tasks

## 1. Lockfile e instalación determinista

- [x] 1.1 Añadir comparación entre dependencias del manifiesto y `jolt.lock`, incluyendo versión y clasificador, y devolver error contextual en `--locked`; verificar con pruebas de manifiesto cambiado y lockfile compatible.
- [x] 1.2 Validar SHA-256 de cada artefacto bloqueado antes de enlazarlo y rechazar checksum ausente o incorrecto; verificar con pruebas de JAR alterado y JAR válido.
- [x] 1.3 Propagar errores de resolución, descarga, guardado y enlace en `install` y `sync`, evitando guardar un lockfile que describa una instalación incompleta; verificar códigos de salida y mensajes en pruebas de CLI.
- [x] 1.4 Hacer atómica la persistencia del lockfile y probar que un fallo de escritura conserva el archivo anterior; verificar con un directorio temporal y una ruta de destino no escribible.

## 2. Caché de artefactos

- [x] 2.1 Publicar JARs mediante temporal y reemplazo en el mismo directorio, sin exponer archivos parciales; verificar que la ruta final solo aparece después de completar la escritura.
- [x] 2.2 Integrar cálculo y comprobación de SHA-256 en la lectura/publicación de la caché y excluir temporales o archivos corruptos del enlace; verificar checksum correcto, incorrecto y archivo incompleto.
- [x] 2.3 Añadir una prueba de dos operaciones concurrentes sobre el mismo artefacto que confirme que nunca se lee un contenido parcial; verificar con hilos y directorio temporal.

## 3. Red y selección de JDK

- [x] 3.1 Configurar timeout explícito para peticiones Maven y convertir timeout en error contextual; verificar con un servidor de prueba que no responde.
- [x] 3.2 Eliminar la selección de JDKs almacenados en la caché de Jolt y resolver únicamente un JDK compatible del sistema; verificar que un JDK cacheado no se usa cuando existe uno del sistema.
- [x] 3.3 Eliminar la descarga y el uso de JDKs administrados por Jolt; devolver un error accionable cuando no exista un JDK compatible del sistema y verificar que no se realiza ninguna petición ni extracción.

## 4. Verificación y documentación operativa

- [x] 4.1 Sustituir la terminación fija de `verify` por la espera de arranque o timeout del flujo y cubrir una aplicación lenta; verificar que un proceso lento válido no se mata prematuramente.
- [x] 4.2 Documentar la regeneración de `jolt.lock` cuando `--locked` rechace un lockfile desactualizado y el cambio de errores antes tratados como advertencias; verificar que los comandos documentados coinciden con la CLI.
- [x] 4.3 Ejecutar `cargo fmt --check`, `cargo test` y una prueba de instalación/sync con dependencias controladas; corregir cualquier regresión y registrar el resultado.
  - Resultado: formato correcto, 48 pruebas exitosas y 1 fixture ignorado; la prueba controlada completó `install` y `sync` con un repositorio HTTP local.

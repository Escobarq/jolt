# Diseño de ramas estable y de desarrollo

## Intención

Mantener una rama estable que siempre pueda instalarse con Cargo y separar de
ella las implementaciones y pruebas en curso.

## Objetivos

- Conservar `master` como rama estable.
- Crear `dev` para integración, implementaciones y pruebas.
- Convertir `dev` en la rama predeterminada de GitHub.
- Ejecutar formato y pruebas automáticamente en cambios de `dev` y `master`.
- Proteger `master` para que futuros cambios lleguen mediante pull request y
  superen CI.
- Documentar la instalación estable desde `master`:

  ```powershell
  cargo install --git https://github.com/Escobarq/jolt --branch master --locked jolt
  ```

## No objetivos

- No renombrar `master` a `main`.
- No publicar `v0.7.4` ni crear una GitHub Release.
- No generar instaladores NSIS o WiX.
- No integrar UPX, SignPath ni firma de código.
- No cambiar la versión actual de Jolt.

## Enfoques considerados

1. **`master` estable y predeterminada:** simple, pero facilita que nuevos
   cambios apunten accidentalmente a la rama estable.
2. **`master` estable y `dev` predeterminada:** separa claramente desarrollo y
   publicación, manteniendo una URL estable para `cargo install`.
3. **Renombrar a `main`:** no aporta comportamiento y obliga a migrar
   referencias existentes.

Se eligió el segundo enfoque.

## Flujo

1. Hacer un único commit de bootstrap en `master` con la documentación y CI.
2. Verificar y subir ese commit.
3. Crear `dev` desde el `master` verificado y subirla.
4. Cambiar la rama predeterminada de GitHub a `dev`.
5. Crear una regla para `master` que impida borrado y force-push, exija pull
   request y requiera el resultado exitoso de CI.
6. Trabajar en ramas de característica y dirigir sus pull requests a `dev`.
7. Promover una versión estable mediante pull request de `dev` a `master`.

## CI

Un workflow de GitHub Actions para Rust ejecutará:

- `cargo fmt --all -- --check`
- `cargo test --locked`

Se activará en pushes y pull requests dirigidos a `dev` o `master`. La regla
de `master` exigirá el job de pruebas, sin requerir aprobación de otra persona,
para no bloquear un repositorio mantenido por una sola persona.

## Riesgos

- Como `dev` será la rama predeterminada, un clon normal quedará en desarrollo.
  La documentación y los usuarios deberán usar explícitamente `master` para la
  instalación estable.
- Una regla de protección configurada con un nombre de check incorrecto podría
  bloquear promociones; se verificará el nombre real del job antes de exigirlo.

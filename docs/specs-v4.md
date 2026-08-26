# Jolt - Especificaciones Tecnicas Fase 4

Este documento detalla la arquitectura y diseño tecnico para las funcionalidades de la Fase 4:
1. **Empaquetado Binario Nativo Multiplataforma con `jpackage` (`jolt package` / `jolt build --package`)**
2. **Configuracion de Empaquetado en `jolt.toml` (`[package]`)**
3. **Deteccion Automatica de Clase Principal (`detect_main_class`)**

---

## 1. Modulo Q: Empaquetado Binario Nativo Multiplataforma (`jolt package`)

**Responsabilidad:** Generar aplicaciones autocontenidas y lanzadores binarios nativos ejecutables sin requerir que el usuario final tenga Java instalado, utilizando la herramienta estándar `jpackage` del JDK (Java 14+).

### Capacidades:
- **Lanzadores portátiles (`app-image`)**:
  - **Linux**: Directorio autocontenido con ejecutable en `dist/<app>/bin/<app>` y runtime minimalista integrado.
  - **Windows**: Directorio con lanzador ejecutable nativo `dist/<app>/<app>.exe`.
  - **macOS**: Bundle de aplicación `dist/<app>.app`.
- **Instaladores de sistema**:
  - **Linux**: Paquetes `.deb` y `.rpm`.
  - **Windows**: Paquetes instalables `.msi` y `.exe`.
  - **macOS**: Instaladores `.dmg` y `.pkg`.

### Tareas implementadas:
- [x] Extensión del manifiesto en `src/manifest.rs`: sección `[package]` y `[project].main_class`.
- [x] Subcomando `Package` (alias `pkg`, `bundle`) y flag `--package` (`-p`) en `src/cli.rs`.
- [x] Integración de `jpackage` en `src/toolchain.rs` (`Toolchain.jpackage_bin`).
- [x] Motor de empaquetado nativo `BuildEngine::package_native_app` en `src/engine.rs`.
- [x] Detección estática automática de la clase principal `BuildEngine::detect_main_class`.
- [x] Diagnóstico de `jpackage` y formatos soportados en `src/checker.rs` (`jolt check`).
- [x] Tests unitarios en `manifest.rs`, `engine.rs`, `toolchain.rs`.

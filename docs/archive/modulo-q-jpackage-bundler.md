# Modulo Q: Empaquetado Binario Nativo Multiplataforma (jpackage)

**Responsabilidad:** Generar aplicaciones autocontenidas y lanzadores binarios nativos ejecutables para Linux, Windows y macOS sin requerir Java instalado en el sistema destino, integrando `jpackage`.

### Funcionalidades implementadas:
- Formatos portables (`app-image`) e instaladores del sistema (`.deb`, `.rpm`, `.msi`, `.exe`, `.dmg`, `.pkg`).
- Detección estática inteligente de la clase principal (`BuildEngine::detect_main_class`).
- Tabla `[package]` en `jolt.toml` con opciones para vendor, icon, dest y java_options.
- Subcomando `jolt package` y flag `jolt build --package`.

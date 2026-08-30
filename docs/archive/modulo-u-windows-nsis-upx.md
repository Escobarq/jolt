# Módulo U: Empaquetado en Windows con UPX y NSIS (Windows Installer)

**Responsabilidad:** Generar instaladores nativos completos para Windows (`.exe` basados en NSIS con interfaz MUI2) y optimizar el tamaño de los ejecutables y librerías dinámicas mediante compresión UPX, integrando soporte para variable de entorno `PATH`, accesos directos y desinstalador limpio.

### Funcionalidades implementadas:
- **Compresión con UPX (`upx.exe`):** Reducción de hasta un 70%+ del peso de binarios (`.exe`) y librerías (`.dll`) antes del empaquetado.
- **Generador de Instaladores NSIS (`makensis.exe`):**
  - Generación de instaladores nativos modernos con Modern UI 2 (MUI2) y soporte multilingüe (español e inglés).
  - Configuración automática de la variable de entorno `PATH` con notificación en tiempo real a Windows (`WM_SETTINGCHANGE`).
  - Creación de accesos directos en Menú Inicio y Escritorio.
  - Soporte para ámbito de instalación `per-user` (`%LOCALAPPDATA%\Programs\`, sin requerir permisos de administrador / UAC) y `per-machine` (`C:\Program Files`).
  - Desinstalador limpio (`uninstall.exe`) que elimina archivos, accesos directos, claves de registro y remueve la ruta de `PATH`.
  - Registro completo en *Configuración > Aplicaciones instaladas* de Windows.
- **Configuración en `jolt.toml` (`[package.windows]`):**
  - Opciones para `upx`, `upx_args`, `installer`, `add_to_path`, `desktop_shortcut`, `start_menu`, `scope` y `license`.
- **Integración CLI:**
  - `jolt build --installer` para compilar y generar el instalador en un solo paso con progreso paso a paso en terminal (`[1/5]` a `[5/5]`).
  - `jolt package --type nsis` con flags `--upx`, `--no-upx`, `--add-to-path` y `--scope`.
- **Auditoría de Entorno (`jolt check`):** Detección automática y diagnóstico de `makensis` (NSIS) y `upx`.
- **Instalador Oficial de Jolt CLI:** Script `scripts/build-windows-installer.ps1` y plantilla `installer/windows/jolt_installer.nsi` para generar el instalador oficial de Jolt (`jolt-setup.exe`).

# Modulo T: Workspaces Multimodulo y Dependencias Locales

**Responsabilidad:** Orquestar repositorios con múltiples submódulos heterogéneos y dependencias inter-módulo.

### Funcionalidades implementadas:
- Sección `[workspace]` en la raíz con lista de miembros.
- Dependencias locales por ruta (`path dependencies`: `"core" = { path = "../core" }`).
- Resolución automática de classpath y compilación encadenada de dependencias locales.
- Fusión de dependencias locales en Fat-JARs y empaquetados binarios nativos.
- Comandos centralizados desde la raíz con flags `--member` / `-p` y `--all`.

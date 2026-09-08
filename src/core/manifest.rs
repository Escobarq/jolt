use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, value, Item};

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct Workspace {
    #[serde(default)]
    pub members: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct JoltManifest {
    pub workspace: Option<Workspace>,
    pub project: Option<Project>,
    pub package: Option<PackageConfig>,
    pub graalvm: Option<GraalVmConfig>,
    #[serde(default)]
    pub dependencies: Option<HashMap<String, toml::Value>>,
    #[serde(rename = "dev-dependencies", default)]
    pub dev_dependencies: Option<HashMap<String, toml::Value>>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub java_version: Option<String>,
    pub main_class: Option<String>,
    pub package: Option<String>,
    pub group_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub struct GraalVmConfig {
    pub enabled: Option<bool>,
    pub args: Option<Vec<String>>,
    pub main_class: Option<String>,
    pub name: Option<String>,
    pub reflection_config: Option<String>,
    pub resources_config: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct WindowsPackageConfig {
    pub upx: Option<bool>,
    pub upx_args: Option<Vec<String>>,
    pub installer: Option<String>,
    pub add_to_path: Option<bool>,
    pub desktop_shortcut: Option<bool>,
    pub start_menu: Option<bool>,
    pub scope: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct PackageConfig {
    pub r#type: Option<String>,
    pub name: Option<String>,
    pub main_class: Option<String>,
    pub icon: Option<String>,
    pub vendor: Option<String>,
    pub description: Option<String>,
    pub copyright: Option<String>,
    pub dest: Option<String>,
    pub java_options: Option<Vec<String>>,
    pub windows: Option<WindowsPackageConfig>,
    pub graalvm: Option<GraalVmConfig>,
}

impl JoltManifest {
    #[allow(dead_code)]
    pub fn parse(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Retorna `true` si el manifiesto define un workspace multimódulo
    pub fn is_workspace(&self) -> bool {
        self.workspace.is_some()
    }

    /// Retorna la configuración de GraalVM (a nivel raíz o dentro de package)
    pub fn graalvm_config(&self) -> Option<&GraalVmConfig> {
        self.graalvm.as_ref().or_else(|| self.package.as_ref().and_then(|p| p.graalvm.as_ref()))
    }

    /// Extrae la versión y/o la ruta local de una especificación de dependencia TOML
    pub fn parse_dependency_spec(val: &toml::Value) -> (Option<String>, Option<String>) {
        match val {
            toml::Value::String(s) => (Some(s.clone()), None),
            toml::Value::Table(t) => {
                let version = t.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());
                let path = t.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                (version, path)
            }
            _ => (None, None),
        }
    }

    /// Busca hacia arriba en la jerarquía de directorios si existe un workspace raíz
    pub fn find_root_workspace(start_dir: &Path) -> Option<(std::path::PathBuf, JoltManifest)> {
        let mut curr = if start_dir.is_relative() {
            std::env::current_dir().unwrap_or_else(|_| start_dir.to_path_buf()).join(start_dir)
        } else {
            start_dir.to_path_buf()
        };

        if let Ok(canon) = curr.canonicalize() {
            curr = canon;
        }

        loop {
            let manifest_path = curr.join("jolt.toml");
            if manifest_path.exists() {
                if let Ok(manifest) = Self::load_from_file(&manifest_path) {
                    if manifest.is_workspace() {
                        return Some((curr, manifest));
                    }
                }
            }

            if let Some(parent) = curr.parent() {
                if parent == curr {
                    break;
                }
                curr = parent.to_path_buf();
            } else {
                break;
            }
        }
        None
    }

    /// Añade un miembro a la lista `[workspace].members` de un archivo jolt.toml conservando formato
    pub fn add_member_to_workspace(
        workspace_manifest: &Path,
        member_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = if workspace_manifest.exists() {
            fs::read_to_string(workspace_manifest)?
        } else {
            "[workspace]\nmembers = []\n".to_string()
        };

        let mut doc = content.parse::<DocumentMut>()?;

        if !doc.contains_key("workspace") {
            doc["workspace"] = Item::Table(toml_edit::Table::new());
        }

        if let Some(ws) = doc.get_mut("workspace").and_then(|w| w.as_table_mut()) {
            if !ws.contains_key("members") {
                ws["members"] = Item::Value(toml_edit::Value::Array(toml_edit::Array::new()));
            }

            if let Some(members) = ws.get_mut("members").and_then(|m| m.as_array_mut()) {
                let already_exists = members.iter().any(|v| v.as_str() == Some(member_name));
                if !already_exists {
                    members.push(member_name);
                }
            }
        }

        fs::write(workspace_manifest, doc.to_string())?;
        Ok(())
    }

    /// Crea un archivo `jolt.toml` para un workspace monorepo
    pub fn create_workspace_file(manifest_path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = r#"[workspace]
members = []
"#;
        fs::write(manifest_path, content)?;
        Ok(())
    }

    /// Añade o actualiza una dependencia en el archivo jolt.toml conservando formato y comentarios
    pub fn add_dependency_to_file(
        manifest_path: &Path,
        group_artifact: &str,
        version: &str,
        is_dev: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(manifest_path)?;
        let mut doc = content.parse::<DocumentMut>()?;

        let table_key = if is_dev { "dev-dependencies" } else { "dependencies" };

        if !doc.contains_key(table_key) {
            doc[table_key] = Item::Table(toml_edit::Table::new());
        }

        if let Item::Table(ref mut deps) = doc[table_key] {
            deps[group_artifact] = value(version);
        }

        fs::write(manifest_path, doc.to_string())?;
        Ok(())
    }

    /// Remueve una dependencia de jolt.toml conservando formato y comentarios
    pub fn remove_dependency_from_file(manifest_path: &Path, group_artifact: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(manifest_path)?;
        let mut doc = content.parse::<DocumentMut>()?;

        let mut removed = false;

        if let Some(Item::Table(deps)) = doc.get_mut("dependencies") {
            if deps.remove(group_artifact).is_some() {
                removed = true;
            }
        }

        if let Some(Item::Table(dev_deps)) = doc.get_mut("dev-dependencies") {
            if dev_deps.remove(group_artifact).is_some() {
                removed = true;
            }
        }

        if removed {
            fs::write(manifest_path, doc.to_string())?;
        }

        Ok(removed)
    }

    /// Carga y parsea el archivo jolt.toml
    pub fn load_from_file(manifest_path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(manifest_path)?;
        let manifest: JoltManifest = toml::from_str(&content)?;
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_manifest() {
        let toml_content = r#"
        [project]
        name = "mi-app"
        version = "1.0.0"
        java_version = "21"

        [dependencies]
        "org.springframework.boot:spring-boot-starter-web" = "3.2.0"
        "com.google.guava:guava" = "33.0.0-jre"

        [dev-dependencies]
        "org.junit.jupiter:junit-jupiter" = "5.10.1"
        "#;

        let manifest = JoltManifest::parse(toml_content).expect("Failed to parse toml");
        let proj = manifest.project.expect("Missing project");

        assert_eq!(proj.name, "mi-app");
        assert_eq!(proj.version, "1.0.0");
        assert_eq!(proj.java_version, Some("21".to_string()));

        let deps = manifest.dependencies.expect("Missing dependencies");
        let (guava_ver, _) = JoltManifest::parse_dependency_spec(deps.get("com.google.guava:guava").unwrap());
        assert_eq!(guava_ver, Some("33.0.0-jre".to_string()));

        let (spring_ver, _) = JoltManifest::parse_dependency_spec(deps.get("org.springframework.boot:spring-boot-starter-web").unwrap());
        assert_eq!(spring_ver, Some("3.2.0".to_string()));

        let dev_deps = manifest.dev_dependencies.expect("Missing dev-dependencies");
        let (junit_ver, _) = JoltManifest::parse_dependency_spec(dev_deps.get("org.junit.jupiter:junit-jupiter").unwrap());
        assert_eq!(junit_ver, Some("5.10.1".to_string()));
    }

    #[test]
    fn test_parse_manifest_without_dependencies() {
        let toml_content = r#"
        [project]
        name = "simple-app"
        version = "0.1.0"
        "#;

        let manifest = JoltManifest::parse(toml_content).expect("Failed to parse toml");
        let proj = manifest.project.expect("Missing project");

        assert_eq!(proj.name, "simple-app");
        assert_eq!(proj.version, "0.1.0");
        assert_eq!(proj.java_version, None);
        assert!(manifest.dependencies.is_none());
        assert!(manifest.dev_dependencies.is_none());
    }

    #[test]
    fn test_add_and_remove_dev_dependency() {
        let temp_dir = std::env::temp_dir().join("jolt_manifest_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let manifest_file = temp_dir.join("jolt.toml");

        let initial_toml = r#"[project]
name = "test-pkg"
version = "0.1.0"
"#;
        fs::write(&manifest_file, initial_toml).unwrap();

        // Add regular dependency
        JoltManifest::add_dependency_to_file(&manifest_file, "com.google.guava:guava", "33.0.0-jre", false).unwrap();
        // Add dev dependency
        JoltManifest::add_dependency_to_file(&manifest_file, "org.junit.jupiter:junit-jupiter-api", "5.10.2", true).unwrap();

        let manifest = JoltManifest::load_from_file(&manifest_file).unwrap();
        let deps = manifest.dependencies.expect("expected dependencies");
        let (guava_ver, _) = JoltManifest::parse_dependency_spec(deps.get("com.google.guava:guava").unwrap());
        assert_eq!(guava_ver, Some("33.0.0-jre".to_string()));

        let dev_deps = manifest.dev_dependencies.expect("expected dev-dependencies");
        let (junit_ver, _) = JoltManifest::parse_dependency_spec(dev_deps.get("org.junit.jupiter:junit-jupiter-api").unwrap());
        assert_eq!(junit_ver, Some("5.10.2".to_string()));

        // Remove dev dependency
        let removed = JoltManifest::remove_dependency_from_file(&manifest_file, "org.junit.jupiter:junit-jupiter-api").unwrap();
        assert!(removed);

        let manifest_after = JoltManifest::load_from_file(&manifest_file).unwrap();
        assert!(manifest_after.dev_dependencies.as_ref().map(|d| d.is_empty()).unwrap_or(true));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_parse_manifest_with_package_config() {
        let toml_content = r#"
        [project]
        name = "desktop-app"
        version = "1.2.0"
        java_version = "21"
        main_class = "com.example.Main"

        [package]
        type = "app-image"
        name = "DesktopApp"
        vendor = "Acme Corp"
        description = "A native desktop app"
        icon = "src/main/resources/icon.png"
        dest = "dist"
        java_options = ["-Xmx512m", "-Dfile.encoding=UTF-8"]
        "#;

        let manifest = JoltManifest::parse(toml_content).expect("Failed to parse toml");
        let proj = manifest.project.expect("Missing project");
        assert_eq!(proj.name, "desktop-app");
        assert_eq!(proj.main_class, Some("com.example.Main".to_string()));

        let pkg = manifest.package.expect("Expected package configuration");
        assert_eq!(pkg.r#type, Some("app-image".to_string()));
        assert_eq!(pkg.name, Some("DesktopApp".to_string()));
        assert_eq!(pkg.vendor, Some("Acme Corp".to_string()));
        assert_eq!(pkg.description, Some("A native desktop app".to_string()));
        assert_eq!(pkg.dest, Some("dist".to_string()));
        assert_eq!(
            pkg.java_options,
            Some(vec!["-Xmx512m".to_string(), "-Dfile.encoding=UTF-8".to_string()])
        );
    }

    #[test]
    fn test_parse_manifest_with_windows_package_config() {
        let toml_content = r#"
        [project]
        name = "my-tool"
        version = "2.0.0"

        [package]
        type = "nsis"
        name = "MyTool"

        [package.windows]
        upx = true
        upx_args = ["--best", "--lzma"]
        installer = "nsis"
        add_to_path = true
        desktop_shortcut = true
        start_menu = true
        scope = "per-user"
        license = "LICENSE"
        "#;

        let manifest = JoltManifest::parse(toml_content).expect("Failed to parse toml");
        let pkg = manifest.package.expect("Expected package configuration");
        assert_eq!(pkg.r#type, Some("nsis".to_string()));
        let win = pkg.windows.expect("Expected windows configuration");
        assert_eq!(win.upx, Some(true));
        assert_eq!(win.upx_args, Some(vec!["--best".to_string(), "--lzma".to_string()]));
        assert_eq!(win.installer, Some("nsis".to_string()));
        assert_eq!(win.add_to_path, Some(true));
        assert_eq!(win.desktop_shortcut, Some(true));
        assert_eq!(win.start_menu, Some(true));
        assert_eq!(win.scope, Some("per-user".to_string()));
        assert_eq!(win.license, Some("LICENSE".to_string()));
    }

    #[test]
    fn test_parse_workspace_manifest() {
        let toml_content = r#"
        [workspace]
        members = [
            "core",
            "desktop-app",
            "web-api"
        ]
        "#;

        let manifest = JoltManifest::parse(toml_content).expect("Failed to parse workspace toml");
        assert!(manifest.is_workspace());
        let ws = manifest.workspace.unwrap();
        assert_eq!(ws.members, vec!["core", "desktop-app", "web-api"]);
        assert!(manifest.project.is_none());
    }

    #[test]
    fn test_parse_dependency_with_path() {
        let toml_content = r#"
        [project]
        name = "desktop-ui"
        version = "0.1.0"
        package = "org.equipo.desktop"
        group_id = "org.equipo"

        [dependencies]
        "org.openjfx:javafx-controls" = "21.0.2:linux"
        "core" = { path = "../core", version = "0.1.0" }
        "#;

        let manifest = JoltManifest::parse(toml_content).expect("Failed to parse toml");
        let proj = manifest.project.unwrap();
        assert_eq!(proj.package, Some("org.equipo.desktop".to_string()));
        assert_eq!(proj.group_id, Some("org.equipo".to_string()));

        let deps = manifest.dependencies.unwrap();
        let jfx = deps.get("org.openjfx:javafx-controls").unwrap();
        let (jfx_ver, jfx_path) = JoltManifest::parse_dependency_spec(jfx);
        assert_eq!(jfx_ver, Some("21.0.2:linux".to_string()));
        assert_eq!(jfx_path, None);

        let core = deps.get("core").unwrap();
        let (core_ver, core_path) = JoltManifest::parse_dependency_spec(core);
        assert_eq!(core_ver, Some("0.1.0".to_string()));
        assert_eq!(core_path, Some("../core".to_string()));
    }

    #[test]
    fn test_parse_graalvm_config() {
        let toml_content = r#"
        [project]
        name = "native-app"
        version = "0.1.0"

        [graalvm]
        enabled = true
        main_class = "com.example.Main"
        name = "native-app-bin"
        args = ["--no-fallback", "-H:+ReportExceptionStackTraces"]
        reflection_config = "reflect-config.json"
        resources_config = "resource-config.json"
        "#;

        let manifest = JoltManifest::parse(toml_content).expect("Failed to parse toml with graalvm");
        let gvm = manifest.graalvm_config().expect("Expected graalvm config");
        assert_eq!(gvm.enabled, Some(true));
        assert_eq!(gvm.main_class, Some("com.example.Main".to_string()));
        assert_eq!(gvm.name, Some("native-app-bin".to_string()));
        assert_eq!(gvm.args, Some(vec!["--no-fallback".to_string(), "-H:+ReportExceptionStackTraces".to_string()]));
        assert_eq!(gvm.reflection_config, Some("reflect-config.json".to_string()));
        assert_eq!(gvm.resources_config, Some("resource-config.json".to_string()));
    }
}


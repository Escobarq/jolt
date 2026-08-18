use serde::Deserialize;
use std::collections::HashMap;

use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, value, Item};

#[derive(Debug, Deserialize, PartialEq)]
pub struct JoltManifest {
    pub project: Project,
    #[serde(default)]
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "dev-dependencies", default)]
    pub dev_dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub java_version: Option<String>,
}

impl JoltManifest {
    #[allow(dead_code)]
    pub fn parse(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
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

        assert_eq!(manifest.project.name, "mi-app");
        assert_eq!(manifest.project.version, "1.0.0");
        assert_eq!(manifest.project.java_version, Some("21".to_string()));

        let deps = manifest.dependencies.expect("Missing dependencies");
        assert_eq!(deps.get("com.google.guava:guava").unwrap(), "33.0.0-jre");
        assert_eq!(deps.get("org.springframework.boot:spring-boot-starter-web").unwrap(), "3.2.0");

        let dev_deps = manifest.dev_dependencies.expect("Missing dev-dependencies");
        assert_eq!(dev_deps.get("org.junit.jupiter:junit-jupiter").unwrap(), "5.10.1");
    }

    #[test]
    fn test_parse_manifest_without_dependencies() {
        let toml_content = r#"
        [project]
        name = "simple-app"
        version = "0.1.0"
        "#;

        let manifest = JoltManifest::parse(toml_content).expect("Failed to parse toml");

        assert_eq!(manifest.project.name, "simple-app");
        assert_eq!(manifest.project.version, "0.1.0");
        assert_eq!(manifest.project.java_version, None);
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
        assert_eq!(deps.get("com.google.guava:guava").unwrap(), "33.0.0-jre");

        let dev_deps = manifest.dev_dependencies.expect("expected dev-dependencies");
        assert_eq!(dev_deps.get("org.junit.jupiter:junit-jupiter-api").unwrap(), "5.10.2");

        // Remove dev dependency
        let removed = JoltManifest::remove_dependency_from_file(&manifest_file, "org.junit.jupiter:junit-jupiter-api").unwrap();
        assert!(removed);

        let manifest_after = JoltManifest::load_from_file(&manifest_file).unwrap();
        assert!(manifest_after.dev_dependencies.as_ref().map(|d| d.is_empty()).unwrap_or(true));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}


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
    pub fn add_dependency_to_file(manifest_path: &Path, group_artifact: &str, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(manifest_path)?;
        let mut doc = content.parse::<DocumentMut>()?;

        if !doc.contains_key("dependencies") {
            doc["dependencies"] = Item::Table(toml_edit::Table::new());
        }

        if let Item::Table(ref mut deps) = doc["dependencies"] {
            deps[group_artifact] = value(version);
        }

        fs::write(manifest_path, doc.to_string())?;
        Ok(())
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
}

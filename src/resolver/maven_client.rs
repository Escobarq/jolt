use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;

use super::{Dependency, DependencyNode, SearchResultItem};
use super::pom_parser::parse_pom_dependencies;

#[derive(Deserialize, Debug)]
struct MavenSearchResponse {
    response: MavenSearchDocs,
}

#[derive(Deserialize, Debug)]
struct MavenSearchDocs {
    #[serde(default)]
    docs: Vec<MavenDoc>,
}

#[derive(Deserialize, Debug)]
struct MavenDoc {
    #[serde(default)]
    g: Option<String>,
    #[serde(default)]
    a: Option<String>,
    #[serde(rename = "latestVersion", default)]
    latest_version: Option<String>,
    #[serde(default)]
    v: Option<String>,
    #[serde(default)]
    p: Option<String>,
    #[serde(rename = "versionCount", default)]
    version_count: Option<u32>,
}

#[derive(Clone)]
pub struct MavenClient {
    client: reqwest::Client,
    search_base_url: String,
    repo_base_url: String,
}

impl Default for MavenClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MavenClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("jolt-package-manager/0.2.0")
            .build()
            .unwrap_or_default();

        Self {
            client,
            search_base_url: "https://search.maven.org/solrsearch/select".to_string(),
            repo_base_url: "https://repo1.maven.org/maven2".to_string(),
        }
    }

    /// Busca la última versión disponible para un grupo y artefacto en Maven Central
    pub async fn fetch_latest_version(
        &self,
        group_id: &str,
        artifact_id: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "{}?q=g:\"{}\"+AND+a:\"{}\"&rows=1&wt=json",
            self.search_base_url, group_id, artifact_id
        );

        let resp: MavenSearchResponse = self.client.get(&url).send().await?.json().await?;

        if let Some(doc) = resp.response.docs.first() {
            if let Some(ref ver) = doc.latest_version {
                return Ok(ver.clone());
            }
            if let Some(ref ver) = doc.v {
                return Ok(ver.clone());
            }
        }

        Err(format!(
            "No se encontró la dependencia '{}:{}' en Maven Central",
            group_id, artifact_id
        )
        .into())
    }

    /// Busca paquetes en Maven Central basados en una consulta de texto libre
    pub async fn search_packages(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResultItem>, Box<dyn Error + Send + Sync>> {
        let clean_query = query.trim().replace(' ', "+");
        let url = format!(
            "{}?q={}&rows={}&wt=json",
            self.search_base_url, clean_query, limit
        );

        let resp: MavenSearchResponse = self.client.get(&url).send().await?.json().await?;

        let mut results = Vec::new();
        for doc in resp.response.docs {
            if let (Some(group_id), Some(artifact_id)) = (doc.g, doc.a) {
                let version = doc.latest_version.or(doc.v).unwrap_or_else(|| "latest".to_string());
                results.push(SearchResultItem {
                    group_id,
                    artifact_id,
                    version,
                    package_type: doc.p,
                    version_count: doc.version_count,
                });
            }
        }

        Ok(results)
    }

    /// Obtiene el contenido del archivo POM desde el repositorio Maven
    pub async fn fetch_pom(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let group_path = group_id.replace('.', "/");
        let url = format!(
            "{}/{}/{}/{}/{}-{}.pom",
            self.repo_base_url, group_path, artifact_id, version, artifact_id, version
        );

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(format!(
                "Error al descargar POM para '{}:{}:{}' (Status: {})",
                group_id,
                artifact_id,
                version,
                response.status()
            )
            .into());
        }

        let body = response.text().await?;
        Ok(body)
    }

    /// Descarga el binario JAR desde el repositorio Maven
    #[allow(dead_code)]
    pub async fn download_jar(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        self.download_jar_with_classifier(group_id, artifact_id, version, None).await
    }

    /// Descarga el binario JAR con clasificador de plataforma opcional (ej. linux, mac, win)
    pub async fn download_jar_with_classifier(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        classifier: Option<&str>,
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let group_path = group_id.replace('.', "/");
        let file_name = match classifier {
            Some(c) if !c.is_empty() => format!("{}-{}-{}.jar", artifact_id, version, c),
            _ => format!("{}-{}.jar", artifact_id, version),
        };

        let url = format!(
            "{}/{}/{}/{}/{}",
            self.repo_base_url, group_path, artifact_id, version, file_name
        );

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(format!(
                "Error al descargar JAR para '{}:{}:{}{:?}' (Status: {})",
                group_id,
                artifact_id,
                version,
                classifier,
                response.status()
            )
            .into());
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Parsea las dependencias declaradas en un archivo POM en formato XML
    pub fn parse_pom_dependencies(
        xml_content: &str,
    ) -> Result<Vec<Dependency>, Box<dyn Error + Send + Sync>> {
        parse_pom_dependencies(xml_content)
    }

    /// Construye el árbol de dependencias transitivas (hasta 1 nivel de profundidad)
    pub async fn fetch_dependency_tree(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<DependencyNode, Box<dyn Error + Send + Sync>> {
        let pom = self.fetch_pom(group_id, artifact_id, version).await?;
        let deps = parse_pom_dependencies(&pom)?;

        let mut child_nodes = Vec::new();
        let mut visited = HashSet::new();

        for dep in deps {
            // Ignorar dependencias con scope test o opcionales por defecto
            if let Some(ref scope) = dep.scope {
                if scope == "test" || scope == "provided" {
                    continue;
                }
            }

            if !dep.version.is_empty() && !dep.version.starts_with('$') {
                let key = format!("{}:{}", dep.group_id, dep.artifact_id);
                if !visited.contains(&key) {
                    visited.insert(key);
                    child_nodes.push(DependencyNode {
                        group_id: dep.group_id,
                        artifact_id: dep.artifact_id,
                        version: dep.version,
                        dependencies: Vec::new(),
                    });
                }
            }
        }

        Ok(DependencyNode {
            group_id: group_id.to_string(),
            artifact_id: artifact_id.to_string(),
            version: version.to_string(),
            dependencies: child_nodes,
        })
    }
}

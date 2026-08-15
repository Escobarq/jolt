use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyNode {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub dependencies: Vec<DependencyNode>,
}

#[derive(Deserialize, Debug)]
struct MavenSearchResponse {
    response: MavenSearchDocs,
}

#[derive(Deserialize, Debug)]
struct MavenSearchDocs {
    docs: Vec<MavenDoc>,
}

#[derive(Deserialize, Debug)]
struct MavenDoc {
    #[serde(rename = "latestVersion", default)]
    latest_version: Option<String>,
    #[serde(default)]
    v: Option<String>,
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
            .user_agent("jolt-package-manager/0.1.0")
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
    pub async fn download_jar(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let group_path = group_id.replace('.', "/");
        let url = format!(
            "{}/{}/{}/{}/{}-{}.jar",
            self.repo_base_url, group_path, artifact_id, version, artifact_id, version
        );

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(format!(
                "Error al descargar JAR para '{}:{}:{}' (Status: {})",
                group_id,
                artifact_id,
                version,
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
        use quick_xml::events::Event;
        use quick_xml::reader::Reader;

        let mut reader = Reader::from_str(xml_content);
        reader.config_mut().trim_text(true);

        let mut dependencies = Vec::new();
        let mut in_dependencies = false;
        let mut in_dependency = false;
        let mut current_tag = String::new();

        let mut curr_group = String::new();
        let mut curr_artifact = String::new();
        let mut curr_version = String::new();
        let mut curr_scope = None;

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "dependencies" {
                        in_dependencies = true;
                    } else if in_dependencies && name == "dependency" {
                        in_dependency = true;
                        curr_group.clear();
                        curr_artifact.clear();
                        curr_version.clear();
                        curr_scope = None;
                    }
                    current_tag = name;
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "dependencies" {
                        in_dependencies = false;
                    } else if in_dependencies && name == "dependency" {
                        in_dependency = false;
                        if !curr_group.is_empty() && !curr_artifact.is_empty() {
                            dependencies.push(Dependency {
                                group_id: curr_group.clone(),
                                artifact_id: curr_artifact.clone(),
                                version: curr_version.clone(),
                                scope: curr_scope.clone(),
                            });
                        }
                    }
                    current_tag.clear();
                }
                Ok(Event::Text(e)) => {
                    if in_dependency {
                        let text = String::from_utf8_lossy(e.as_ref()).trim().to_string();
                        match current_tag.as_str() {
                            "groupId" => curr_group = text,
                            "artifactId" => curr_artifact = text,
                            "version" => curr_version = text,
                            "scope" => curr_scope = Some(text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Box::new(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(dependencies)
    }

    /// Construye el árbol de dependencias transitivas (hasta 1 nivel de profundidad para evitar recursión infinita)
    pub async fn fetch_dependency_tree(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<DependencyNode, Box<dyn Error + Send + Sync>> {
        let pom = self.fetch_pom(group_id, artifact_id, version).await?;
        let deps = Self::parse_pom_dependencies(&pom)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pom_xml() {
        let sample_pom = r#"
        <project xmlns="http://maven.apache.org/POM/4.0.0">
            <modelVersion>4.0.0</modelVersion>
            <groupId>com.example</groupId>
            <artifactId>sample-project</artifactId>
            <version>1.0.0</version>
            <dependencies>
                <dependency>
                    <groupId>com.google.guava</groupId>
                    <artifactId>guava</artifactId>
                    <version>33.0.0-jre</version>
                </dependency>
                <dependency>
                    <groupId>org.junit.jupiter</groupId>
                    <artifactId>junit-jupiter</artifactId>
                    <version>5.10.1</version>
                    <scope>test</scope>
                </dependency>
            </dependencies>
        </project>
        "#;

        let deps = MavenClient::parse_pom_dependencies(sample_pom).expect("Failed to parse POM");
        assert_eq!(deps.len(), 2);

        assert_eq!(deps[0].group_id, "com.google.guava");
        assert_eq!(deps[0].artifact_id, "guava");
        assert_eq!(deps[0].version, "33.0.0-jre");
        assert_eq!(deps[0].scope, None);

        assert_eq!(deps[1].group_id, "org.junit.jupiter");
        assert_eq!(deps[1].artifact_id, "junit-jupiter");
        assert_eq!(deps[1].version, "5.10.1");
        assert_eq!(deps[1].scope, Some("test".to_string()));
    }
}

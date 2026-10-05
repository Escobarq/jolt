use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;

use super::pom_parser::parse_pom_dependencies;
use super::{Dependency, DependencyNode, SearchResultItem};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn pom(dependencies: &str) -> String {
        format!(
            r#"<project>
                <dependencies>{dependencies}</dependencies>
            </project>"#
        )
    }

    fn dependency(group: &str, artifact: &str, version: &str, extra: &str) -> String {
        format!(
            "<dependency><groupId>{group}</groupId><artifactId>{artifact}</artifactId><version>{version}</version>{extra}</dependency>"
        )
    }

    #[tokio::test]
    async fn resolves_transitive_dependencies_and_skips_non_runtime_scopes() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();

        let server = thread::spawn(move || {
            for _ in 0..3 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0_u8; 2048];
                let bytes_read = stream.read(&mut request).unwrap();
                let request = String::from_utf8_lossy(&request[..bytes_read]);
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or_default();

                let body = match path {
                    "/com/example/root/1.0/root-1.0.pom" => pom(&format!(
                        "{}{}",
                        dependency("com.example", "child", "1.0", ""),
                        dependency("com.example", "provided", "1.0", "<scope>provided</scope>")
                    )),
                    "/com/example/child/1.0/child-1.0.pom" => pom(&format!(
                        "{}{}",
                        dependency("com.example", "grandchild", "1.0", ""),
                        dependency("com.example", "test-only", "1.0", "<scope>test</scope>")
                    )),
                    "/com/example/grandchild/1.0/grandchild-1.0.pom" => pom(""),
                    _ => panic!("unexpected POM request: {path}"),
                };

                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
                .unwrap();
            }
        });

        let client = MavenClient {
            client: reqwest::Client::new(),
            search_base_url: String::new(),
            repo_base_url: format!("http://{}", address),
        };
        let dependencies = client
            .resolve_dependencies(&[(
                "com.example".to_string(),
                "root".to_string(),
                "1.0".to_string(),
            )])
            .await
            .unwrap();

        server.join().unwrap();

        let coordinates: Vec<_> = dependencies
            .iter()
            .map(|dependency| {
                (
                    dependency.group_id.as_str(),
                    dependency.artifact_id.as_str(),
                )
            })
            .collect();
        assert_eq!(
            coordinates,
            vec![("com.example", "child"), ("com.example", "grandchild"),]
        );
    }
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
                let version = doc
                    .latest_version
                    .or(doc.v)
                    .unwrap_or_else(|| "latest".to_string());
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
        self.download_jar_with_classifier(group_id, artifact_id, version, None)
            .await
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

    /// Resuelve todas las dependencias compilables del grafo Maven.
    pub async fn resolve_dependencies(
        &self,
        roots: &[(String, String, String)],
    ) -> Result<Vec<Dependency>, Box<dyn Error + Send + Sync>> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = roots.to_vec();
        while let Some((group_id, artifact_id, version)) = queue.pop() {
            let key = format!("{}:{}:{}", group_id, artifact_id, version);
            if !visited.insert(key) {
                continue;
            }
            let pom = self.fetch_pom(&group_id, &artifact_id, &version).await?;
            for dep in parse_pom_dependencies(&pom)? {
                if matches!(dep.scope.as_deref(), Some("test" | "provided" | "system")) {
                    continue;
                }
                queue.push((
                    dep.group_id.clone(),
                    dep.artifact_id.clone(),
                    dep.version.clone(),
                ));
                resolved.push(dep);
            }
        }
        Ok(resolved)
    }

    /// Construye el árbol de dependencias transitivas completo.
    pub async fn fetch_dependency_tree(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<DependencyNode, Box<dyn Error + Send + Sync>> {
        let roots = vec![(
            group_id.to_string(),
            artifact_id.to_string(),
            version.to_string(),
        )];
        let deps = self.resolve_dependencies(&roots).await?;
        let child_nodes = deps
            .into_iter()
            .map(|dep| DependencyNode {
                group_id: dep.group_id,
                artifact_id: dep.artifact_id,
                version: dep.version,
                dependencies: Vec::new(),
            })
            .collect();

        Ok(DependencyNode {
            group_id: group_id.to_string(),
            artifact_id: artifact_id.to_string(),
            version: version.to_string(),
            dependencies: child_nodes,
        })
    }
}

pub mod maven_client;
pub mod pom_parser;

pub use maven_client::MavenClient;
pub use pom_parser::parse_pom_dependencies;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResultItem {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub package_type: Option<String>,
    pub version_count: Option<u32>,
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

        let deps = parse_pom_dependencies(sample_pom).expect("Failed to parse POM");
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

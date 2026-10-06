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
    pub optional: bool,
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
                <dependency>
                    <groupId>com.fasterxml.jackson.core</groupId>
                    <artifactId>jackson-databind</artifactId>
                    <version>2.17.0</version>
                    <optional>true</optional>
                </dependency>
            </dependencies>
        </project>
        "#;

        let deps = parse_pom_dependencies(sample_pom).expect("Failed to parse POM");
        assert_eq!(deps.len(), 3);

        assert_eq!(deps[0].group_id, "com.google.guava");
        assert_eq!(deps[0].artifact_id, "guava");
        assert_eq!(deps[0].version, "33.0.0-jre");
        assert_eq!(deps[0].scope, None);
        assert!(!deps[0].optional);

        assert_eq!(deps[1].group_id, "org.junit.jupiter");
        assert_eq!(deps[1].artifact_id, "junit-jupiter");
        assert_eq!(deps[1].version, "5.10.1");
        assert_eq!(deps[1].scope, Some("test".to_string()));
        assert!(!deps[1].optional);

        assert_eq!(deps[2].group_id, "com.fasterxml.jackson.core");
        assert_eq!(deps[2].artifact_id, "jackson-databind");
        assert!(deps[2].optional);
    }

    #[test]
    fn test_parse_pom_ignores_dependency_management_and_resolves_properties() {
        let pom_with_mgmt = r#"
        <project xmlns="http://maven.apache.org/POM/4.0.0">
            <modelVersion>4.0.0</modelVersion>
            <groupId>org.example</groupId>
            <artifactId>complex-app</artifactId>
            <version>2.5.0</version>
            <properties>
                <spring.version>6.1.4</spring.version>
            </properties>
            <dependencyManagement>
                <dependencies>
                    <dependency>
                        <groupId>org.codehaus.plexus</groupId>
                        <artifactId>plexus-utils</artifactId>
                    </dependency>
                    <dependency>
                        <groupId>org.springframework</groupId>
                        <artifactId>spring-core</artifactId>
                        <version>${spring.version}</version>
                    </dependency>
                </dependencies>
            </dependencyManagement>
            <dependencies>
                <dependency>
                    <groupId>org.springframework</groupId>
                    <artifactId>spring-web</artifactId>
                    <version>${spring.version}</version>
                </dependency>
                <dependency>
                    <groupId>org.example</groupId>
                    <artifactId>self-module</artifactId>
                    <version>${project.version}</version>
                </dependency>
                <dependency>
                    <groupId>org.unversioned</groupId>
                    <artifactId>missing-ver</artifactId>
                </dependency>
            </dependencies>
        </project>
        "#;

        let deps = parse_pom_dependencies(pom_with_mgmt).expect("Failed to parse POM with management");
        // Should only contain spring-web (with resolved version 6.1.4) and self-module (with resolved 2.5.0)
        // plexus-utils (in dependencyManagement) and missing-ver (no version) must be excluded!
        assert_eq!(deps.len(), 2);

        assert_eq!(deps[0].group_id, "org.springframework");
        assert_eq!(deps[0].artifact_id, "spring-web");
        assert_eq!(deps[0].version, "6.1.4");

        assert_eq!(deps[1].group_id, "org.example");
        assert_eq!(deps[1].artifact_id, "self-module");
        assert_eq!(deps[1].version, "2.5.0");
    }
}

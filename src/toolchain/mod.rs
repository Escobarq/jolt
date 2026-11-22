pub mod detector;

use std::error::Error;
use std::path::PathBuf;

pub use detector::{
    detect_java_vendor, discover_system_jdks, find_system_jdk, inspect_jdk_dir,
    parse_java_major_version,
};

#[derive(Debug, Clone)]
pub struct Toolchain {
    pub version: String,
    pub major_version: u32,
    pub vendor: String,
    pub java_bin: PathBuf,
    pub javac_bin: PathBuf,
    pub jar_bin: PathBuf,
    pub jpackage_bin: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InstalledJdk {
    pub version_raw: String,
    pub major_version: u32,
    pub vendor: String,
    pub home_dir: PathBuf,
    pub java_bin: PathBuf,
    pub javac_bin: PathBuf,
    pub jar_bin: PathBuf,
    pub jpackage_bin: PathBuf,
}

impl From<InstalledJdk> for Toolchain {
    fn from(jdk: InstalledJdk) -> Self {
        Self {
            version: jdk.version_raw,
            major_version: jdk.major_version,
            vendor: jdk.vendor,
            java_bin: jdk.java_bin,
            javac_bin: jdk.javac_bin,
            jar_bin: jdk.jar_bin,
            jpackage_bin: jdk.jpackage_bin,
        }
    }
}

pub struct ToolchainManager {}

impl Default for ToolchainManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolchainManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Detecta si el sistema operativo cuenta con una versión compatible de Java
    pub fn find_system_jdk(&self, requested_version: &str) -> Option<Toolchain> {
        find_system_jdk(requested_version)
    }

    /// Escanea sistemáticamente variables de entorno y directorios
    pub fn discover_system_jdks(&self) -> Vec<InstalledJdk> {
        discover_system_jdks()
    }

    /// Resuelve el Toolchain requerido usando únicamente el sistema anfitrión.
    pub async fn resolve_toolchain(
        &self,
        requested_version: &str,
    ) -> Result<Toolchain, Box<dyn Error + Send + Sync>> {
        // Jolt no descarga ni selecciona toolchains administrados por su caché.
        if let Some(toolchain) = self.find_system_jdk(requested_version) {
            println!(
                "⚡ Usando JDK detectado en el sistema: {} (Java {}, {}) en {}",
                toolchain.vendor,
                toolchain.major_version,
                toolchain.version,
                toolchain.java_bin.display()
            );
            return Ok(toolchain);
        }

        let discovered = self.discover_system_jdks();
        let mut msg = format!(
            "No se encontró un JDK compatible (Java {} o superior) instalado en tu sistema.\n",
            requested_version
        );
        if !discovered.is_empty() {
            msg.push_str("\nJDKs detectados en tu entorno que no cumplen el requisito:\n");
            for jdk in discovered {
                msg.push_str(&format!(
                    "  • {} (Java {}) en {}\n",
                    jdk.vendor,
                    jdk.major_version,
                    jdk.home_dir.display()
                ));
            }
        } else {
            msg.push_str("No se detectó ningún JDK instalado en PATH, JAVA_HOME, GRAALVM_HOME ni en las rutas estándar.\n");
        }
        msg.push_str(&format!(
            "\n👉 Solución: Instala Java {} (o superior, ej. Oracle GraalVM, Eclipse Temurin) o agrega tu JDK actual al PATH / JAVA_HOME.",
            requested_version
        ));
        Err(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_formats() {
        assert_eq!(parse_java_major_version("javac 25.0.4"), Some(25));
        assert_eq!(parse_java_major_version("javac 21.0.2"), Some(21));
        assert_eq!(
            parse_java_major_version("openjdk version \"17.0.9\" 2023-10-17"),
            Some(17)
        );
        assert_eq!(
            parse_java_major_version("java version \"1.8.0_351\""),
            Some(8)
        );
        assert_eq!(parse_java_major_version("javac 17"), Some(17));
    }

    #[test]
    fn test_detect_vendors() {
        assert_eq!(
            detect_java_vendor("Java(TM) SE Runtime Environment Oracle GraalVM 25.0.4+7.1"),
            "Oracle GraalVM"
        );
        assert_eq!(
            detect_java_vendor("OpenJDK Runtime Environment Temurin-21.0.2+13"),
            "Eclipse Temurin"
        );
        assert_eq!(detect_java_vendor("Corretto-17.0.9.8.1"), "Amazon Corretto");
        assert_eq!(detect_java_vendor("Zulu21.32+17-CA"), "Azul Zulu");
    }

    #[test]
    fn test_find_system_jdk() {
        let manager = ToolchainManager::new();
        let toolchain = manager.find_system_jdk("21");
        assert!(
            toolchain.is_some(),
            "Debe detectar un JDK compatible >= 21 si existe"
        );
        let tc = toolchain.unwrap();
        assert!(tc.major_version >= 21);
    }

    #[tokio::test]
    async fn does_not_download_when_system_jdk_is_missing() {
        let manager = ToolchainManager::new();
        let error = manager
            .resolve_toolchain("999")
            .await
            .expect_err("No debe existir un JDK de prueba compatible");
        let message = error.to_string();

        assert!(message.contains("No se encontró un JDK compatible"));
    }
}

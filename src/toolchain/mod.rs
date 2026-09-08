pub mod detector;
pub mod downloader;

use std::error::Error;
use std::path::PathBuf;

pub use detector::{detect_java_vendor, discover_system_jdks, find_system_jdk, inspect_jdk_dir, parse_java_major_version};
pub use downloader::{download_and_extract_jdk, find_binaries_in_dir};

#[derive(Debug, Clone)]
pub struct Toolchain {
    pub version: String,
    pub major_version: u32,
    pub vendor: String,
    pub java_bin: PathBuf,
    pub javac_bin: PathBuf,
    pub jar_bin: PathBuf,
    pub jpackage_bin: PathBuf,
    pub native_image_bin: Option<PathBuf>,
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
    pub native_image_bin: Option<PathBuf>,
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
            native_image_bin: jdk.native_image_bin,
        }
    }
}

pub struct ToolchainManager {
    jdks_root: PathBuf,
    client: reqwest::Client,
}

impl Default for ToolchainManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolchainManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let jdks_root = home.join(".jolt").join("jdks");
        let client = reqwest::Client::builder()
            .user_agent("jolt-package-manager/0.6.0")
            .build()
            .unwrap_or_default();
        Self { jdks_root, client }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn with_root(root: PathBuf) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("jolt-package-manager/0.6.0")
            .build()
            .unwrap_or_default();
        Self {
            jdks_root: root,
            client,
        }
    }

    /// Detecta si el JDK ya está aprovisionado en la caché global de Jolt (~/.jolt/jdks)
    pub fn find_cached_jdk(&self, version: &str) -> Option<Toolchain> {
        let jdk_dir = self.jdks_root.join(version);
        if !jdk_dir.exists() {
            return None;
        }

        if let Some(jdk) = inspect_jdk_dir(&jdk_dir) {
            return Some(jdk.into());
        }

        let (java, javac, jar, jpackage) = find_binaries_in_dir(&jdk_dir)?;
        let major = parse_java_major_version(version).unwrap_or(21);
        Some(Toolchain {
            version: version.to_string(),
            major_version: major,
            vendor: "Eclipse Temurin (Jolt Cache)".to_string(),
            java_bin: java,
            javac_bin: javac,
            jar_bin: jar,
            jpackage_bin: jpackage,
            native_image_bin: None,
        })
    }

    /// Detecta si el sistema operativo cuenta con una versión compatible de Java
    pub fn find_system_jdk(&self, requested_version: &str) -> Option<Toolchain> {
        find_system_jdk(requested_version)
    }

    /// Escanea sistemáticamente variables de entorno y directorios
    pub fn discover_system_jdks(&self) -> Vec<InstalledJdk> {
        discover_system_jdks()
    }

    /// Resuelve el Toolchain requerido (caché local -> sistema anfitrión -> auto-descarga si está autorizada)
    pub async fn resolve_toolchain(
        &self,
        requested_version: &str,
        allow_download: bool,
    ) -> Result<Toolchain, Box<dyn Error + Send + Sync>> {
        // 1. Revisar caché local de Jolt
        if let Some(toolchain) = self.find_cached_jdk(requested_version) {
            println!("⚡ Usando JDK desde caché de Jolt: {} en {}", toolchain.version, toolchain.java_bin.display());
            return Ok(toolchain);
        }

        // 2. Revisar si el sistema anfitrión tiene un JDK compatible
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

        // 3. Si no hay JDK compatible
        if allow_download {
            println!("🌐 Descargando OpenJDK Temurin {} para tu arquitectura...", requested_version);
            download_and_extract_jdk(&self.client, &self.jdks_root, requested_version).await
        } else {
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
                "\n👉 Solución: Instala Java {} (o superior, ej. Oracle GraalVM, Eclipse Temurin) o agrega tu JDK actual al PATH / JAVA_HOME.\n",
                requested_version
            ));
            msg.push_str(&format!(
                "💡 Para que Jolt descargue e instale automáticamente Eclipse Temurin {}, ejecuta con el argumento '--download-jdk'.",
                requested_version
            ));
            Err(msg.into())
        }
    }

    /// Alias compatible con versiones anteriores
    #[allow(dead_code)]
    pub async fn get_or_download_toolchain(
        &self,
        requested_version: &str,
    ) -> Result<Toolchain, Box<dyn Error + Send + Sync>> {
        self.resolve_toolchain(requested_version, false).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_formats() {
        assert_eq!(parse_java_major_version("javac 25.0.4"), Some(25));
        assert_eq!(parse_java_major_version("javac 21.0.2"), Some(21));
        assert_eq!(parse_java_major_version("openjdk version \"17.0.9\" 2023-10-17"), Some(17));
        assert_eq!(parse_java_major_version("java version \"1.8.0_351\""), Some(8));
        assert_eq!(parse_java_major_version("javac 17"), Some(17));
    }

    #[test]
    fn test_detect_vendors() {
        assert_eq!(detect_java_vendor("Java(TM) SE Runtime Environment Oracle GraalVM 25.0.4+7.1"), "Oracle GraalVM");
        assert_eq!(detect_java_vendor("OpenJDK Runtime Environment Temurin-21.0.2+13"), "Eclipse Temurin");
        assert_eq!(detect_java_vendor("Corretto-17.0.9.8.1"), "Amazon Corretto");
        assert_eq!(detect_java_vendor("Zulu21.32+17-CA"), "Azul Zulu");
    }

    #[test]
    fn test_find_system_jdk() {
        let manager = ToolchainManager::new();
        let toolchain = manager.find_system_jdk("21");
        assert!(toolchain.is_some(), "Debe detectar un JDK compatible >= 21 si existe");
        let tc = toolchain.unwrap();
        assert!(tc.major_version >= 21);
    }
}

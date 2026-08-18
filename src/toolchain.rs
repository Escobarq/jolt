use flate2::read::GzDecoder;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Toolchain {
    pub version: String,
    pub java_bin: PathBuf,
    pub javac_bin: PathBuf,
    pub jar_bin: PathBuf,
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
            .user_agent("jolt-package-manager/0.2.0")
            .build()
            .unwrap_or_default();
        Self { jdks_root, client }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn with_root(root: PathBuf) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("jolt-package-manager/0.2.0")
            .build()
            .unwrap_or_default();
        Self {
            jdks_root: root,
            client,
        }
    }

    /// Detecta si el JDK ya está aprovisionado en la caché global de Jolt
    pub fn find_cached_jdk(&self, version: &str) -> Option<Toolchain> {
        let jdk_dir = self.jdks_root.join(version);
        if !jdk_dir.exists() {
            return None;
        }

        let (java, javac, jar) = Self::find_binaries_in_dir(&jdk_dir)?;
        Some(Toolchain {
            version: version.to_string(),
            java_bin: java,
            javac_bin: javac,
            jar_bin: jar,
        })
    }

    /// Detecta si el sistema operativo ya cuenta con una versión compatible de Java
    pub fn find_system_jdk(&self, requested_version: &str) -> Option<Toolchain> {
        let output = Command::new("javac").arg("-version").output().ok()?;
        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let version_str = format!("{} {}", stdout, stderr);

        if version_str.contains(requested_version) {
            return Some(Toolchain {
                version: requested_version.to_string(),
                java_bin: PathBuf::from("java"),
                javac_bin: PathBuf::from("javac"),
                jar_bin: PathBuf::from("jar"),
            });
        }

        None
    }

    /// Aprovisiona el Toolchain requerido (usando sistema, caché global, o descargando de Adoptium)
    pub async fn get_or_download_toolchain(
        &self,
        requested_version: &str,
    ) -> Result<Toolchain, Box<dyn Error + Send + Sync>> {
        // 1. Revisar caché local de Jolt
        if let Some(toolchain) = self.find_cached_jdk(requested_version) {
            return Ok(toolchain);
        }

        // 2. Revisar si el sistema anfitrión tiene la versión requerida
        if let Some(toolchain) = self.find_system_jdk(requested_version) {
            return Ok(toolchain);
        }

        // 3. Descargar automáticamente de Adoptium Temurin
        println!("🌐 Descargando OpenJDK Temurin {} para tu arquitectura...", requested_version);
        self.download_and_extract_jdk(requested_version).await
    }

    /// Descarga y descomprime OpenJDK desde la API de Adoptium
    pub async fn download_and_extract_jdk(
        &self,
        version: &str,
    ) -> Result<Toolchain, Box<dyn Error + Send + Sync>> {
        let os = if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "mac"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            return Err("Sistema operativo no soportado para auto-descarga de JDK".into());
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            return Err("Arquitectura no soportada para auto-descarga de JDK".into());
        };

        let url = format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jdk/hotspot/normal/eclipse?project=jdk",
            version, os, arch
        );

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(format!(
                "No se pudo descargar JDK {} desde Adoptium (Status: {})",
                version,
                response.status()
            )
            .into());
        }

        let bytes = response.bytes().await?;
        let target_dir = self.jdks_root.join(version);
        fs::create_dir_all(&target_dir)?;

        // Descomprimir tar.gz
        let tar = GzDecoder::new(&bytes[..]);
        let mut archive = Archive::new(tar);
        archive.unpack(&target_dir)?;

        let (java, javac, jar) = Self::find_binaries_in_dir(&target_dir)
            .ok_or("No se encontraron los binarios de Java dentro del archivo descomprimido")?;

        Ok(Toolchain {
            version: version.to_string(),
            java_bin: java,
            javac_bin: javac,
            jar_bin: jar,
        })
    }

    fn find_binaries_in_dir(dir: &Path) -> Option<(PathBuf, PathBuf, PathBuf)> {
        fn scan(d: &Path) -> (Option<PathBuf>, Option<PathBuf>, Option<PathBuf>) {
            let mut java = None;
            let mut javac = None;
            let mut jar = None;

            if let Ok(entries) = fs::read_dir(d) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        let (c_java, c_javac, c_jar) = scan(&p);
                        if java.is_none() { java = c_java; }
                        if javac.is_none() { javac = c_javac; }
                        if jar.is_none() { jar = c_jar; }
                    } else if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                        if name == "java" || name == "java.exe" {
                            java = Some(p.clone());
                        } else if name == "javac" || name == "javac.exe" {
                            javac = Some(p.clone());
                        } else if name == "jar" || name == "jar.exe" {
                            jar = Some(p.clone());
                        }
                    }
                }
            }
            (java, javac, jar)
        }

        let (java, javac, jar) = scan(dir);
        Some((java?, javac?, jar?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_system_jdk() {
        let manager = ToolchainManager::new();
        // El sistema tiene Java 21 instalado
        let toolchain = manager.find_system_jdk("21");
        assert!(toolchain.is_some());
        let tc = toolchain.unwrap();
        assert_eq!(tc.version, "21");
    }
}

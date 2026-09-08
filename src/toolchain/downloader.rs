use flate2::read::GzDecoder;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tar::Archive;

use super::detector::parse_java_major_version;
use super::Toolchain;

/// Descarga y descomprime OpenJDK desde la API de Adoptium Temurin
pub async fn download_and_extract_jdk(
    client: &reqwest::Client,
    jdks_root: &Path,
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

    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        return Err(format!(
            "No se pudo descargar JDK {} desde Adoptium (Status: {})",
            version,
            response.status()
        )
        .into());
    }

    let bytes = response.bytes().await?;
    let target_dir = jdks_root.join(version);
    fs::create_dir_all(&target_dir)?;

    // Descomprimir tar.gz
    let tar = GzDecoder::new(&bytes[..]);
    let mut archive = Archive::new(tar);
    archive.unpack(&target_dir)?;

    let (java, javac, jar, jpackage) = find_binaries_in_dir(&target_dir)
        .ok_or("No se encontraron los binarios de Java dentro del archivo descomprimido")?;

    let major = parse_java_major_version(version).unwrap_or(21);
    Ok(Toolchain {
        version: version.to_string(),
        major_version: major,
        vendor: "Eclipse Temurin".to_string(),
        java_bin: java,
        javac_bin: javac,
        jar_bin: jar,
        jpackage_bin: jpackage,
        native_image_bin: None,
    })
}

pub fn find_binaries_in_dir(dir: &Path) -> Option<(PathBuf, PathBuf, PathBuf, PathBuf)> {
    fn scan(d: &Path) -> (Option<PathBuf>, Option<PathBuf>, Option<PathBuf>, Option<PathBuf>) {
        let mut java = None;
        let mut javac = None;
        let mut jar = None;
        let mut jpackage = None;

        if let Ok(entries) = fs::read_dir(d) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let (c_java, c_javac, c_jar, c_jpackage) = scan(&p);
                    if java.is_none() { java = c_java; }
                    if javac.is_none() { javac = c_javac; }
                    if jar.is_none() { jar = c_jar; }
                    if jpackage.is_none() { jpackage = c_jpackage; }
                } else if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                    if name == "java" || name == "java.exe" {
                        java = Some(p.clone());
                    } else if name == "javac" || name == "javac.exe" {
                        javac = Some(p.clone());
                    } else if name == "jar" || name == "jar.exe" {
                        jar = Some(p.clone());
                    } else if name == "jpackage" || name == "jpackage.exe" {
                        jpackage = Some(p.clone());
                    }
                }
            }
        }
        (java, javac, jar, jpackage)
    }

    let (java, javac, jar, jpackage) = scan(dir);
    let jpackage_found = jpackage.unwrap_or_else(|| PathBuf::from("jpackage"));
    Some((java?, javac?, jar?, jpackage_found))
}

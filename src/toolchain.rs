use flate2::read::GzDecoder;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;

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

/// Extrae la versión mayor de Java (ej. 25 de "25.0.4", 21 de "21.0.2", 8 de "1.8.0_351")
pub fn parse_java_major_version(version_output: &str) -> Option<u32> {
    for line in version_output.lines() {
        for word in line.split_whitespace() {
            let clean = word.trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || !c.is_ascii_graphic());
            let num_part = clean.trim_start_matches(|c: char| !c.is_ascii_digit());
            if num_part.is_empty() {
                continue;
            }
            let mut parts = num_part.split('.');
            if let Some(first) = parts.next() {
                if first == "1" {
                    if let Some(second) = parts.next() {
                        let clean_sec = second.split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
                        if let Ok(v) = clean_sec.parse::<u32>() {
                            return Some(v);
                        }
                    }
                } else {
                    let clean_first = first.split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
                    if let Ok(v) = clean_first.parse::<u32>() {
                        if v >= 1 {
                            return Some(v);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Identifica el proveedor o distribución del JDK
pub fn detect_java_vendor(output: &str) -> String {
    let lower = output.to_lowercase();
    if lower.contains("graalvm") {
        "Oracle GraalVM".to_string()
    } else if lower.contains("temurin") || lower.contains("adoptium") {
        "Eclipse Temurin".to_string()
    } else if lower.contains("corretto") {
        "Amazon Corretto".to_string()
    } else if lower.contains("zulu") || lower.contains("azul") {
        "Azul Zulu".to_string()
    } else if lower.contains("liberica") || lower.contains("bellsoft") {
        "BellSoft Liberica".to_string()
    } else if lower.contains("microsoft") {
        "Microsoft OpenJDK".to_string()
    } else if lower.contains("oracle") {
        "Oracle JDK".to_string()
    } else if lower.contains("openjdk") {
        "OpenJDK".to_string()
    } else {
        "Java HotSpot".to_string()
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

    /// Detecta si un directorio dado contiene una instalación válida de JDK
    pub fn inspect_jdk_dir(dir: &Path) -> Option<InstalledJdk> {
        if !dir.exists() {
            return None;
        }

        let bin_dir = if dir.join("bin").is_dir() {
            dir.join("bin")
        } else if dir.is_dir() && (dir.join("javac.exe").exists() || dir.join("javac").exists()) {
            dir.to_path_buf()
        } else {
            return None;
        };

        let javac_bin = if bin_dir.join("javac.exe").exists() {
            bin_dir.join("javac.exe")
        } else if bin_dir.join("javac").exists() {
            bin_dir.join("javac")
        } else {
            return None;
        };

        let java_bin = if bin_dir.join("java.exe").exists() {
            bin_dir.join("java.exe")
        } else if bin_dir.join("java").exists() {
            bin_dir.join("java")
        } else {
            return None;
        };

        let jar_bin = if bin_dir.join("jar.exe").exists() {
            bin_dir.join("jar.exe")
        } else if bin_dir.join("jar").exists() {
            bin_dir.join("jar")
        } else {
            PathBuf::from("jar")
        };

        let jpackage_bin = if bin_dir.join("jpackage.exe").exists() {
            bin_dir.join("jpackage.exe")
        } else if bin_dir.join("jpackage").exists() {
            bin_dir.join("jpackage")
        } else {
            PathBuf::from("jpackage")
        };

        let native_image_bin = if bin_dir.join("native-image.cmd").exists() {
            Some(bin_dir.join("native-image.cmd"))
        } else if bin_dir.join("native-image.exe").exists() {
            Some(bin_dir.join("native-image.exe"))
        } else if bin_dir.join("native-image").exists() {
            Some(bin_dir.join("native-image"))
        } else {
            None
        };

        // Extraer versión desde javac y java
        let javac_out = Command::new(&javac_bin).arg("-version").output().ok()?;
        let stdout = String::from_utf8_lossy(&javac_out.stdout);
        let stderr = String::from_utf8_lossy(&javac_out.stderr);
        let combined_javac = format!("{} {}", stdout, stderr);

        let major_version = parse_java_major_version(&combined_javac)?;

        let java_out = Command::new(&java_bin).arg("-version").output().ok();
        let combined_java = java_out
            .as_ref()
            .map(|o| format!("{} {}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
            .unwrap_or_default();

        let vendor = detect_java_vendor(&format!("{} {}", combined_javac, combined_java));

        let version_raw = combined_javac
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();

        let home_dir = if dir.join("bin").is_dir() {
            dir.to_path_buf()
        } else {
            dir.parent().unwrap_or(dir).to_path_buf()
        };

        let clean_path = |p: &Path| -> PathBuf {
            let s = p.to_string_lossy();
            if s.starts_with(r"\\?\") {
                PathBuf::from(&s[4..])
            } else {
                p.to_path_buf()
            }
        };

        Some(InstalledJdk {
            version_raw: if version_raw.is_empty() { format!("Java {}", major_version) } else { version_raw },
            major_version,
            vendor,
            home_dir: clean_path(&home_dir),
            java_bin: clean_path(&java_bin),
            javac_bin: clean_path(&javac_bin),
            jar_bin: clean_path(&jar_bin),
            jpackage_bin: clean_path(&jpackage_bin),
            native_image_bin: native_image_bin.as_ref().map(|p| clean_path(p)),
        })
    }

    /// Escanea sistemáticamente variables de entorno, PATH y directorios estándar para encontrar JDKs
    pub fn discover_system_jdks(&self) -> Vec<InstalledJdk> {
        let mut jdks = Vec::new();
        let mut visited = HashSet::new();
        let mut known_homes = HashSet::new();

        let mut check_candidate = |p: &Path, list: &mut Vec<InstalledJdk>| {
            let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
            if visited.insert(canon.clone()) {
                if let Some(jdk) = Self::inspect_jdk_dir(&canon) {
                    let canon_home = jdk.home_dir.canonicalize().unwrap_or_else(|_| jdk.home_dir.clone());
                    if known_homes.insert(canon_home) {
                        list.push(jdk);
                    }
                }
            }
        };

        // 1. Variable GRAALVM_HOME
        if let Ok(val) = std::env::var("GRAALVM_HOME") {
            let p = PathBuf::from(val);
            check_candidate(&p, &mut jdks);
        }

        // 2. Variable JAVA_HOME
        if let Ok(val) = std::env::var("JAVA_HOME") {
            let p = PathBuf::from(val);
            check_candidate(&p, &mut jdks);
        }

        // 3. Binarios en PATH (javac)
        if let Ok(path_var) = std::env::var("PATH") {
            for entry in std::env::split_paths(&path_var) {
                let javac_exe = if cfg!(windows) { entry.join("javac.exe") } else { entry.join("javac") };
                if javac_exe.exists() {
                    let home = entry.parent().unwrap_or(&entry);
                    check_candidate(home, &mut jdks);
                }
            }
        }

        // 4. Directorios estándar según el SO
        let mut standard_roots = Vec::new();

        if cfg!(windows) {
            standard_roots.push(PathBuf::from("C:\\Program Files\\GraalVm"));
            standard_roots.push(PathBuf::from("C:\\Program Files\\Java"));
            standard_roots.push(PathBuf::from("C:\\Program Files\\Eclipse Adoptium"));
            standard_roots.push(PathBuf::from("C:\\Program Files\\Amazon Corretto"));
            standard_roots.push(PathBuf::from("C:\\Program Files\\BellSoft"));
            standard_roots.push(PathBuf::from("C:\\Program Files\\Microsoft"));
            standard_roots.push(PathBuf::from("C:\\Program Files\\Zulu"));

            if let Ok(prog_files) = std::env::var("ProgramFiles") {
                standard_roots.push(PathBuf::from(&prog_files).join("Java"));
                standard_roots.push(PathBuf::from(&prog_files).join("GraalVm"));
            }
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                standard_roots.push(PathBuf::from(&local_app_data).join("Programs").join("Eclipse Adoptium"));
            }
        } else if cfg!(target_os = "macos") {
            standard_roots.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
            if let Some(home) = dirs::home_dir() {
                standard_roots.push(home.join("Library").join("Java").join("JavaVirtualMachines"));
                standard_roots.push(home.join(".sdkman").join("candidates").join("java"));
            }
        } else {
            // Linux u otros Unix
            standard_roots.push(PathBuf::from("/usr/lib/jvm"));
            standard_roots.push(PathBuf::from("/opt/java"));
            standard_roots.push(PathBuf::from("/opt/graalvm"));
            if let Some(home) = dirs::home_dir() {
                standard_roots.push(home.join(".sdkman").join("candidates").join("java"));
            }
        }

        for root in standard_roots {
            if root.is_dir() {
                // El root mismo podría ser un JDK o contener múltiples subcarpetas de JDK
                check_candidate(&root, &mut jdks);
                if let Ok(entries) = fs::read_dir(&root) {
                    for entry in entries.flatten() {
                        let sub = entry.path();
                        if sub.is_dir() {
                            // En macOS puede tener Contents/Home
                            if sub.join("Contents").join("Home").is_dir() {
                                check_candidate(&sub.join("Contents").join("Home"), &mut jdks);
                            } else {
                                check_candidate(&sub, &mut jdks);
                            }
                        }
                    }
                }
            }
        }

        jdks
    }

    /// Detecta si el JDK ya está aprovisionado en la caché global de Jolt (~/.jolt/jdks)
    pub fn find_cached_jdk(&self, version: &str) -> Option<Toolchain> {
        let jdk_dir = self.jdks_root.join(version);
        if !jdk_dir.exists() {
            return None;
        }

        if let Some(jdk) = Self::inspect_jdk_dir(&jdk_dir) {
            return Some(jdk.into());
        }

        let (java, javac, jar, jpackage) = Self::find_binaries_in_dir(&jdk_dir)?;
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
    /// Si el proyecto requiere Java 21 y el usuario tiene Java 25, es compatible y se reutiliza.
    pub fn find_system_jdk(&self, requested_version: &str) -> Option<Toolchain> {
        let target_major = requested_version.parse::<u32>().unwrap_or_else(|_| {
            parse_java_major_version(requested_version).unwrap_or(21)
        });

        let system_jdks = self.discover_system_jdks();

        // 1. Filtrar los que cumplen compatibilidad (major >= target_major)
        let mut compatible: Vec<InstalledJdk> = system_jdks
            .into_iter()
            .filter(|jdk| jdk.major_version >= target_major)
            .collect();

        if compatible.is_empty() {
            return None;
        }

        // 2. Ordenar candidatos:
        //    - Primero: coincidencia exacta de versión
        //    - Segundo: GraalVM (si existe compatibilidad)
        //    - Tercero: versión más cercana a la solicitada
        compatible.sort_by(|a, b| {
            let a_exact = a.major_version == target_major;
            let b_exact = b.major_version == target_major;
            if a_exact && !b_exact {
                return std::cmp::Ordering::Less;
            }
            if !a_exact && b_exact {
                return std::cmp::Ordering::Greater;
            }

            let a_graal = a.vendor.contains("GraalVM") || a.native_image_bin.is_some();
            let b_graal = b.vendor.contains("GraalVM") || b.native_image_bin.is_some();
            if a_graal && !b_graal {
                return std::cmp::Ordering::Less;
            }
            if !a_graal && b_graal {
                return std::cmp::Ordering::Greater;
            }

            let diff_a = a.major_version.saturating_sub(target_major);
            let diff_b = b.major_version.saturating_sub(target_major);
            diff_a.cmp(&diff_b)
        });

        let chosen = compatible.into_iter().next()?;
        Some(chosen.into())
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
            self.download_and_extract_jdk(requested_version).await
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

    /// Alias compatible con versiones anteriores: resuelve el toolchain sin descarga silenciosa
    #[allow(dead_code)]
    pub async fn get_or_download_toolchain(
        &self,
        requested_version: &str,
    ) -> Result<Toolchain, Box<dyn Error + Send + Sync>> {
        self.resolve_toolchain(requested_version, false).await
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

        let (java, javac, jar, jpackage) = Self::find_binaries_in_dir(&target_dir)
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
        // Detecta el JDK instalado en el sistema (ej. GraalVM 25 en este entorno)
        let toolchain = manager.find_system_jdk("21");
        assert!(toolchain.is_some(), "Debe detectar un JDK compatible >= 21 si existe");
        let tc = toolchain.unwrap();
        assert!(tc.major_version >= 21);
    }
}

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{InstalledJdk, Toolchain};

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
pub fn discover_system_jdks() -> Vec<InstalledJdk> {
    let mut jdks = Vec::new();
    let mut visited = HashSet::new();
    let mut known_homes = HashSet::new();

    let mut check_candidate = |p: &Path, list: &mut Vec<InstalledJdk>| {
        let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        if visited.insert(canon.clone()) {
            if let Some(jdk) = inspect_jdk_dir(&canon) {
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
            check_candidate(&root, &mut jdks);
            if let Ok(entries) = fs::read_dir(&root) {
                for entry in entries.flatten() {
                    let sub = entry.path();
                    if sub.is_dir() {
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

/// Detecta si el sistema anfitrión cuenta con un JDK compatible
pub fn find_system_jdk(requested_version: &str) -> Option<Toolchain> {
    let target_major = requested_version.parse::<u32>().unwrap_or_else(|_| {
        parse_java_major_version(requested_version).unwrap_or(21)
    });

    let system_jdks = discover_system_jdks();

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

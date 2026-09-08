use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::toolchain::Toolchain;
use super::jar::build_standalone_jar;

/// Compila la aplicación a un binario nativo autónomo usando GraalVM Native Image
pub fn build_native_image(
    project_dir: &Path,
    project_name: &str,
    version: &str,
    main_class: &str,
    toolchain: Option<&Toolchain>,
    graalvm_config: Option<&crate::core::manifest::GraalVmConfig>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    // 1. Localizar el ejecutable native-image
    let native_image_bin = toolchain
        .and_then(|t| t.native_image_bin.clone())
        .or_else(|| {
            if let Ok(g_home) = std::env::var("GRAALVM_HOME") {
                let bin_dir = PathBuf::from(g_home).join("bin");
                for name in &["native-image.cmd", "native-image.exe", "native-image"] {
                    let cand = bin_dir.join(name);
                    if cand.exists() {
                        return Some(cand);
                    }
                }
            }
            if let Ok(path_var) = std::env::var("PATH") {
                for entry in std::env::split_paths(&path_var) {
                    for name in &["native-image.cmd", "native-image.exe", "native-image"] {
                        let cand = entry.join(name);
                        if cand.exists() {
                            return Some(cand);
                        }
                    }
                }
            }
            None
        })
        .ok_or_else(|| {
            "No se encontró el ejecutable 'native-image'. Asegúrate de tener GraalVM instalado con el componente native-image en tu PATH o GRAALVM_HOME."
        })?;

    println!("⚡ Compilando Fat-JAR previo para GraalVM Native Image...");
    let target_jar = build_standalone_jar(project_dir, project_name, version, main_class, toolchain)?;

    let dist_dir = project_dir.join("dist");
    fs::create_dir_all(&dist_dir)?;

    let bin_name = graalvm_config
        .and_then(|c| c.name.as_deref())
        .unwrap_or(project_name);

    let clean_path = |p: &Path| -> PathBuf {
        let s = p.to_string_lossy();
        if s.starts_with(r"\\?\") {
            PathBuf::from(&s[4..])
        } else {
            p.to_path_buf()
        }
    };

    // Asegurar que el Fat-JAR tenga ruta absoluta limpia ya que native-image se ejecuta con current_dir = dist/
    let abs_target_jar = if target_jar.is_relative() {
        let curr = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        curr.join(&target_jar)
    } else {
        target_jar.clone()
    };
    let clean_target_jar = clean_path(&abs_target_jar);
    let clean_native_image_bin = clean_path(&native_image_bin);

    println!("🚀 Invocando GraalVM Native Image para generar binario nativo '{}'...", bin_name);
    println!("   Ejecutable: {}", clean_native_image_bin.display());

    let mut cmd = Command::new(&clean_native_image_bin);
    cmd.current_dir(&dist_dir);

    // Pasar el Fat-JAR con ruta absoluta
    cmd.arg("-jar").arg(&clean_target_jar);

    // Nombre del binario
    cmd.arg("-o").arg(bin_name);

    // Argumentos configurados en jolt.toml
    if let Some(cfg) = graalvm_config {
        if let Some(args) = &cfg.args {
            for arg in args {
                cmd.arg(arg);
            }
        }
        if let Some(refl) = &cfg.reflection_config {
            let refl_path = if Path::new(refl).is_relative() {
                let curr = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                curr.join(project_dir).join(refl)
            } else {
                PathBuf::from(refl)
            };
            let clean_refl = clean_path(&refl_path);
            cmd.arg(format!("-H:ReflectionConfigurationFiles={}", clean_refl.display()));
        }
        if let Some(res) = &cfg.resources_config {
            let res_path = if Path::new(res).is_relative() {
                let curr = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                curr.join(project_dir).join(res)
            } else {
                PathBuf::from(res)
            };
            let clean_res = clean_path(&res_path);
            cmd.arg(format!("-H:ResourceConfigurationFiles={}", clean_res.display()));
        }
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("Fallo en la compilación nativa con GraalVM (código: {:?})", status.code()).into());
    }

    let out_binary = if cfg!(windows) {
        dist_dir.join(format!("{}.exe", bin_name))
    } else {
        dist_dir.join(bin_name)
    };

    println!("✅ Binario nativo generado exitosamente en: {}", out_binary.display());
    Ok(out_binary)
}

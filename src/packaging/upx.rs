use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Localiza el binario de UPX en PATH o en ubicaciones estándar de Windows (Scoop, Chocolatey, etc.)
pub fn find_upx_binary() -> Option<PathBuf> {
    if let Ok(output) = Command::new("upx").arg("--version").output() {
        if output.status.success() {
            return Some(PathBuf::from("upx"));
        }
    }

    if cfg!(target_os = "windows") {
        if let Some(home) = dirs::home_dir() {
            let scoop_shim = home.join("scoop").join("shims").join("upx.exe");
            if scoop_shim.is_file() {
                return Some(scoop_shim);
            }
            let local_scoop = home.join("AppData").join("Local").join("scoop").join("shims").join("upx.exe");
            if local_scoop.is_file() {
                return Some(local_scoop);
            }
        }
        let choco_bin = PathBuf::from("C:\\ProgramData\\chocolatey\\bin\\upx.exe");
        if choco_bin.is_file() {
            return Some(choco_bin);
        }
    }

    None
}

/// Comprime ejecutables y bibliotecas dinámicas (.exe, .dll, .so) con UPX
pub fn compress_with_upx(
    target_dir_or_file: &Path,
    custom_args: Option<&[String]>,
    verbose: bool,
) -> Result<(u64, u64, usize), Box<dyn Error + Send + Sync>> {
    let upx_bin = find_upx_binary().ok_or("UPX no está instalado o no se encuentra en el sistema")?;

    let mut targets = Vec::new();
    if target_dir_or_file.is_file() {
        targets.push(target_dir_or_file.to_path_buf());
    } else if target_dir_or_file.is_dir() {
        for entry in WalkDir::new(target_dir_or_file).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() {
                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if ext == "exe" || ext == "dll" || ext == "so" {
                    targets.push(p.to_path_buf());
                }
            }
        }
    }

    if targets.is_empty() {
        return Ok((0, 0, 0));
    }

    let default_args = vec!["--best".to_string(), "--lzma".to_string()];
    let args_to_use = custom_args.unwrap_or(&default_args);

    let mut total_original = 0u64;
    let mut total_compressed = 0u64;
    let mut compressed_count = 0;

    for target in targets {
        let orig_len = target.metadata().map(|m| m.len()).unwrap_or(0);
        total_original += orig_len;

        let mut cmd = Command::new(&upx_bin);
        for a in args_to_use {
            cmd.arg(a);
        }
        cmd.arg(&target);

        let ok = if verbose {
            cmd.status().map(|s| s.success()).unwrap_or(false)
        } else {
            cmd.output().map(|o| o.status.success()).unwrap_or(false)
        };

        let new_len = target.metadata().map(|m| m.len()).unwrap_or(orig_len);
        if ok && new_len < orig_len {
            compressed_count += 1;
            total_compressed += new_len;
            let pct = 100.0 - (new_len as f64 / orig_len as f64 * 100.0);
            println!(
                "       [UPX] {} ({:.2} MB -> {:.2} MB, -{:.1}%)",
                target.file_name().and_then(|f| f.to_str()).unwrap_or("archivo"),
                orig_len as f64 / (1024.0 * 1024.0),
                new_len as f64 / (1024.0 * 1024.0),
                pct
            );
        } else {
            total_compressed += new_len;
            if verbose {
                println!("       [UPX] Omitido: {}", target.display());
            }
        }
    }

    Ok((total_original, total_compressed, compressed_count))
}

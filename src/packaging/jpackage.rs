use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::build::detect_main_class;
use crate::toolchain::Toolchain;
use super::jar::build_standalone_jar;
use super::nsis::{find_makensis_binary, generate_nsis_script};
use super::upx::compress_with_upx;

/// Empaqueta la aplicación como un ejecutable binario autónomo o instalador usando jpackage / NSIS / UPX
pub fn package_native_app(
    project_dir: &Path,
    manifest: &crate::core::manifest::JoltManifest,
    cli_type: Option<&str>,
    cli_dest: Option<&str>,
    cli_name: Option<&str>,
    cli_app_version: Option<&str>,
    cli_main_class: Option<&str>,
    cli_icon: Option<&str>,
    cli_java_options: Option<&str>,
    cli_upx: Option<bool>,
    cli_add_to_path: Option<bool>,
    cli_scope: Option<&str>,
    verbose: bool,
    toolchain: Option<&Toolchain>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    // 1. Determinar clase principal
    let main_class = cli_main_class
        .map(|s| s.to_string())
        .or_else(|| manifest.package.as_ref().and_then(|p| p.main_class.clone()))
        .or_else(|| manifest.project.as_ref().and_then(|p| p.main_class.clone()))
        .or_else(|| detect_main_class(project_dir))
        .unwrap_or_else(|| "Main".to_string());

    // 2. Determinar nombre de la aplicación / binario
    let raw_app_name = cli_name
        .map(|s| s.to_string())
        .or_else(|| manifest.package.as_ref().and_then(|p| p.name.clone()))
        .or_else(|| manifest.project.as_ref().map(|p| p.name.clone()))
        .unwrap_or_else(|| "app".to_string());

    let app_name = Path::new(&raw_app_name)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(&raw_app_name)
        .replace(['/', '\\', ' ', ':'], "_");

    // 3. Determinar y sanitizar versión de la aplicación para jpackage (debe ser dígitos y puntos, ej. 1.0.0)
    let raw_version = cli_app_version
        .map(|s| s.to_string())
        .or_else(|| manifest.project.as_ref().map(|p| p.version.clone()))
        .unwrap_or_else(|| "1.0.0".to_string());

    let sanitized_version: String = {
        let base = raw_version.split('-').next().unwrap_or("1.0.0").trim();
        let cleaned: String = base.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
        if cleaned.is_empty() {
            "1.0.0".to_string()
        } else {
            cleaned
        }
    };

    // 4. Determinar configuración de Windows / UPX / NSIS
    let win_cfg = manifest.package.as_ref().and_then(|p| p.windows.as_ref());
    let is_upx_requested = cli_upx.or_else(|| win_cfg.and_then(|w| w.upx)).unwrap_or(false);
    let is_add_to_path = cli_add_to_path.or_else(|| win_cfg.and_then(|w| w.add_to_path)).unwrap_or(false);
    let scope_str = cli_scope
        .map(|s| s.to_string())
        .or_else(|| win_cfg.and_then(|w| w.scope.clone()))
        .unwrap_or_else(|| "per-user".to_string());
    let desktop_shortcut = win_cfg.and_then(|w| w.desktop_shortcut).unwrap_or(true);
    let start_menu = win_cfg.and_then(|w| w.start_menu).unwrap_or(true);
    let license_file = win_cfg.and_then(|w| w.license.as_ref().map(|l| project_dir.join(l)));

    // 5. Determinar tipo de paquete
    let mut pkg_type = cli_type
        .map(|s| s.to_string())
        .or_else(|| manifest.package.as_ref().and_then(|p| p.r#type.clone()))
        .unwrap_or_else(|| "app-image".to_string())
        .to_lowercase();

    // Si se especificó 'nsis' o 'exe' en Windows, preparar el pipeline
    if cfg!(target_os = "windows") && (pkg_type == "nsis" || pkg_type == "setup") {
        pkg_type = "nsis".to_string();
    }

    // Validar tipo de paquete según el sistema operativo (Windows y Linux)
    let valid_types = if cfg!(target_os = "linux") {
        vec!["app-image"]
    } else if cfg!(target_os = "windows") {
        vec!["msi", "app-image", "exe", "nsis", "setup"]
    } else {
        vec!["app-image"]
    };

    if !valid_types.contains(&pkg_type.as_str()) {
        return Err(format!(
            "Tipo de paquete '{}' no soportado en esta plataforma ({:?}). Opciones válidas: {}",
            pkg_type,
            std::env::consts::OS,
            valid_types.join(", ")
        ).into());
    }

    // 6. Determinar directorio de destino
    let dest_rel = cli_dest
        .map(|s| s.to_string())
        .or_else(|| manifest.package.as_ref().and_then(|p| p.dest.clone()))
        .unwrap_or_else(|| "dist".to_string());
    let dest_dir = project_dir.join(&dest_rel);
    fs::create_dir_all(&dest_dir)?;

    // 7. Preparar staging Fat-JAR en target/package_input/app.jar
    println!("[INFO] [1/5] Compilando clases y preparando staging Fat-JAR...");
    let package_input_dir = project_dir.join("target").join("package_input");
    if package_input_dir.exists() {
        let _ = fs::remove_dir_all(&package_input_dir);
    }
    fs::create_dir_all(&package_input_dir)?;

    let standalone_jar = build_standalone_jar(
        project_dir,
        &app_name,
        &sanitized_version,
        &main_class,
        toolchain,
    )?;

    let staged_jar_path = package_input_dir.join("app.jar");
    fs::copy(&standalone_jar, &staged_jar_path)?;

    let jpackage_bin = toolchain
        .map(|t| t.jpackage_bin.as_path())
        .unwrap_or_else(|| Path::new("jpackage"));

    let icon_path = cli_icon
        .map(|s| project_dir.join(s))
        .or_else(|| manifest.package.as_ref().and_then(|p| p.icon.as_ref().map(|s| project_dir.join(s))));

    let vendor_str = manifest.package.as_ref().and_then(|p| p.vendor.clone()).unwrap_or_else(|| "Jolt App".to_string());
    let _desc_str = manifest.package.as_ref().and_then(|p| p.description.clone()).unwrap_or_else(|| "Aplicación construida con Jolt".to_string());

    // Manejo especial: Instalador NSIS en Windows
    if cfg!(target_os = "windows") && pkg_type == "nsis" {
        let makensis_bin = find_makensis_binary().ok_or(
            "makensis (NSIS) no fue encontrado en PATH ni en las rutas estándar. Por favor instala NSIS (ej: scoop install nsis o https://nsis.sourceforge.io/)."
        )?;

        // 7.A: Construir App-Image temporal
        println!("[INFO] [2/5] Generando App-Image intermedia con jpackage...");
        let staging_app_dir = project_dir.join("target").join("staging_app_image");
        if staging_app_dir.exists() {
            let _ = fs::remove_dir_all(&staging_app_dir);
        }
        fs::create_dir_all(&staging_app_dir)?;

        let mut cmd = Command::new(jpackage_bin);
        cmd.arg("--input").arg(&package_input_dir)
            .arg("--main-jar").arg("app.jar")
            .arg("--main-class").arg(&main_class)
            .arg("--name").arg(&app_name)
            .arg("--app-version").arg(&sanitized_version)
            .arg("--dest").arg(&staging_app_dir)
            .arg("--type").arg("app-image");

        if let Some(pkg_cfg) = &manifest.package {
            if let Some(vendor) = &pkg_cfg.vendor { cmd.arg("--vendor").arg(vendor); }
            if let Some(desc) = &pkg_cfg.description { cmd.arg("--description").arg(desc); }
            if let Some(copyright) = &pkg_cfg.copyright { cmd.arg("--copyright").arg(copyright); }
            if let Some(opts) = &pkg_cfg.java_options {
                for opt in opts { cmd.arg("--java-options").arg(opt); }
            }
        }
        if let Some(cli_opts) = cli_java_options {
            for opt in cli_opts.split_whitespace() { cmd.arg("--java-options").arg(opt); }
        }
        if let Some(ref icon) = icon_path {
            if icon.exists() { cmd.arg("--icon").arg(icon); }
        }
        if verbose { cmd.arg("--verbose"); }

        let output = cmd.output()?;
        if !output.status.success() {
            let _ = fs::remove_dir_all(&package_input_dir);
            let _ = fs::remove_dir_all(&staging_app_dir);
            let err_str = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Fallo al generar app-image intermedia:\n{}", err_str).into());
        }

        let inner_app_dir = staging_app_dir.join(&app_name);

        // 7.B: Optimización UPX si fue solicitada
        if is_upx_requested {
            println!("[INFO] [3/5] Comprimiendo binarios con UPX para el instalador...");
            let custom_upx_args = win_cfg.and_then(|w| w.upx_args.as_deref());
            let _ = compress_with_upx(&inner_app_dir, custom_upx_args, verbose);
        } else {
            println!("[INFO] [3/5] Compresión UPX omitida.");
        }

        // 7.C: Generar script NSIS
        println!("[INFO] [4/5] Generando script NSIS profesional...");
        let exe_name = format!("{}.exe", app_name);
        let final_installer_exe = dest_dir.join(format!("{}-{}-setup.exe", app_name, sanitized_version));
        let nsis_script_content = generate_nsis_script(
            &app_name,
            &sanitized_version,
            &vendor_str,
            icon_path.as_deref(),
            &inner_app_dir,
            &exe_name,
            is_add_to_path,
            desktop_shortcut,
            start_menu,
            &scope_str,
            license_file.as_deref(),
            &final_installer_exe,
        );

        let nsis_script_path = project_dir.join("target").join("installer.nsi");
        fs::write(&nsis_script_path, nsis_script_content)?;

        // 7.D: Compilar instalador con makensis
        println!("[INFO] [5/5] Compilando instalador con NSIS (makensis)...");
        let mut nsis_cmd = Command::new(&makensis_bin);
        nsis_cmd.arg(&nsis_script_path);

        let nsis_out = nsis_cmd.output()?;
        if !nsis_out.status.success() {
            let err = String::from_utf8_lossy(&nsis_out.stderr);
            let out = String::from_utf8_lossy(&nsis_out.stdout);
            let msg = if !err.trim().is_empty() { err } else { out };
            return Err(format!("Error en makensis al generar instalador:\n{}", msg).into());
        }

        // Limpiar archivos temporales
        let _ = fs::remove_dir_all(&package_input_dir);
        let _ = fs::remove_dir_all(&staging_app_dir);

        if final_installer_exe.exists() {
            let size = final_installer_exe.metadata().map(|m| m.len()).unwrap_or(0);
            println!(
                "[OK] Instalador NSIS generado exitosamente: {} ({:.2} MB)",
                final_installer_exe.display(),
                size as f64 / (1024.0 * 1024.0)
            );
        }

        return Ok(final_installer_exe);
    }

    // Flujo estándar jpackage
    println!("[INFO] [2/3] Empaquetando aplicación con jpackage (tipo: '{}', clase principal: '{}')...", pkg_type, main_class);
    let target_app_dir = dest_dir.join(&app_name);
    if target_app_dir.exists() {
        let _ = fs::remove_dir_all(&target_app_dir);
    }

    let mut cmd = Command::new(jpackage_bin);
    cmd.arg("--input").arg(&package_input_dir)
        .arg("--main-jar").arg("app.jar")
        .arg("--main-class").arg(&main_class)
        .arg("--name").arg(&app_name)
        .arg("--app-version").arg(&sanitized_version)
        .arg("--dest").arg(&dest_dir)
        .arg("--type").arg(&pkg_type);

    if let Some(pkg_cfg) = &manifest.package {
        if let Some(vendor) = &pkg_cfg.vendor { cmd.arg("--vendor").arg(vendor); }
        if let Some(desc) = &pkg_cfg.description { cmd.arg("--description").arg(desc); }
        if let Some(copyright) = &pkg_cfg.copyright { cmd.arg("--copyright").arg(copyright); }
        if let Some(opts) = &pkg_cfg.java_options {
            for opt in opts { cmd.arg("--java-options").arg(opt); }
        }
    }
    if let Some(cli_opts) = cli_java_options {
        for opt in cli_opts.split_whitespace() { cmd.arg("--java-options").arg(opt); }
    }
    if let Some(ref icon) = icon_path {
        if icon.exists() { cmd.arg("--icon").arg(icon); }
    }
    if verbose { cmd.arg("--verbose"); }

    let output = if verbose {
        let s = cmd.status()?;
        if s.success() { Ok(()) } else { Err(format!("jpackage falló con código: {:?}", s.code())) }
    } else {
        let out = cmd.output()?;
        if out.status.success() {
            Ok(())
        } else {
            let err_str = String::from_utf8_lossy(&out.stderr);
            let out_str = String::from_utf8_lossy(&out.stdout);
            let full_msg = if !err_str.trim().is_empty() { err_str.to_string() } else { out_str.to_string() };
            Err(format!("Error en jpackage:\n{}", full_msg))
        }
    };

    if let Err(e) = output {
        let _ = fs::remove_dir_all(&package_input_dir);
        return Err(e.into());
    }

    // Limpiar staging
    let _ = fs::remove_dir_all(&package_input_dir);

    // Si es app-image y se pidió UPX, optimizar el directorio de salida
    if pkg_type == "app-image" && is_upx_requested {
        println!("[INFO] [3/3] Comprimiendo binarios de la aplicación con UPX...");
        let custom_upx_args = win_cfg.and_then(|w| w.upx_args.as_deref());
        let app_out_dir = dest_dir.join(&app_name);
        let _ = compress_with_upx(&app_out_dir, custom_upx_args, verbose);
    }

    // Determinar ruta resultante para el usuario
    let output_path = if pkg_type == "app-image" {
        if cfg!(target_os = "windows") {
            dest_dir.join(&app_name).join(format!("{}.exe", app_name))
        } else {
            dest_dir.join(&app_name).join("bin").join(&app_name)
        }
    } else if pkg_type == "msi" {
        let candidate_msi = dest_dir.join(format!("{}-{}.msi", app_name, sanitized_version));
        if candidate_msi.exists() {
            candidate_msi
        } else {
            dest_dir.clone()
        }
    } else {
        dest_dir.clone()
    };

    Ok(output_path)
}

use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::build::compile;
use crate::toolchain::Toolchain;

/// Empaqueta el proyecto en un archivo JAR estándar
pub fn build_jar(
    project_dir: &Path,
    project_name: &str,
    version: &str,
    main_class: &str,
    toolchain: Option<&Toolchain>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    compile(project_dir, toolchain)?;

    let target_dir = project_dir.join("target");
    let jar_file = target_dir.join(format!("{}-{}.jar", project_name, version));
    let classes_dir = target_dir.join("classes");

    let jar_path = toolchain
        .map(|t| t.jar_bin.as_path())
        .unwrap_or_else(|| Path::new("jar"));

    let mut cmd = Command::new(jar_path);
    cmd.arg("cfe")
        .arg(&jar_file)
        .arg(main_class)
        .arg("-C")
        .arg(&classes_dir)
        .arg(".");

    let output = cmd.output()?;
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error al empaquetar JAR:\n{}", error_msg).into());
    }

    Ok(jar_file)
}

/// Empaqueta todas las clases y dependencias en un único Fat-JAR autónomo (Uber-JAR)
pub fn build_standalone_jar(
    project_dir: &Path,
    project_name: &str,
    version: &str,
    main_class: &str,
    toolchain: Option<&Toolchain>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    // Asegurar que el proyecto esté compilado
    compile(project_dir, toolchain)?;

    let target_dir = project_dir.join("target");
    fs::create_dir_all(&target_dir)?;

    let standalone_jar_path = target_dir.join(format!("{}-{}-standalone.jar", project_name, version));
    let file = File::create(&standalone_jar_path)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut added_entries = HashSet::new();

    // 1. Escribir META-INF/MANIFEST.MF
    zip.start_file("META-INF/MANIFEST.MF", options)?;
    let manifest_content = format!(
        "Manifest-Version: 1.0\r\nMain-Class: {}\r\nCreated-By: Jolt {}\r\n\r\n",
        main_class,
        env!("CARGO_PKG_VERSION")
    );

    zip.write_all(manifest_content.as_bytes())?;
    added_entries.insert("META-INF/MANIFEST.MF".to_string());
    added_entries.insert("META-INF/".to_string());

    // 2. Añadir clases y recursos del proyecto desde target/classes
    let classes_dir = target_dir.join("classes");
    if classes_dir.is_dir() {
        for entry in WalkDir::new(&classes_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Ok(rel_path) = path.strip_prefix(&classes_dir) {
                let rel_str = rel_path.to_string_lossy().replace('\\', "/");
                if rel_str.is_empty() {
                    continue;
                }

                if path.is_file() {
                    if !added_entries.contains(&rel_str) {
                        added_entries.insert(rel_str.clone());
                        zip.start_file(&rel_str, options)?;
                        let mut f = File::open(path)?;
                        let mut buf = Vec::new();
                        f.read_to_end(&mut buf)?;
                        zip.write_all(&buf)?;
                    }
                }
            }
        }
    }

    // 2.1 Añadir clases y recursos de dependencias locales (path dependencies)
    let manifest_path = project_dir.join("jolt.toml");
    if let Ok(manifest) = crate::core::manifest::JoltManifest::load_from_file(&manifest_path) {
        if let Some(deps) = &manifest.dependencies {
            for (_name, spec) in deps {
                let (_ver, local_path_opt) = crate::core::manifest::JoltManifest::parse_dependency_spec(spec);
                if let Some(rel_path) = local_path_opt {
                    let dep_dir = project_dir.join(&rel_path);
                    let dep_classes = dep_dir.join("target").join("classes");
                    if dep_classes.is_dir() {
                        for entry in WalkDir::new(&dep_classes).into_iter().filter_map(|e| e.ok()) {
                            let path = entry.path();
                            if let Ok(rel_path) = path.strip_prefix(&dep_classes) {
                                let rel_str = rel_path.to_string_lossy().replace('\\', "/");
                                if !rel_str.is_empty() && path.is_file() && !added_entries.contains(&rel_str) {
                                    added_entries.insert(rel_str.clone());
                                    zip.start_file(&rel_str, options)?;
                                    let mut f = File::open(path)?;
                                    let mut buf = Vec::new();
                                    f.read_to_end(&mut buf)?;
                                    zip.write_all(&buf)?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Extraer y fusionar cada dependencia .jar en .jolt/modules/ (solo producción)
    let modules_dir = project_dir.join(".jolt").join("modules");
    if modules_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&modules_dir) {
            for entry in entries.flatten() {
                let jar_path = entry.path();
                if jar_path.extension().and_then(|s| s.to_str()) == Some("jar") {
                    if let Ok(jar_file) = File::open(&jar_path) {
                        if let Ok(mut archive) = ZipArchive::new(jar_file) {
                            for i in 0..archive.len() {
                                if let Ok(mut zip_entry) = archive.by_index(i) {
                                    let name = zip_entry.name().to_string();

                                    // Filtrar firmas digitales y manifiestos de librerías para evitar SecurityException
                                    if name.starts_with("META-INF/") && (
                                        name.ends_with(".SF") ||
                                        name.ends_with(".DSA") ||
                                        name.ends_with(".RSA") ||
                                        name == "META-INF/MANIFEST.MF" ||
                                        name == "META-INF/INDEX.LIST"
                                    ) {
                                        continue;
                                    }

                                    if zip_entry.is_file() && !added_entries.contains(&name) {
                                        added_entries.insert(name.clone());
                                        zip.start_file(&name, options)?;
                                        let mut buf = Vec::new();
                                        zip_entry.read_to_end(&mut buf)?;
                                        zip.write_all(&buf)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    zip.finish()?;
    Ok(standalone_jar_path)
}

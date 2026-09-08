use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use crate::toolchain::Toolchain;

/// Recopila todos los archivos .java recursivamente dentro de un directorio
pub fn collect_java_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    files.extend(collect_java_files(&path));
                } else if path.extension().and_then(|s| s.to_str()) == Some("java") {
                    files.push(path);
                }
            }
        }
    }
    files
}

/// Copia recursivamente los recursos estáticos desde src/main/resources a target/classes
pub fn copy_resources(project_dir: &Path, target_classes: &Path) {
    let resources_dirs = [
        project_dir.join("src").join("main").join("resources"),
        project_dir.join("src").join("resources"),
    ];

    for res_dir in &resources_dirs {
        if res_dir.is_dir() {
            for entry in WalkDir::new(res_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(rel_path) = path.strip_prefix(res_dir) {
                        let dest = target_classes.join(rel_path);
                        if let Some(parent) = dest.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        let _ = fs::copy(path, dest);
                    }
                }
            }
        }
    }
}

/// Construye el classpath a partir de `.jolt/modules/`, dependencias locales (path dependencies) y directorios adicionales
pub fn build_classpath(project_dir: &Path, include_classes: bool) -> String {
    let mut parts = Vec::new();

    if include_classes {
        let classes_dir = project_dir.join("target").join("classes");
        if classes_dir.exists() {
            parts.push(classes_dir.to_string_lossy().to_string());
        }
    }

    let modules_dir = project_dir.join(".jolt").join("modules");
    if modules_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&modules_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                    parts.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    // Dependencias locales inter-módulos (path dependencies)
    let manifest_path = project_dir.join("jolt.toml");
    if let Ok(manifest) = crate::core::manifest::JoltManifest::load_from_file(&manifest_path) {
        if let Some(deps) = &manifest.dependencies {
            for (_name, spec) in deps {
                let (_ver, local_path_opt) = crate::core::manifest::JoltManifest::parse_dependency_spec(spec);
                if let Some(rel_path) = local_path_opt {
                    let dep_dir = project_dir.join(&rel_path);
                    let dep_classes = dep_dir.join("target").join("classes");
                    if dep_classes.exists() {
                        parts.push(dep_classes.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    let separator = if cfg!(windows) { ";" } else { ":" };
    parts.join(separator)
}

/// Construye el classpath para pruebas incluyendo `.jolt/modules/`, `.jolt/dev-modules/` y dependencias locales
pub fn build_test_classpath(project_dir: &Path, include_classes: bool) -> String {
    let mut parts = Vec::new();

    if include_classes {
        let classes_dir = project_dir.join("target").join("classes");
        if classes_dir.exists() {
            parts.push(classes_dir.to_string_lossy().to_string());
        }
    }

    let search_dirs = [
        project_dir.join(".jolt").join("modules"),
        project_dir.join(".jolt").join("dev-modules"),
    ];

    for dir in &search_dirs {
        if dir.is_dir() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                        parts.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    // Dependencias locales inter-módulos
    let manifest_path = project_dir.join("jolt.toml");
    if let Ok(manifest) = crate::core::manifest::JoltManifest::load_from_file(&manifest_path) {
        if let Some(deps) = &manifest.dependencies {
            for (_name, spec) in deps {
                let (_ver, local_path_opt) = crate::core::manifest::JoltManifest::parse_dependency_spec(spec);
                if let Some(rel_path) = local_path_opt {
                    let dep_dir = project_dir.join(&rel_path);
                    let dep_classes = dep_dir.join("target").join("classes");
                    if dep_classes.exists() {
                        parts.push(dep_classes.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    let separator = if cfg!(windows) { ";" } else { ":" };
    parts.join(separator)
}

/// Compila todos los archivos .java de `src/` colocando los .class en `target/classes/`
pub fn compile(
    project_dir: &Path,
    toolchain: Option<&Toolchain>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    // Compilar dependencias locales previas si existen
    let manifest_path = project_dir.join("jolt.toml");
    if let Ok(manifest) = crate::core::manifest::JoltManifest::load_from_file(&manifest_path) {
        if let Some(deps) = &manifest.dependencies {
            for (_name, spec) in deps {
                let (_ver, local_path_opt) = crate::core::manifest::JoltManifest::parse_dependency_spec(spec);
                if let Some(rel_path) = local_path_opt {
                    let dep_dir = project_dir.join(&rel_path);
                    let dep_manifest_path = dep_dir.join("jolt.toml");
                    if dep_manifest_path.exists() {
                        let _ = compile(&dep_dir, toolchain);
                    }
                }
            }
        }
    }

    let main_src_dirs = [
        project_dir.join("src").join("main").join("java"),
        project_dir.join("src").join("main"),
    ];

    let mut java_files = Vec::new();
    let mut found_main_dir = false;
    for dir in &main_src_dirs {
        if dir.is_dir() {
            java_files.extend(collect_java_files(dir));
            found_main_dir = true;
        }
    }

    // Si no existe estructura src/main/, buscar en src/ excluyendo src/test/
    if !found_main_dir {
        let src_dir = project_dir.join("src");
        let all_files = collect_java_files(&src_dir);
        let test_dir = project_dir.join("src").join("test");
        for f in all_files {
            if !f.starts_with(&test_dir) {
                java_files.push(f);
            }
        }
    }

    if java_files.is_empty() {
        return Err("No se encontraron archivos .java en el directorio fuente principal ('src/main/java' o 'src/')".into());
    }

    let target_classes = project_dir.join("target").join("classes");
    fs::create_dir_all(&target_classes)?;

    // Copiar recursos estáticos al classpath
    copy_resources(project_dir, &target_classes);

    let classpath = build_classpath(project_dir, false);

    let javac_path = toolchain
        .map(|t| t.javac_bin.as_path())
        .unwrap_or_else(|| Path::new("javac"));

    let mut cmd = Command::new(javac_path);
    cmd.arg("-d").arg(&target_classes);

    if !classpath.is_empty() {
        cmd.arg("-cp").arg(&classpath);
    }

    for file in &java_files {
        cmd.arg(file);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Fallo en la compilación Java:\n{}", error_msg).into());
    }

    Ok(target_classes)
}

/// Detecta automáticamente la clase principal analizando el código fuente en `src/`
pub fn detect_main_class(project_dir: &Path) -> Option<String> {
    let src_dirs = [
        project_dir.join("src").join("main").join("java"),
        project_dir.join("src").join("main"),
        project_dir.join("src"),
    ];

    let test_dir = project_dir.join("src").join("test");

    for dir in &src_dirs {
        if dir.is_dir() {
            let java_files = collect_java_files(dir);
            for file_path in java_files {
                if file_path.starts_with(&test_dir) {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&file_path) {
                    if content.contains("static void main") {
                        let mut package_name: Option<String> = None;
                        let mut class_name: Option<String> = None;

                        for line in content.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("package ") && trimmed.ends_with(';') {
                                package_name = Some(
                                    trimmed[8..trimmed.len() - 1].trim().to_string()
                                );
                            }
                            if class_name.is_none() {
                                if let Some(pos) = trimmed.find("class ") {
                                    if !trimmed.starts_with("//") && !trimmed.starts_with("/*") && !trimmed.starts_with('*') {
                                        let after_class = trimmed[pos + 6..].trim();
                                        let name = after_class
                                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                                            .next()
                                            .unwrap_or("");
                                        if !name.is_empty() {
                                            class_name = Some(name.to_string());
                                        }
                                    }
                                }
                            }
                        }

                        let resolved_class = class_name.or_else(|| {
                            file_path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string())
                        });

                        if let Some(cls) = resolved_class {
                            if let Some(pkg) = package_name {
                                return Some(format!("{}.{}", pkg, cls));
                            } else {
                                return Some(cls);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

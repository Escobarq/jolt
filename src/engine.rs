use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub struct BuildEngine;

impl BuildEngine {
    /// Recopila todos los archivos .java recursivamente dentro de un directorio
    pub fn collect_java_files(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if dir.is_dir() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        files.extend(Self::collect_java_files(&path));
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

    /// Construye el classpath a partir de `.jolt/modules/` y directorios adicionales
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

        let separator = if cfg!(windows) { ";" } else { ":" };
        parts.join(separator)
    }

    /// Compila todos los archivos .java de `src/` colocando los .class en `target/classes/`
    pub fn compile(
        project_dir: &Path,
        toolchain: Option<&crate::toolchain::Toolchain>,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        let src_dir = project_dir.join("src");
        let java_files = Self::collect_java_files(&src_dir);

        if java_files.is_empty() {
            return Err("No se encontraron archivos .java en el directorio 'src/'".into());
        }

        let target_classes = project_dir.join("target").join("classes");
        fs::create_dir_all(&target_classes)?;

        // Copiar recursos estáticos al classpath
        Self::copy_resources(project_dir, &target_classes);

        let classpath = Self::build_classpath(project_dir, false);

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

    /// Ejecuta la aplicación Java con el classpath completo
    pub fn run(
        project_dir: &Path,
        main_class: &str,
        toolchain: Option<&crate::toolchain::Toolchain>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Asegurar que esté compilado
        Self::compile(project_dir, toolchain)?;

        let classpath = Self::build_classpath(project_dir, true);

        let java_path = toolchain
            .map(|t| t.java_bin.as_path())
            .unwrap_or_else(|| Path::new("java"));

        let mut cmd = Command::new(java_path);
        if !classpath.is_empty() {
            cmd.arg("-cp").arg(&classpath);
        }
        cmd.arg(main_class);

        // Heredar stdio para streaming interactivo en tiempo real
        let mut child = cmd.spawn()?;
        let status = child.wait()?;

        if !status.success() {
            return Err(format!("El programa terminó con código de salida: {:?}", status.code()).into());
        }

        Ok(())
    }

    /// Empaqueta el proyecto en un archivo JAR estándar
    pub fn build_jar(
        project_dir: &Path,
        project_name: &str,
        version: &str,
        main_class: &str,
        toolchain: Option<&crate::toolchain::Toolchain>,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        Self::compile(project_dir, toolchain)?;

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
        toolchain: Option<&crate::toolchain::Toolchain>,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        // Asegurar que el proyecto esté compilado
        Self::compile(project_dir, toolchain)?;

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
            "Manifest-Version: 1.0\r\nMain-Class: {}\r\nCreated-By: Jolt 0.1.0\r\n\r\n",
            main_class
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

        // 3. Extraer y fusionar cada dependencia .jar en .jolt/modules/
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_java_files() {
        let temp_dir = std::env::temp_dir().join("jolt_test_build");
        let src_dir = temp_dir.join("src").join("main").join("java");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&src_dir).unwrap();

        fs::write(src_dir.join("Main.java"), "public class Main {}").unwrap();
        fs::write(src_dir.join("Util.java"), "public class Util {}").unwrap();
        fs::write(src_dir.join("README.txt"), "Ignored file").unwrap();

        let java_files = BuildEngine::collect_java_files(&temp_dir.join("src"));
        assert_eq!(java_files.len(), 2);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}

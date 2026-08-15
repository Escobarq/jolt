use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

    /// Empaqueta el proyecto en un archivo JAR ejecutable
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

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::toolchain::Toolchain;
use super::compiler::{build_test_classpath, collect_java_files, compile};

/// Compila los archivos de prueba en `src/test/` colocando los .class en `target/test-classes/`
pub fn compile_tests(
    project_dir: &Path,
    toolchain: Option<&Toolchain>,
    junit_jar: &Path,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    // Asegurar que el código principal esté compilado
    compile(project_dir, toolchain)?;

    let test_src_dirs = [
        project_dir.join("src").join("test").join("java"),
        project_dir.join("src").join("test"),
    ];

    let mut test_files = Vec::new();
    for dir in &test_src_dirs {
        if dir.is_dir() {
            test_files.extend(collect_java_files(dir));
        }
    }

    if test_files.is_empty() {
        return Err("No se encontraron archivos de prueba en 'src/test/java/'".into());
    }

    let target_test_classes = project_dir.join("target").join("test-classes");
    fs::create_dir_all(&target_test_classes)?;

    // Classpath para tests: target/classes + dependencias prod + dev-dependencies + junit_jar
    let base_cp = build_test_classpath(project_dir, true);
    let separator = if cfg!(windows) { ";" } else { ":" };
    let test_cp = if base_cp.is_empty() {
        junit_jar.to_string_lossy().to_string()
    } else {
        format!("{}{}{}", base_cp, separator, junit_jar.display())
    };

    let javac_path = toolchain
        .map(|t| t.javac_bin.as_path())
        .unwrap_or_else(|| Path::new("javac"));

    let mut cmd = Command::new(javac_path);
    cmd.arg("-d").arg(&target_test_classes)
        .arg("-cp").arg(&test_cp);

    for file in &test_files {
        cmd.arg(file);
    }

    let output = cmd.output()?;
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Fallo en la compilación de pruebas Java:\n{}", error_msg).into());
    }

    Ok(target_test_classes)
}

/// Ejecuta la suite de pruebas JUnit 5 mediante el Launcher de consola
pub fn run_tests(
    project_dir: &Path,
    toolchain: Option<&Toolchain>,
    junit_jar: &Path,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Compilar tests y código principal
    compile_tests(project_dir, toolchain, junit_jar)?;

    let target_classes = project_dir.join("target").join("classes");
    let target_test_classes = project_dir.join("target").join("test-classes");
    let base_cp = build_test_classpath(project_dir, false);

    let separator = if cfg!(windows) { ";" } else { ":" };
    let mut scan_cp = format!("{}{}{}", target_test_classes.display(), separator, target_classes.display());
    if !base_cp.is_empty() {
        scan_cp = format!("{}{}{}", scan_cp, separator, base_cp);
    }

    let java_path = toolchain
        .map(|t| t.java_bin.as_path())
        .unwrap_or_else(|| Path::new("java"));

    let mut cmd = Command::new(java_path);
    cmd.arg("-jar").arg(junit_jar)
        .arg("execute")
        .arg("--class-path").arg(&scan_cp)
        .arg("--scan-class-path")
        .arg("--disable-banner")
        .arg("--details=tree");

    let mut child = cmd.spawn()?;
    let status = child.wait()?;

    if !status.success() {
        return Err(format!("Las pruebas unitarias terminaron con errores (código: {:?})", status.code()).into());
    }

    Ok(())
}

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::error::Error;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use super::compiler::{build_classpath, compile};
use crate::toolchain::Toolchain;

/// Ejecuta la aplicación Java con el classpath completo
pub fn run(
    project_dir: &Path,
    main_class: &str,
    toolchain: Option<&Toolchain>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Asegurar que esté compilado
    compile(project_dir, toolchain)?;

    let classpath = build_classpath(project_dir, true);

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
        return Err(format!(
            "El programa terminó con código de salida: {:?}",
            status.code()
        )
        .into());
    }

    Ok(())
}

/// Inicia un subproceso hijo de la aplicación Java
pub fn spawn_process(
    project_dir: &Path,
    main_class: &str,
    toolchain: Option<&Toolchain>,
) -> Result<std::process::Child, Box<dyn Error + Send + Sync>> {
    let classpath = build_classpath(project_dir, true);
    let java_path = toolchain
        .map(|t| t.java_bin.as_path())
        .unwrap_or_else(|| Path::new("java"));

    let mut cmd = Command::new(java_path);
    if !classpath.is_empty() {
        cmd.arg("-cp").arg(&classpath);
    }
    cmd.arg(main_class);

    let child = cmd.spawn()?;
    Ok(child)
}

pub fn verify_start(
    project_dir: &Path,
    main_class: &str,
    toolchain: Option<&Toolchain>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    compile(project_dir, toolchain)?;
    let classpath = build_classpath(project_dir, true);
    let java_path = toolchain
        .map(|t| t.java_bin.as_path())
        .unwrap_or_else(|| Path::new("java"));
    let mut child = Command::new(java_path)
        .arg("-cp")
        .arg(&classpath)
        .arg(main_class)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    std::thread::sleep(Duration::from_millis(750));
    if let Some(status) = child.try_wait()? {
        if !status.success() {
            let mut stderr = String::new();
            if let Some(mut pipe) = child.stderr.take() {
                pipe.read_to_string(&mut stderr)?;
            }
            return Err(diagnose_class_not_found(&stderr).into());
        }
        return Ok(());
    }
    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}

fn diagnose_class_not_found(stderr: &str) -> String {
    if let Some(class_name) = stderr
        .split("ClassNotFoundException:")
        .nth(1)
        .and_then(|s| s.lines().next())
    {
        let class_name = class_name.trim();
        let suggestion = if class_name.starts_with("com.fasterxml.jackson") {
            "com.fasterxml.jackson.core:jackson-databind"
        } else if class_name.starts_with("org.eclipse.jetty") {
            "org.eclipse.jetty:jetty-server"
        } else if class_name.starts_with("org.slf4j") {
            "org.slf4j:slf4j-api"
        } else if class_name.starts_with("kotlin.") {
            "org.jetbrains.kotlin:kotlin-stdlib"
        } else {
            "la dependencia Maven que contiene esa clase"
        };
        return format!(
            "ClassNotFoundException: '{}'. Falta una dependencia en el classpath. Añádela con `jolt add {}` y ejecuta `jolt install`.",
            class_name, suggestion
        );
    }
    format!("La aplicación no pudo arrancar:\n{}", stderr.trim())
}

/// Ejecuta la aplicación en modo Watch / Hot Reload reaccionando a cambios de archivos
pub fn run_watch(
    project_dir: &Path,
    main_class: &str,
    toolchain: Option<&Toolchain>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("[INFO] Modo Watch activado. Observando cambios en 'src/' y 'jolt.toml'...");

    // Compilación inicial
    if let Err(e) = compile(project_dir, toolchain) {
        eprintln!("[ERROR] Error de compilacion inicial:\n{}", e);
    }

    let mut current_child = match spawn_process(project_dir, main_class, toolchain) {
        Ok(child) => Some(child),
        Err(e) => {
            eprintln!("[WARN] No se pudo iniciar el proceso Java inicial: {}", e);
            None
        }
    };

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    let src_dir = project_dir.join("src");
    if src_dir.exists() {
        watcher.watch(&src_dir, RecursiveMode::Recursive)?;
    }
    let manifest_path = project_dir.join("jolt.toml");
    if manifest_path.exists() {
        watcher.watch(&manifest_path, RecursiveMode::NonRecursive)?;
    }

    let debounce_duration = Duration::from_millis(300);
    let mut last_reload = Instant::now();

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                let should_reload = match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => true,
                    _ => false,
                };

                if should_reload && last_reload.elapsed() >= debounce_duration {
                    last_reload = Instant::now();
                    println!("\n[INFO] Cambio detectado en archivos. Recompilando...");

                    // Matar proceso anterior si sigue activo
                    if let Some(mut child) = current_child.take() {
                        let _ = child.kill();
                        let _ = child.wait();
                    }

                    // Recompilar
                    match compile(project_dir, toolchain) {
                        Ok(_) => {
                            println!("[INFO] Reiniciando aplicacion...");
                            match spawn_process(project_dir, main_class, toolchain) {
                                Ok(child) => current_child = Some(child),
                                Err(e) => eprintln!("[ERROR] Error al reiniciar: {}", e),
                            }
                        }
                        Err(e) => {
                            eprintln!("[ERROR] Error de compilacion:\n{}", e);
                            eprintln!("[INFO] Esperando correcciones para reintentar...");
                        }
                    }
                }
            }
            Ok(Err(e)) => eprintln!("[WARN] Error en observador de archivos: {:?}", e),
            Err(_) => break,
        }
    }

    if let Some(mut child) = current_child {
        let _ = child.kill();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::diagnose_class_not_found;

    #[test]
    fn explains_known_missing_framework_dependencies() {
        let cases = [
            (
                "java.lang.ClassNotFoundException: com.fasterxml.jackson.databind.ObjectMapper",
                "com.fasterxml.jackson.core:jackson-databind",
            ),
            (
                "java.lang.ClassNotFoundException: org.eclipse.jetty.server.Server",
                "org.eclipse.jetty:jetty-server",
            ),
            (
                "java.lang.ClassNotFoundException: org.slf4j.Logger",
                "org.slf4j:slf4j-api",
            ),
            (
                "java.lang.ClassNotFoundException: kotlin.Unit",
                "org.jetbrains.kotlin:kotlin-stdlib",
            ),
        ];

        for (stderr, artifact) in cases {
            let message = diagnose_class_not_found(stderr);
            assert!(message.contains("Falta una dependencia"));
            assert!(message.contains(artifact));
            assert!(message.contains("jolt add"));
        }
    }

    #[test]
    fn preserves_unknown_startup_errors() {
        let message = diagnose_class_not_found("java.lang.IllegalStateException: boom");

        assert_eq!(
            message,
            "La aplicación no pudo arrancar:\njava.lang.IllegalStateException: boom"
        );
    }
}

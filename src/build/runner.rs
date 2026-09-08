use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::error::Error;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use crate::toolchain::Toolchain;
use super::compiler::{build_classpath, compile};

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
        return Err(format!("El programa terminó con código de salida: {:?}", status.code()).into());
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

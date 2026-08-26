use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
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

    /// Construye el classpath a partir de `.jolt/modules/` y directorios adicionales (solo dependencias de producción)
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

    /// Construye el classpath para pruebas incluyendo `.jolt/modules/` (producción) y `.jolt/dev-modules/` (testing/desarrollo)
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

        let separator = if cfg!(windows) { ";" } else { ":" };
        parts.join(separator)
    }

    /// Compila todos los archivos .java de `src/` colocando los .class en `target/classes/`
    pub fn compile(
        project_dir: &Path,
        toolchain: Option<&crate::toolchain::Toolchain>,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        let main_src_dirs = [
            project_dir.join("src").join("main").join("java"),
            project_dir.join("src").join("main"),
        ];

        let mut java_files = Vec::new();
        let mut found_main_dir = false;
        for dir in &main_src_dirs {
            if dir.is_dir() {
                java_files.extend(Self::collect_java_files(dir));
                found_main_dir = true;
            }
        }

        // Si no existe estructura src/main/, buscar en src/ excluyendo src/test/
        if !found_main_dir {
            let src_dir = project_dir.join("src");
            let all_files = Self::collect_java_files(&src_dir);
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

    /// Inicia un subproceso hijo de la aplicación Java
    pub fn spawn_process(
        project_dir: &Path,
        main_class: &str,
        toolchain: Option<&crate::toolchain::Toolchain>,
    ) -> Result<std::process::Child, Box<dyn Error + Send + Sync>> {
        let classpath = Self::build_classpath(project_dir, true);
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
        toolchain: Option<&crate::toolchain::Toolchain>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("[INFO] Modo Watch activado. Observando cambios en 'src/' y 'jolt.toml'...");

        // Compilación inicial
        if let Err(e) = Self::compile(project_dir, toolchain) {
            eprintln!("[ERROR] Error de compilacion inicial:\n{}", e);
        }

        let mut current_child = match Self::spawn_process(project_dir, main_class, toolchain) {
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
                        match Self::compile(project_dir, toolchain) {
                            Ok(_) => {
                                println!("[INFO] Reiniciando aplicacion...");
                                match Self::spawn_process(project_dir, main_class, toolchain) {
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

    /// Compila los archivos de prueba en `src/test/` colocando los .class en `target/test-classes/`
    pub fn compile_tests(
        project_dir: &Path,
        toolchain: Option<&crate::toolchain::Toolchain>,
        junit_jar: &Path,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        // Asegurar que el código principal esté compilado
        Self::compile(project_dir, toolchain)?;

        let test_src_dirs = [
            project_dir.join("src").join("test").join("java"),
            project_dir.join("src").join("test"),
        ];

        let mut test_files = Vec::new();
        for dir in &test_src_dirs {
            if dir.is_dir() {
                test_files.extend(Self::collect_java_files(dir));
            }
        }

        if test_files.is_empty() {
            return Err("No se encontraron archivos de prueba en 'src/test/java/'".into());
        }

        let target_test_classes = project_dir.join("target").join("test-classes");
        fs::create_dir_all(&target_test_classes)?;

        // Classpath para tests: target/classes + dependencias prod + dev-dependencies + junit_jar
        let base_cp = Self::build_test_classpath(project_dir, true);
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
        toolchain: Option<&crate::toolchain::Toolchain>,
        junit_jar: &Path,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Compilar tests y código principal
        Self::compile_tests(project_dir, toolchain, junit_jar)?;

        let target_classes = project_dir.join("target").join("classes");
        let target_test_classes = project_dir.join("target").join("test-classes");
        let base_cp = Self::build_test_classpath(project_dir, false);

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
                let java_files = Self::collect_java_files(dir);
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

    /// Empaqueta la aplicación como un ejecutable binario autónomo o instalador usando jpackage
    pub fn package_native_app(
        project_dir: &Path,
        manifest: &crate::manifest::JoltManifest,
        cli_type: Option<&str>,
        cli_dest: Option<&str>,
        cli_name: Option<&str>,
        cli_app_version: Option<&str>,
        cli_main_class: Option<&str>,
        cli_icon: Option<&str>,
        cli_java_options: Option<&str>,
        verbose: bool,
        toolchain: Option<&crate::toolchain::Toolchain>,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        // 1. Determinar clase principal
        let main_class = cli_main_class
            .map(|s| s.to_string())
            .or_else(|| manifest.package.as_ref().and_then(|p| p.main_class.clone()))
            .or_else(|| manifest.project.main_class.clone())
            .or_else(|| Self::detect_main_class(project_dir))
            .unwrap_or_else(|| "Main".to_string());

        // 2. Determinar nombre de la aplicación / binario
        let raw_app_name = cli_name
            .map(|s| s.to_string())
            .or_else(|| manifest.package.as_ref().and_then(|p| p.name.clone()))
            .unwrap_or_else(|| manifest.project.name.clone());

        let app_name = Path::new(&raw_app_name)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(&raw_app_name)
            .replace(['/', '\\', ' ', ':'], "_");

        // 3. Determinar y sanitizar versión de la aplicación para jpackage (debe ser dígitos y puntos, ej. 1.0.0)
        let raw_version = cli_app_version
            .map(|s| s.to_string())
            .unwrap_or_else(|| manifest.project.version.clone());

        let sanitized_version: String = {
            let base = raw_version.split('-').next().unwrap_or("1.0.0").trim();
            let cleaned: String = base.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
            if cleaned.is_empty() {
                "1.0.0".to_string()
            } else {
                cleaned
            }
        };

        // 4. Determinar tipo de paquete
        let pkg_type = cli_type
            .map(|s| s.to_string())
            .or_else(|| manifest.package.as_ref().and_then(|p| p.r#type.clone()))
            .unwrap_or_else(|| "app-image".to_string())
            .to_lowercase();

        // Validar tipo de paquete según el sistema operativo
        let valid_types = if cfg!(target_os = "linux") {
            vec!["app-image", "deb", "rpm"]
        } else if cfg!(target_os = "windows") {
            vec!["app-image", "msi", "exe"]
        } else if cfg!(target_os = "macos") {
            vec!["app-image", "dmg", "pkg"]
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

        // 5. Determinar directorio de destino
        let dest_rel = cli_dest
            .map(|s| s.to_string())
            .or_else(|| manifest.package.as_ref().and_then(|p| p.dest.clone()))
            .unwrap_or_else(|| "dist".to_string());
        let dest_dir = project_dir.join(&dest_rel);
        fs::create_dir_all(&dest_dir)?;

        // Limpiar directorio previo de la aplicación si ya existía para permitir sobreescritura
        let target_app_dir = dest_dir.join(&app_name);
        if target_app_dir.exists() {
            let _ = fs::remove_dir_all(&target_app_dir);
        }

        // 6. Preparar staging Fat-JAR en target/package_input/app.jar
        let package_input_dir = project_dir.join("target").join("package_input");
        if package_input_dir.exists() {
            let _ = fs::remove_dir_all(&package_input_dir);
        }
        fs::create_dir_all(&package_input_dir)?;

        let standalone_jar = Self::build_standalone_jar(
            project_dir,
            &app_name,
            &sanitized_version,
            &main_class,
            toolchain,
        )?;

        let staged_jar_path = package_input_dir.join("app.jar");
        fs::copy(&standalone_jar, &staged_jar_path)?;

        // 7. Configurar comando jpackage
        let jpackage_bin = toolchain
            .map(|t| t.jpackage_bin.as_path())
            .unwrap_or_else(|| Path::new("jpackage"));

        let mut cmd = Command::new(jpackage_bin);
        cmd.arg("--input").arg(&package_input_dir)
            .arg("--main-jar").arg("app.jar")
            .arg("--main-class").arg(&main_class)
            .arg("--name").arg(&app_name)
            .arg("--app-version").arg(&sanitized_version)
            .arg("--dest").arg(&dest_dir)
            .arg("--type").arg(&pkg_type);

        // Metadatos adicionales desde [package]
        if let Some(pkg_cfg) = &manifest.package {
            if let Some(vendor) = &pkg_cfg.vendor {
                cmd.arg("--vendor").arg(vendor);
            }
            if let Some(desc) = &pkg_cfg.description {
                cmd.arg("--description").arg(desc);
            }
            if let Some(copyright) = &pkg_cfg.copyright {
                cmd.arg("--copyright").arg(copyright);
            }
            if let Some(opts) = &pkg_cfg.java_options {
                for opt in opts {
                    cmd.arg("--java-options").arg(opt);
                }
            }
        }

        // JVM options desde CLI
        if let Some(cli_opts) = cli_java_options {
            for opt in cli_opts.split_whitespace() {
                cmd.arg("--java-options").arg(opt);
            }
        }

        // Ícono
        let icon_path = cli_icon
            .map(|s| project_dir.join(s))
            .or_else(|| manifest.package.as_ref().and_then(|p| p.icon.as_ref().map(|s| project_dir.join(s))));

        if let Some(icon) = icon_path {
            if icon.exists() {
                cmd.arg("--icon").arg(&icon);
            } else {
                eprintln!("[WARN] Archivo de icono '{}' no encontrado, omitiendo.", icon.display());
            }
        }

        if verbose {
            cmd.arg("--verbose");
        }

        println!("[INFO] Empaquetando aplicación con jpackage (tipo: '{}', clase principal: '{}')...", pkg_type, main_class);

        let output = if verbose {
            let s = cmd.status()?;
            if s.success() {
                Ok(())
            } else {
                Err(format!("jpackage falló con código: {:?}", s.code()))
            }
        } else {
            let out = cmd.output()?;
            if out.status.success() {
                Ok(())
            } else {
                let err_str = String::from_utf8_lossy(&out.stderr);
                let out_str = String::from_utf8_lossy(&out.stdout);
                let full_msg = if !err_str.trim().is_empty() {
                    err_str.to_string()
                } else {
                    out_str.to_string()
                };
                Err(format!("Error en jpackage:\n{}", full_msg))
            }
        };

        if let Err(e) = output {
            let _ = fs::remove_dir_all(&package_input_dir);
            return Err(e.into());
        }

        // Limpiar staging
        let _ = fs::remove_dir_all(&package_input_dir);

        // 8. Determinar ruta resultante para el usuario
        let output_path = if pkg_type == "app-image" {
            if cfg!(target_os = "windows") {
                dest_dir.join(&app_name).join(format!("{}.exe", app_name))
            } else if cfg!(target_os = "macos") {
                dest_dir.join(format!("{}.app", app_name))
            } else {
                dest_dir.join(&app_name).join("bin").join(&app_name)
            }
        } else {
            dest_dir.clone()
        };

        Ok(output_path)
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

    #[test]
    fn test_build_classpath_separation() {
        let temp_dir = std::env::temp_dir().join("jolt_test_engine_cp");
        let _ = fs::remove_dir_all(&temp_dir);
        let prod_modules = temp_dir.join(".jolt").join("modules");
        let dev_modules = temp_dir.join(".jolt").join("dev-modules");
        let classes_dir = temp_dir.join("target").join("classes");

        fs::create_dir_all(&prod_modules).unwrap();
        fs::create_dir_all(&dev_modules).unwrap();
        fs::create_dir_all(&classes_dir).unwrap();

        fs::write(prod_modules.join("gson-2.14.0.jar"), b"dummy").unwrap();
        fs::write(dev_modules.join("junit-jupiter-api-5.10.2.jar"), b"dummy").unwrap();

        let prod_cp = BuildEngine::build_classpath(&temp_dir, false);
        assert!(prod_cp.contains("gson-2.14.0.jar"));
        assert!(!prod_cp.contains("junit-jupiter-api-5.10.2.jar"));

        let test_cp = BuildEngine::build_test_classpath(&temp_dir, true);
        assert!(test_cp.contains("classes"));
        assert!(test_cp.contains("gson-2.14.0.jar"));
        assert!(test_cp.contains("junit-jupiter-api-5.10.2.jar"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_main_class() {
        let temp_dir = std::env::temp_dir().join("jolt_test_detect_main");
        let _ = fs::remove_dir_all(&temp_dir);
        let src_dir = temp_dir.join("src").join("main").join("java").join("com").join("example");
        fs::create_dir_all(&src_dir).unwrap();

        let java_code = r#"
        package com.example;

        public class MyAwesomeApp {
            public static void main(String[] args) {
                System.out.println("Hello World");
            }
        }
        "#;
        fs::write(src_dir.join("MyAwesomeApp.java"), java_code).unwrap();

        let detected = BuildEngine::detect_main_class(&temp_dir);
        assert_eq!(detected, Some("com.example.MyAwesomeApp".to_string()));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}


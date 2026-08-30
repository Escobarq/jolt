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

    /// Construye el classpath a partir de `.jolt/modules/`, dependencias locales (path dependencies) y directorios adicionales (solo dependencias de producción)
    pub fn build_classpath(project_dir: &Path, include_classes: bool) -> String {
        let mut parts = Vec::new();

        if include_classes {
            let classes_dir = project_dir.join("target").join("classes");
            if classes_dir.exists() {
                parts.push(classes_dir.to_string_lossy().to_string());
            }
        }

        // 1. Módulos descargados en .jolt/modules
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

        // 2. Dependencias locales inter-módulos (path dependencies)
        let manifest_path = project_dir.join("jolt.toml");
        if let Ok(manifest) = crate::manifest::JoltManifest::load_from_file(&manifest_path) {
            if let Some(deps) = &manifest.dependencies {
                for (_name, spec) in deps {
                    let (_ver, local_path_opt) = crate::manifest::JoltManifest::parse_dependency_spec(spec);
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
        if let Ok(manifest) = crate::manifest::JoltManifest::load_from_file(&manifest_path) {
            if let Some(deps) = &manifest.dependencies {
                for (_name, spec) in deps {
                    let (_ver, local_path_opt) = crate::manifest::JoltManifest::parse_dependency_spec(spec);
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
        toolchain: Option<&crate::toolchain::Toolchain>,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        // Compilar dependencias locales previas si existen
        let manifest_path = project_dir.join("jolt.toml");
        if let Ok(manifest) = crate::manifest::JoltManifest::load_from_file(&manifest_path) {
            if let Some(deps) = &manifest.dependencies {
                for (_name, spec) in deps {
                    let (_ver, local_path_opt) = crate::manifest::JoltManifest::parse_dependency_spec(spec);
                    if let Some(rel_path) = local_path_opt {
                        let dep_dir = project_dir.join(&rel_path);
                        let dep_manifest_path = dep_dir.join("jolt.toml");
                        if dep_manifest_path.exists() {
                            let _ = Self::compile(&dep_dir, toolchain);
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

        // 2.1 Añadir clases y recursos de dependencias locales (path dependencies)
        let manifest_path = project_dir.join("jolt.toml");
        if let Ok(manifest) = crate::manifest::JoltManifest::load_from_file(&manifest_path) {
            if let Some(deps) = &manifest.dependencies {
                for (_name, spec) in deps {
                    let (_ver, local_path_opt) = crate::manifest::JoltManifest::parse_dependency_spec(spec);
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

    /// Localiza el compilador makensis (NSIS) en PATH o en directorios habituales de Windows
    pub fn find_makensis_binary() -> Option<PathBuf> {
        if let Ok(output) = Command::new("makensis").arg("/VERSION").output() {
            if output.status.success() {
                return Some(PathBuf::from("makensis"));
            }
        }

        if cfg!(target_os = "windows") {
            let candidates = [
                PathBuf::from("C:\\Program Files (x86)\\NSIS\\makensis.exe"),
                PathBuf::from("C:\\Program Files\\NSIS\\makensis.exe"),
            ];
            for candidate in &candidates {
                if candidate.is_file() {
                    return Some(candidate.clone());
                }
            }

            if let Some(home) = dirs::home_dir() {
                let scoop_nsis = home.join("scoop").join("apps").join("nsis").join("current").join("makensis.exe");
                if scoop_nsis.is_file() {
                    return Some(scoop_nsis);
                }
                let scoop_shim = home.join("scoop").join("shims").join("makensis.exe");
                if scoop_shim.is_file() {
                    return Some(scoop_shim);
                }
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
        let upx_bin = Self::find_upx_binary().ok_or("UPX no está instalado o no se encuentra en el sistema")?;

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

    /// Genera el script de NSIS para empaquetar una aplicación Windows
    pub fn generate_nsis_script(
        app_name: &str,
        app_version: &str,
        vendor: &str,
        icon_path: Option<&Path>,
        staging_dir: &Path,
        exe_name: &str,
        add_to_path: bool,
        desktop_shortcut: bool,
        start_menu: bool,
        scope: &str,
        license_path: Option<&Path>,
        output_installer: &Path,
    ) -> String {
        let is_per_machine = scope.eq_ignore_ascii_case("per-machine") || scope.eq_ignore_ascii_case("admin");
        let exec_level = if is_per_machine { "admin" } else { "user" };
        let reg_root = if is_per_machine { "HKLM" } else { "HKCU" };
        let default_dir = if is_per_machine {
            format!("$PROGRAMFILES64\\{}", app_name)
        } else {
            format!("$LOCALAPPDATA\\Programs\\{}", app_name)
        };

        let staging_dir_str = staging_dir.to_string_lossy().replace('\\', "/");
        let output_installer_str = output_installer.to_string_lossy().replace('\\', "/");

        let icon_section = if let Some(icon) = icon_path {
            let icon_str = icon.to_string_lossy().replace('\\', "/");
            format!(
                "!define MUI_ICON \"{}\"\n!define MUI_UNICON \"{}\"\n",
                icon_str, icon_str
            )
        } else {
            String::new()
        };

        let license_section = if let Some(lic) = license_path {
            let lic_str = lic.to_string_lossy().replace('\\', "/");
            format!("!insertmacro MUI_PAGE_LICENSE \"{}\"\n", lic_str)
        } else {
            String::new()
        };

        let mut shortcuts_code = String::new();
        let mut uninstall_shortcuts_code = String::new();

        if start_menu {
            shortcuts_code.push_str(&format!(
                "    CreateDirectory \"$SMPROGRAMS\\{}\"\n    CreateShortCut \"$SMPROGRAMS\\{}\\{}.lnk\" \"$INSTDIR\\{}\" \"\" \"$INSTDIR\\{}\" 0\n    CreateShortCut \"$SMPROGRAMS\\{}\\Desinstalar {}.lnk\" \"$INSTDIR\\uninstall.exe\" \"\" \"$INSTDIR\\uninstall.exe\" 0\n",
                app_name, app_name, app_name, exe_name, exe_name, app_name, app_name
            ));
            uninstall_shortcuts_code.push_str(&format!(
                "    Delete \"$SMPROGRAMS\\{}\\{}.lnk\"\n    Delete \"$SMPROGRAMS\\{}\\Desinstalar {}.lnk\"\n    RMDir \"$SMPROGRAMS\\{}\"\n",
                app_name, app_name, app_name, app_name, app_name
            ));
        }

        if desktop_shortcut {
            shortcuts_code.push_str(&format!(
                "    CreateShortCut \"$DESKTOP\\{}.lnk\" \"$INSTDIR\\{}\" \"\" \"$INSTDIR\\{}\" 0\n",
                app_name, exe_name, exe_name
            ));
            uninstall_shortcuts_code.push_str(&format!(
                "    Delete \"$DESKTOP\\{}.lnk\"\n",
                app_name
            ));
        }

        let (path_add_code, path_remove_code) = if add_to_path {
            let env_key = if is_per_machine {
                "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment"
            } else {
                "Environment"
            };

            let add = format!(
                r#"
    ; Agregar al PATH ({scope})
    DetailPrint "Configurando variable de entorno PATH..."
    ReadRegStr $0 {reg_root} "{env_key}" "PATH"
    ${{If}} $0 == ""
        WriteRegExpandStr {reg_root} "{env_key}" "PATH" "$INSTDIR"
    ${{Else}}
        Push "$0"
        Push "$INSTDIR"
        Call StrContains
        Pop $1
        ${{If}} $1 == "0"
            WriteRegExpandStr {reg_root} "{env_key}" "PATH" "$0;$INSTDIR"
        ${{EndIf}}
    ${{EndIf}}
    SendMessage ${{HWND_BROADCAST}} ${{WM_SETTINGCHANGE}} 0 "STR:Environment" /TIMEOUT=5000
"#,
                scope = if is_per_machine { "Sistema" } else { "Usuario" },
                reg_root = reg_root,
                env_key = env_key,
            );

            let remove = format!(
                r#"
    ; Remover del PATH ({scope})
    DetailPrint "Removiendo de la variable PATH..."
    ReadRegStr $0 {reg_root} "{env_key}" "PATH"
    Push "$0"
    Push "$INSTDIR;"
    Push ""
    Call un.StrReplace
    Pop $0

    Push "$0"
    Push ";$INSTDIR"
    Push ""
    Call un.StrReplace
    Pop $0

    Push "$0"
    Push "$INSTDIR"
    Push ""
    Call un.StrReplace
    Pop $0

    WriteRegExpandStr {reg_root} "{env_key}" "PATH" "$0"
    SendMessage ${{HWND_BROADCAST}} ${{WM_SETTINGCHANGE}} 0 "STR:Environment" /TIMEOUT=5000
"#,
                scope = if is_per_machine { "Sistema" } else { "Usuario" },
                reg_root = reg_root,
                env_key = env_key,
            );

            (add, remove)
        } else {
            (String::new(), String::new())
        };

        format!(
            r#"; Script NSIS generado automáticamente por Jolt
Unicode True
SetCompressor /SOLID lzma

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "WinMessages.nsh"

!define PRODUCT_NAME "{app_name}"
!define PRODUCT_VERSION "{app_version}"
!define PRODUCT_PUBLISHER "{vendor}"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\{exe_name}"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${{PRODUCT_NAME}}"
!define PRODUCT_UNINST_ROOT_KEY "{reg_root}"

Name "${{PRODUCT_NAME}} v${{PRODUCT_VERSION}}"
OutFile "{output_installer_str}"
InstallDir "{default_dir}"
InstallDirRegKey ${{PRODUCT_UNINST_ROOT_KEY}} "${{PRODUCT_UNINST_KEY}}" "UninstallString"
RequestExecutionLevel {exec_level}

!define MUI_ABORTWARNING
{icon_section}

!insertmacro MUI_PAGE_WELCOME
{license_section}!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

!define MUI_FINISHPAGE_TITLE "Instalación completada"
!define MUI_FINISHPAGE_TEXT "${{PRODUCT_NAME}} se ha instalado correctamente en su equipo."
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "Spanish"
!insertmacro MUI_LANGUAGE "English"

Function StrContains
  Exch $1 ; needle
  Exch
  Exch $0 ; haystack
  Push $2
  Push $3
  Push $4
  StrCpy $2 -1
  StrLen $3 $1
  loop:
    IntOp $2 $2 + 1
    StrCpy $4 $0 $3 $2
    StrCmp $4 "" notfound
    StrCmp $4 $1 found
    Goto loop
  found:
    StrCpy $0 "1"
    Goto done
  notfound:
    StrCpy $0 "0"
  done:
    Pop $4
    Pop $3
    Pop $2
    Pop $1
    Exch $0
FunctionEnd

Function un.StrReplace
  Exch $2 ; replacement
  Exch 1
  Exch $1 ; to replace
  Exch 2
  Exch $0 ; original
  Push $3
  Push $4
  Push $5
  Push $6
  Push $7
  StrLen $4 $1
  StrCpy $7 ""
  loop:
    StrCpy $3 $0 $4
    StrCmp $3 $1 match
    StrCmp $0 "" done
    StrCpy $5 $0 1
    StrCpy $7 "$7$5"
    StrCpy $0 $0 "" 1
    Goto loop
  match:
    StrCpy $7 "$7$2"
    StrCpy $0 $0 "" $4
    Goto loop
  done:
    StrCpy $0 $7
    Pop $7
    Pop $6
    Pop $5
    Pop $4
    Pop $3
    Pop $2
    Pop $1
    Exch $0
FunctionEnd

Section "MainSection" SEC01
    SetOutPath "$INSTDIR"
    File /r "{staging_dir_str}\*.*"
    
    WriteUninstaller "$INSTDIR\uninstall.exe"
    
{shortcuts_code}
    WriteRegStr ${{PRODUCT_UNINST_ROOT_KEY}} "${{PRODUCT_UNINST_KEY}}" "DisplayName" "${{PRODUCT_NAME}}"
    WriteRegStr ${{PRODUCT_UNINST_ROOT_KEY}} "${{PRODUCT_UNINST_KEY}}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
    WriteRegStr ${{PRODUCT_UNINST_ROOT_KEY}} "${{PRODUCT_UNINST_KEY}}" "DisplayIcon" "$\"$INSTDIR\{exe_name}$\""
    WriteRegStr ${{PRODUCT_UNINST_ROOT_KEY}} "${{PRODUCT_UNINST_KEY}}" "DisplayVersion" "${{PRODUCT_VERSION}}"
    WriteRegStr ${{PRODUCT_UNINST_ROOT_KEY}} "${{PRODUCT_UNINST_KEY}}" "Publisher" "${{PRODUCT_PUBLISHER}}"
    WriteRegStr ${{PRODUCT_UNINST_ROOT_KEY}} "${{PRODUCT_UNINST_KEY}}" "InstallLocation" "$INSTDIR"
    WriteRegStr ${{PRODUCT_UNINST_ROOT_KEY}} "${{PRODUCT_DIR_REGKEY}}" "" "$\"$INSTDIR\{exe_name}$\""
{path_add_code}
SectionEnd

Section "Uninstall"
{path_remove_code}
{uninstall_shortcuts_code}
    DeleteRegKey ${{PRODUCT_UNINST_ROOT_KEY}} "${{PRODUCT_UNINST_KEY}}"
    DeleteRegKey ${{PRODUCT_UNINST_ROOT_KEY}} "${{PRODUCT_DIR_REGKEY}}"
    RMDir /r "$INSTDIR"
SectionEnd
"#,
            app_name = app_name,
            app_version = app_version,
            vendor = vendor,
            exe_name = exe_name,
            reg_root = reg_root,
            output_installer_str = output_installer_str,
            default_dir = default_dir,
            exec_level = exec_level,
            icon_section = icon_section,
            license_section = license_section,
            staging_dir_str = staging_dir_str,
            shortcuts_code = shortcuts_code,
            path_add_code = path_add_code,
            path_remove_code = path_remove_code,
            uninstall_shortcuts_code = uninstall_shortcuts_code
        )
    }

    /// Empaqueta la aplicación como un ejecutable binario autónomo o instalador usando jpackage / NSIS / UPX
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
        cli_upx: Option<bool>,
        cli_add_to_path: Option<bool>,
        cli_scope: Option<&str>,
        verbose: bool,
        toolchain: Option<&crate::toolchain::Toolchain>,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        // 1. Determinar clase principal
        let main_class = cli_main_class
            .map(|s| s.to_string())
            .or_else(|| manifest.package.as_ref().and_then(|p| p.main_class.clone()))
            .or_else(|| manifest.project.as_ref().and_then(|p| p.main_class.clone()))
            .or_else(|| Self::detect_main_class(project_dir))
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

        let standalone_jar = Self::build_standalone_jar(
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
            let makensis_bin = Self::find_makensis_binary().ok_or(
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
                if let Some(v) = &pkg_cfg.vendor { cmd.arg("--vendor").arg(v); }
                if let Some(d) = &pkg_cfg.description { cmd.arg("--description").arg(d); }
                if let Some(c) = &pkg_cfg.copyright { cmd.arg("--copyright").arg(c); }
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

            let out = cmd.output()?;
            if !out.status.success() {
                let _ = fs::remove_dir_all(&package_input_dir);
                let _ = fs::remove_dir_all(&staging_app_dir);
                let err_str = String::from_utf8_lossy(&out.stderr);
                return Err(format!("Error en jpackage al generar app-image para NSIS:\n{}", err_str).into());
            }

            let app_image_folder = staging_app_dir.join(&app_name);
            let exe_name = format!("{}.exe", app_name);

            // 7.B: Compresión con UPX si está activada
            if is_upx_requested {
                println!("[INFO] [3/5] Comprimiendo ejecutables y bibliotecas con UPX...");
                let custom_upx_args = win_cfg.and_then(|w| w.upx_args.as_deref());
                match Self::compress_with_upx(&app_image_folder, custom_upx_args, verbose) {
                    Ok((orig, comp, count)) => {
                        if count > 0 && orig > comp {
                            let diff = orig - comp;
                            println!(
                                "  [OK] Optimización UPX completada: {:.2} MB -> {:.2} MB (Ahorro total: {:.2} MB en {} archivos)",
                                orig as f64 / (1024.0 * 1024.0),
                                comp as f64 / (1024.0 * 1024.0),
                                diff as f64 / (1024.0 * 1024.0),
                                count
                            );
                        } else {
                            println!("  [INFO] No se requirió compresión adicional.");
                        }
                    }
                    Err(e) => {
                        eprintln!("[WARN] Omitiendo compresión UPX: {}", e);
                    }
                }
            } else {
                println!("[INFO] [3/5] Compresión UPX omitida.");
            }

            // 7.C: Generar script NSIS
            println!("[INFO] [4/5] Generando script NSIS con configuración de PATH y accesos directos...");
            let final_installer_exe = dest_dir.join(format!("{}-setup-v{}.exe", app_name, sanitized_version));
            let nsis_script_content = Self::generate_nsis_script(
                &app_name,
                &sanitized_version,
                &vendor_str,
                icon_path.as_deref(),
                &app_image_folder,
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

            // 7.D: Compilar script con makensis
            println!("[INFO] [5/5] Compilando instalador nativo con makensis...");
            let mut nsis_cmd = Command::new(&makensis_bin);
            nsis_cmd.arg(&nsis_script_path);

            let nsis_res = if verbose {
                nsis_cmd.status().map(|s| s.success()).unwrap_or(false)
            } else {
                nsis_cmd.output().map(|o| o.status.success()).unwrap_or(false)
            };

            // Limpieza
            let _ = fs::remove_dir_all(&package_input_dir);
            let _ = fs::remove_dir_all(&staging_app_dir);

            if !nsis_res {
                return Err("Error al compilar el script NSIS con makensis.".into());
            }

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
            let _ = Self::compress_with_upx(&app_out_dir, custom_upx_args, verbose);
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

    #[test]
    fn test_generate_nsis_script_content() {
        let staging = Path::new("target/staging");
        let output = Path::new("dist/MyApp-setup.exe");
        let script = BuildEngine::generate_nsis_script(
            "MyApp",
            "1.0.0",
            "Acme Corp",
            None,
            staging,
            "MyApp.exe",
            true,
            true,
            true,
            "per-user",
            None,
            output,
        );

        assert!(script.contains("!define PRODUCT_NAME \"MyApp\""));
        assert!(script.contains("!define PRODUCT_VERSION \"1.0.0\""));
        assert!(script.contains("RequestExecutionLevel user"));
        assert!(script.contains("$LOCALAPPDATA\\Programs\\MyApp"));
        assert!(script.contains("MUI_PAGE_WELCOME"));
        assert!(script.contains("StrContains"));
        assert!(script.contains("un.StrReplace"));
        assert!(script.contains("WriteRegExpandStr HKCU \"Environment\" \"PATH\""));
        assert!(script.contains("CreateShortCut \"$DESKTOP\\MyApp.lnk\""));
    }
}


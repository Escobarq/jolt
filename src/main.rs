mod cache;
mod checker;
mod cli;
mod engine;
mod lockfile;
mod manifest;
mod maven;
mod scaffold;
mod toolchain;

use cache::CacheManager;
use clap::Parser;
use lockfile::{JoltLock, LockedPackage};
use maven::MavenClient;
use std::fs;
use std::path::Path;
use toolchain::ToolchainManager;

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    let maven_client = MavenClient::new();
    let cache_manager = CacheManager::new();
    let toolchain_manager = ToolchainManager::new();

    match &cli.command {
        cli::Commands::Init { name, template, list_templates } => {
            if *list_templates {
                scaffold::print_available_templates();
                return;
            }
            if let Err(e) = scaffold::init_project(name.as_deref(), template.as_deref()) {
                eprintln!("[ERROR] Error al inicializar el proyecto: {}", e);
            }
        }
        cli::Commands::Add { dependency, dev } => {
            let manifest_path = Path::new("jolt.toml");

            if !manifest_path.exists() {
                eprintln!("[ERROR] No se encontro 'jolt.toml'. Ejecuta este comando dentro de un proyecto.");
                return;
            }

            let parts: Vec<&str> = dependency.split(':').collect();
            if parts.len() < 2 {
                eprintln!("[ERROR] Formato de dependencia invalido. Usa: 'groupId:artifactId' o 'groupId:artifactId:version'");
                return;
            }

            let group_id = parts[0];
            let artifact_id = parts[1];
            let classifier = if parts.len() >= 4 { Some(parts[3]) } else { None };

            let raw_version = if parts.len() >= 3 {
                parts[2].to_string()
            } else {
                println!("[INFO] Buscando ultima version para '{}:{}' en Maven Central...", group_id, artifact_id);
                match maven_client.fetch_latest_version(group_id, artifact_id).await {
                    Ok(ver) => {
                        println!("[OK] Ultima version encontrada: {}", ver);
                        ver
                    }
                    Err(e) => {
                        eprintln!("[ERROR] {}", e);
                        return;
                    }
                }
            };

            let version_value = if let Some(c) = classifier {
                format!("{}:{}", raw_version, c)
            } else {
                raw_version.clone()
            };

            let dep_key = format!("{}:{}", group_id, artifact_id);
            let target_folder = if *dev { "dev-modules" } else { "modules" };

            match manifest::JoltManifest::add_dependency_to_file(manifest_path, &dep_key, &version_value, *dev) {
                Ok(_) => {
                    let scope_label = if *dev { "dev-dependencies" } else { "dependencies" };
                    println!("[OK] Dependencia '{} = \"{}\"' anadida a jolt.toml [{}]", dep_key, version_value, scope_label);

                    // Descargar a caché global si no existe
                    if !cache_manager.has_jar_with_classifier(group_id, artifact_id, &raw_version, classifier) {
                        let label = match classifier {
                            Some(c) => format!("{}-{}-{}.jar", artifact_id, raw_version, c),
                            None => format!("{}-{}.jar", artifact_id, raw_version),
                        };
                        println!("[INFO] Descargando {} a la cache global...", label);
                        match maven_client.download_jar_with_classifier(group_id, artifact_id, &raw_version, classifier).await {
                            Ok(bytes) => {
                                if let Err(e) = cache_manager.save_jar_with_classifier(group_id, artifact_id, &raw_version, classifier, &bytes) {
                                    eprintln!("[WARN] Error al guardar en cache: {}", e);
                                }
                            }
                            Err(e) => eprintln!("[WARN] Error al descargar binario JAR: {}", e),
                        }
                    } else {
                        println!("[INFO] Usando {}-{} desde la cache global", artifact_id, raw_version);
                    }

                    // Enlazar al proyecto local en la carpeta adecuada
                    if let Ok(linked) = cache_manager.link_to_project_dir_with_classifier(Path::new("."), target_folder, group_id, artifact_id, &raw_version, classifier) {
                        println!("[OK] Enlazado a {}", linked.display());
                    }

                    // Asegurar configuración del editor/IDE
                    let _ = scaffold::ensure_ide_configuration(Path::new("."), None);

                    // Actualizar jolt.lock
                    let cached_jar = cache_manager.get_jar_path_with_classifier(group_id, artifact_id, &raw_version, classifier);
                    let checksum = CacheManager::compute_file_sha256(&cached_jar).unwrap_or_else(|_| "sha256:unknown".to_string());

                    let mut transitive_names = Vec::new();
                    match maven_client.fetch_dependency_tree(group_id, artifact_id, &raw_version).await {
                        Ok(tree) => {
                            if !tree.dependencies.is_empty() {
                                println!("[INFO] Dependencias transitivas detectadas ({}):", tree.dependencies.len());
                                for child in &tree.dependencies {
                                    println!("       └── {}:{} ({})", child.group_id, child.artifact_id, child.version);
                                    transitive_names.push(format!("{}:{}", child.group_id, child.artifact_id));
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[WARN] No se pudieron resolver las dependencias transitivas: {}", e);
                        }
                    }

                    let lock_path = Path::new("jolt.lock");
                    let mut lock = JoltLock::load_from_file(lock_path).unwrap_or_default();
                    lock.add_or_update_package(LockedPackage {
                        name: dep_key.clone(),
                        version: version_value,
                        checksum,
                        dependencies: transitive_names,
                    });
                    if let Ok(_) = lock.save_to_file(lock_path) {
                        println!("[OK] Lockfile 'jolt.lock' actualizado.");
                    }
                }
                Err(e) => eprintln!("[ERROR] Error al actualizar jolt.toml: {}", e),
            }
        }
        cli::Commands::Search { query, limit } => {
            println!("[INFO] Buscando '{}' en Maven Central...", query);
            match maven_client.search_packages(query, *limit).await {
                Ok(results) => {
                    if results.is_empty() {
                        println!("[WARN] No se encontraron paquetes que coincidan con '{}'.", query);
                        return;
                    }

                    println!("\nResultados encontrados ({}):", results.len());
                    println!("============================================================");
                    for item in results {
                        let full_name = format!("{}:{}", item.group_id, item.artifact_id);
                        let pkg_type = item.package_type.as_deref().unwrap_or("jar");
                        println!("• {:<40} (v{}, {})", full_name, item.version, pkg_type);
                        println!("  Comando: jolt add {}:{}", full_name, item.version);
                        println!();
                    }
                    println!("Sugerencia: Copia y ejecuta cualquiera de los comandos anteriores para anadir la libreria.");
                }
                Err(e) => {
                    eprintln!("[ERROR] Error al consultar Maven Central: {}", e);
                }
            }
        }
        cli::Commands::Remove { dependency } => {
            let manifest_path = Path::new("jolt.toml");
            if !manifest_path.exists() {
                eprintln!("[ERROR] No se encontro 'jolt.toml'. Ejecuta este comando dentro de un proyecto.");
                return;
            }

            match manifest::JoltManifest::remove_dependency_from_file(manifest_path, dependency) {
                Ok(removed) => {
                    if removed {
                        println!("[OK] Dependencia '{}' eliminada de jolt.toml", dependency);

                        // Eliminar JAR correspondiente de .jolt/modules/ y .jolt/dev-modules/
                        let parts: Vec<&str> = dependency.split(':').collect();
                        if parts.len() == 2 {
                            let artifact_id = parts[1];
                            let search_folders = [
                                Path::new(".jolt").join("modules"),
                                Path::new(".jolt").join("dev-modules"),
                            ];
                            for modules_dir in &search_folders {
                                if let Ok(entries) = fs::read_dir(modules_dir) {
                                    for entry in entries.flatten() {
                                        let filename = entry.file_name().to_string_lossy().to_string();
                                        if filename.starts_with(artifact_id) && filename.ends_with(".jar") {
                                            let _ = fs::remove_file(entry.path());
                                            println!("[OK] Removido {}", entry.path().display());
                                        }
                                    }
                                }
                            }
                        }

                        // Actualizar jolt.lock y configuración IDE
                        let lock_path = Path::new("jolt.lock");
                        if let Ok(mut lock) = JoltLock::load_from_file(lock_path) {
                            if lock.remove_package(dependency) {
                                let _ = lock.save_to_file(lock_path);
                                println!("[OK] Lockfile 'jolt.lock' sincronizado.");
                            }
                        }
                        let _ = scaffold::ensure_ide_configuration(Path::new("."), None);
                    } else {
                        println!("[WARN] La dependencia '{}' no se encontro en jolt.toml", dependency);
                    }
                }
                Err(e) => eprintln!("[ERROR] Error al modificar jolt.toml: {}", e),
            }
        }
        cli::Commands::Install { locked } => {
            let manifest_path = Path::new("jolt.toml");
            if !manifest_path.exists() {
                eprintln!("[ERROR] No se encontro 'jolt.toml'. Ejecuta este comando dentro de un proyecto.");
                return;
            }

            let _ = scaffold::ensure_ide_configuration(Path::new("."), None);
            let lock_path = Path::new("jolt.lock");


            if *locked {
                if !lock_path.exists() {
                    eprintln!("[ERROR] Modo --locked activo pero no existe 'jolt.lock'.");
                    return;
                }

                let manifest = manifest::JoltManifest::load_from_file(manifest_path).ok();
                let dev_dep_names: std::collections::HashSet<String> = manifest
                    .as_ref()
                    .and_then(|m| m.dev_dependencies.as_ref())
                    .map(|d| d.keys().cloned().collect())
                    .unwrap_or_default();

                match JoltLock::load_from_file(lock_path) {
                    Ok(lock) => {
                        println!("[INFO] Instalando {} dependencias fijadas desde jolt.lock...", lock.packages.len());
                        let mut count = 0;
                        for pkg in lock.packages {
                            let parts: Vec<&str> = pkg.name.split(':').collect();
                            if parts.len() == 2 {
                                let group_id = parts[0];
                                let artifact_id = parts[1];

                                let ver_parts: Vec<&str> = pkg.version.split(':').collect();
                                let version = ver_parts[0];
                                let classifier = if ver_parts.len() > 1 { Some(ver_parts[1]) } else { None };

                                let target_folder = if dev_dep_names.contains(&pkg.name) {
                                    "dev-modules"
                                } else {
                                    "modules"
                                };

                                if !cache_manager.has_jar_with_classifier(group_id, artifact_id, version, classifier) {
                                    println!("[INFO] Descargando fijado {}:{}:{}{:?}...", group_id, artifact_id, version, classifier);
                                    if let Ok(bytes) = maven_client.download_jar_with_classifier(group_id, artifact_id, version, classifier).await {
                                        let _ = cache_manager.save_jar_with_classifier(group_id, artifact_id, version, classifier, &bytes);
                                    }
                                }

                                if let Ok(_) = cache_manager.link_to_project_dir_with_classifier(Path::new("."), target_folder, group_id, artifact_id, version, classifier) {
                                    count += 1;
                                }
                            }
                        }
                        println!("[OK] Instalacion determinista completada: {} dependencias vinculadas.", count);
                    }
                    Err(e) => eprintln!("[ERROR] Error al leer jolt.lock: {}", e),
                }
                return;
            }

            match manifest::JoltManifest::load_from_file(manifest_path) {
                Ok(manifest) => {
                    let mut prod_count = 0;
                    let mut dev_count = 0;
                    let mut lock = JoltLock::load_from_file(lock_path).unwrap_or_default();

                    // 1. Instalar dependencias de producción (modules/)
                    if let Some(deps) = manifest.dependencies {
                        println!("[INFO] Sincronizando {} dependencias de produccion...", deps.len());
                        for (dep_name, version_spec) in deps {
                            let parts: Vec<&str> = dep_name.split(':').collect();
                            if parts.len() == 2 {
                                let group_id = parts[0];
                                let artifact_id = parts[1];

                                let ver_parts: Vec<&str> = version_spec.split(':').collect();
                                let version = ver_parts[0];
                                let classifier = if ver_parts.len() > 1 { Some(ver_parts[1]) } else { None };

                                if !cache_manager.has_jar_with_classifier(group_id, artifact_id, version, classifier) {
                                    println!("[INFO] Descargando {}:{}:{}{:?}...", group_id, artifact_id, version, classifier);
                                    if let Ok(bytes) = maven_client.download_jar_with_classifier(group_id, artifact_id, version, classifier).await {
                                        let _ = cache_manager.save_jar_with_classifier(group_id, artifact_id, version, classifier, &bytes);
                                    }
                                }

                                if let Ok(_) = cache_manager.link_to_project_dir_with_classifier(Path::new("."), "modules", group_id, artifact_id, version, classifier) {
                                    prod_count += 1;

                                    let cached_jar = cache_manager.get_jar_path_with_classifier(group_id, artifact_id, version, classifier);
                                    let checksum = CacheManager::compute_file_sha256(&cached_jar).unwrap_or_else(|_| "sha256:unknown".to_string());

                                    lock.add_or_update_package(LockedPackage {
                                        name: dep_name.clone(),
                                        version: version_spec.clone(),
                                        checksum,
                                        dependencies: vec![],
                                    });
                                }
                            }
                        }
                    }

                    // 2. Instalar dependencias de desarrollo/testing (dev-modules/)
                    if let Some(dev_deps) = manifest.dev_dependencies {
                        println!("[INFO] Sincronizando {} dependencias de desarrollo...", dev_deps.len());
                        for (dep_name, version_spec) in dev_deps {
                            let parts: Vec<&str> = dep_name.split(':').collect();
                            if parts.len() == 2 {
                                let group_id = parts[0];
                                let artifact_id = parts[1];

                                let ver_parts: Vec<&str> = version_spec.split(':').collect();
                                let version = ver_parts[0];
                                let classifier = if ver_parts.len() > 1 { Some(ver_parts[1]) } else { None };

                                if !cache_manager.has_jar_with_classifier(group_id, artifact_id, version, classifier) {
                                    println!("[INFO] Descargando dev {}:{}:{}{:?}...", group_id, artifact_id, version, classifier);
                                    if let Ok(bytes) = maven_client.download_jar_with_classifier(group_id, artifact_id, version, classifier).await {
                                        let _ = cache_manager.save_jar_with_classifier(group_id, artifact_id, version, classifier, &bytes);
                                    }
                                }

                                if let Ok(_) = cache_manager.link_to_project_dir_with_classifier(Path::new("."), "dev-modules", group_id, artifact_id, version, classifier) {
                                    dev_count += 1;

                                    let cached_jar = cache_manager.get_jar_path_with_classifier(group_id, artifact_id, version, classifier);
                                    let checksum = CacheManager::compute_file_sha256(&cached_jar).unwrap_or_else(|_| "sha256:unknown".to_string());

                                    lock.add_or_update_package(LockedPackage {
                                        name: dep_name.clone(),
                                        version: version_spec.clone(),
                                        checksum,
                                        dependencies: vec![],
                                    });
                                }
                            }
                        }
                    }

                    let _ = lock.save_to_file(lock_path);
                    let _ = scaffold::ensure_ide_configuration(Path::new("."), None);
                    println!("[OK] Instalacion completa: {} en .jolt/modules/ y {} en .jolt/dev-modules/ (guardadas en jolt.lock)", prod_count, dev_count);
                }
                Err(e) => eprintln!("[ERROR] Error al leer jolt.toml: {}", e),
            }
        }
        cli::Commands::Build { standalone } => {
            let manifest_path = Path::new("jolt.toml");
            if !manifest_path.exists() {
                eprintln!("[ERROR] No se encontro 'jolt.toml'. Ejecuta este comando dentro de un proyecto.");
                return;
            }

            match manifest::JoltManifest::load_from_file(manifest_path) {
                Ok(manifest) => {
                    let java_ver = manifest.project.java_version.as_deref().unwrap_or("21");
                    let toolchain = match toolchain_manager.get_or_download_toolchain(java_ver).await {
                        Ok(tc) => Some(tc),
                        Err(e) => {
                            eprintln!("[WARN] No se pudo aprovisionar JDK {}: {}. Usando JDK por defecto del sistema.", java_ver, e);
                            None
                        }
                    };

                    if *standalone {
                        println!("[INFO] Empaquetando Fat-JAR autonomo para '{}'...", manifest.project.name);
                        match engine::BuildEngine::build_standalone_jar(
                            Path::new("."),
                            &manifest.project.name,
                            &manifest.project.version,
                            "Main",
                            toolchain.as_ref(),
                        ) {
                            Ok(jar_path) => println!("[OK] Fat-JAR creado exitosamente en: {}", jar_path.display()),
                            Err(e) => eprintln!("[ERROR] Error al crear Fat-JAR: {}", e),
                        }
                    } else {
                        println!("[INFO] Compilando '{}' con Java {}...", manifest.project.name, java_ver);
                        match engine::BuildEngine::build_jar(
                            Path::new("."),
                            &manifest.project.name,
                            &manifest.project.version,
                            "Main",
                            toolchain.as_ref(),
                        ) {
                            Ok(jar_path) => println!("[OK] JAR estandar creado en: {}", jar_path.display()),
                            Err(e) => eprintln!("[ERROR] {}", e),
                        }
                    }
                }
                Err(e) => eprintln!("[ERROR] Error al leer jolt.toml: {}", e),
            }
        }
        cli::Commands::Run { watch } => {
            let manifest_path = Path::new("jolt.toml");
            if !manifest_path.exists() {
                eprintln!("[ERROR] No se encontro 'jolt.toml'. Ejecuta este comando dentro de un proyecto.");
                return;
            }

            match manifest::JoltManifest::load_from_file(manifest_path) {
                Ok(manifest) => {
                    let java_ver = manifest.project.java_version.as_deref().unwrap_or("21");
                    let toolchain = match toolchain_manager.get_or_download_toolchain(java_ver).await {
                        Ok(tc) => Some(tc),
                        Err(e) => {
                            eprintln!("[WARN] No se pudo aprovisionar JDK {}: {}. Usando JDK por defecto del sistema.", java_ver, e);
                            None
                        }
                    };

                    if *watch {
                        if let Err(e) = engine::BuildEngine::run_watch(Path::new("."), "Main", toolchain.as_ref()) {
                            eprintln!("[ERROR] {}", e);
                        }
                    } else {
                        println!("[INFO] Compilando y ejecutando con Java {}...", java_ver);
                        if let Err(e) = engine::BuildEngine::run(Path::new("."), "Main", toolchain.as_ref()) {
                            eprintln!("[ERROR] {}", e);
                        }
                    }
                }
                Err(e) => eprintln!("[ERROR] Error al leer jolt.toml: {}", e),
            }
        }
        cli::Commands::Test => {
            let manifest_path = Path::new("jolt.toml");
            if !manifest_path.exists() {
                eprintln!("[ERROR] No se encontro 'jolt.toml'. Ejecuta este comando dentro de un proyecto.");
                return;
            }

            match manifest::JoltManifest::load_from_file(manifest_path) {
                Ok(manifest) => {
                    let java_ver = manifest.project.java_version.as_deref().unwrap_or("21");
                    let toolchain = match toolchain_manager.get_or_download_toolchain(java_ver).await {
                        Ok(tc) => Some(tc),
                        Err(e) => {
                            eprintln!("[WARN] No se pudo aprovisionar JDK {}: {}. Usando JDK por defecto del sistema.", java_ver, e);
                            None
                        }
                    };

                    const JUNIT_GROUP: &str = "org.junit.platform";
                    const JUNIT_ARTIFACT: &str = "junit-platform-console-standalone";
                    const JUNIT_VERSION: &str = "1.10.2";

                    if !cache_manager.has_jar(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION) {
                        println!("[INFO] Aprovisionando JUnit 5 Platform Console Launcher ({}) a la cache...", JUNIT_VERSION);
                        match maven_client.download_jar_with_classifier(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION, None).await {
                            Ok(bytes) => {
                                let _ = cache_manager.save_jar_with_classifier(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION, None, &bytes);
                            }
                            Err(e) => {
                                eprintln!("[ERROR] No se pudo descargar JUnit runner: {}", e);
                                return;
                            }
                        }
                    }

                    let junit_jar_path = cache_manager.get_jar_path(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION);

                    println!("[INFO] Ejecutando suite de pruebas unitarias (JUnit 5)...");
                    if let Err(e) = engine::BuildEngine::run_tests(Path::new("."), toolchain.as_ref(), &junit_jar_path) {
                        eprintln!("[ERROR] {}", e);
                    }
                }
                Err(e) => eprintln!("[ERROR] Error al leer jolt.toml: {}", e),
            }
        }
        cli::Commands::Sync => {
            let manifest_path = Path::new("jolt.toml");
            if !manifest_path.exists() {
                eprintln!("[ERROR] No se encontro 'jolt.toml'. Ejecuta este comando dentro de un proyecto.");
                return;
            }

            println!("[INFO] Sincronizando dependencias y regenerando entorno IDE...");

            match manifest::JoltManifest::load_from_file(manifest_path) {
                Ok(manifest) => {
                    let lock_path = Path::new("jolt.lock");
                    let mut lock = JoltLock::load_from_file(lock_path).unwrap_or_default();
                    let mut prod_count = 0;
                    let mut dev_count = 0;

                    let mut active_prod_jars = std::collections::HashSet::new();
                    let mut active_dev_jars = std::collections::HashSet::new();

                    // 1. Sincronizar dependencias de producción (modules/)
                    if let Some(deps) = &manifest.dependencies {
                        for (dep_name, version_spec) in deps {
                            let parts: Vec<&str> = dep_name.split(':').collect();
                            if parts.len() == 2 {
                                let group_id = parts[0];
                                let artifact_id = parts[1];

                                let ver_parts: Vec<&str> = version_spec.split(':').collect();
                                let version = ver_parts[0];
                                let classifier = if ver_parts.len() > 1 { Some(ver_parts[1]) } else { None };

                                let file_name = match classifier {
                                    Some(c) if !c.is_empty() => format!("{}-{}-{}.jar", artifact_id, version, c),
                                    _ => format!("{}-{}.jar", artifact_id, version),
                                };
                                active_prod_jars.insert(file_name.clone());

                                if !cache_manager.has_jar_with_classifier(group_id, artifact_id, version, classifier) {
                                    println!("[INFO] Descargando {}:{}:{}{:?}...", group_id, artifact_id, version, classifier);
                                    match maven_client.download_jar_with_classifier(group_id, artifact_id, version, classifier).await {
                                        Ok(bytes) => {
                                            if let Err(e) = cache_manager.save_jar_with_classifier(group_id, artifact_id, version, classifier, &bytes) {
                                                eprintln!("[WARN] Error al guardar en cache: {}", e);
                                            }
                                        }
                                        Err(e) => eprintln!("[WARN] Error al descargar {}: {}", dep_name, e),
                                    }
                                }

                                if let Ok(_) = cache_manager.link_to_project_dir_with_classifier(Path::new("."), "modules", group_id, artifact_id, version, classifier) {
                                    prod_count += 1;

                                    let cached_jar = cache_manager.get_jar_path_with_classifier(group_id, artifact_id, version, classifier);
                                    let checksum = CacheManager::compute_file_sha256(&cached_jar).unwrap_or_else(|_| "sha256:unknown".to_string());

                                    lock.add_or_update_package(LockedPackage {
                                        name: dep_name.clone(),
                                        version: version_spec.clone(),
                                        checksum,
                                        dependencies: vec![],
                                    });
                                }
                            }
                        }
                    }

                    // 2. Sincronizar dependencias de desarrollo (dev-modules/)
                    if let Some(dev_deps) = &manifest.dev_dependencies {
                        for (dep_name, version_spec) in dev_deps {
                            let parts: Vec<&str> = dep_name.split(':').collect();
                            if parts.len() == 2 {
                                let group_id = parts[0];
                                let artifact_id = parts[1];

                                let ver_parts: Vec<&str> = version_spec.split(':').collect();
                                let version = ver_parts[0];
                                let classifier = if ver_parts.len() > 1 { Some(ver_parts[1]) } else { None };

                                let file_name = match classifier {
                                    Some(c) if !c.is_empty() => format!("{}-{}-{}.jar", artifact_id, version, c),
                                    _ => format!("{}-{}.jar", artifact_id, version),
                                };
                                active_dev_jars.insert(file_name.clone());

                                if !cache_manager.has_jar_with_classifier(group_id, artifact_id, version, classifier) {
                                    println!("[INFO] Descargando dev {}:{}:{}{:?}...", group_id, artifact_id, version, classifier);
                                    match maven_client.download_jar_with_classifier(group_id, artifact_id, version, classifier).await {
                                        Ok(bytes) => {
                                            if let Err(e) = cache_manager.save_jar_with_classifier(group_id, artifact_id, version, classifier, &bytes) {
                                                eprintln!("[WARN] Error al guardar en cache: {}", e);
                                            }
                                        }
                                        Err(e) => eprintln!("[WARN] Error al descargar dev {}: {}", dep_name, e),
                                    }
                                }

                                if let Ok(_) = cache_manager.link_to_project_dir_with_classifier(Path::new("."), "dev-modules", group_id, artifact_id, version, classifier) {
                                    dev_count += 1;

                                    let cached_jar = cache_manager.get_jar_path_with_classifier(group_id, artifact_id, version, classifier);
                                    let checksum = CacheManager::compute_file_sha256(&cached_jar).unwrap_or_else(|_| "sha256:unknown".to_string());

                                    lock.add_or_update_package(LockedPackage {
                                        name: dep_name.clone(),
                                        version: version_spec.clone(),
                                        checksum,
                                        dependencies: vec![],
                                    });
                                }
                            }
                        }
                    }

                    // 3. Limpiar JARs huérfanos en .jolt/modules y .jolt/dev-modules
                    let modules_dir = Path::new(".jolt").join("modules");
                    if modules_dir.is_dir() {
                        if let Ok(entries) = fs::read_dir(&modules_dir) {
                            for entry in entries.flatten() {
                                let fname = entry.file_name().to_string_lossy().to_string();
                                if fname.ends_with(".jar") && !active_prod_jars.contains(&fname) {
                                    let _ = fs::remove_file(entry.path());
                                }
                            }
                        }
                    }
                    let dev_modules_dir = Path::new(".jolt").join("dev-modules");
                    if dev_modules_dir.is_dir() {
                        if let Ok(entries) = fs::read_dir(&dev_modules_dir) {
                            for entry in entries.flatten() {
                                let fname = entry.file_name().to_string_lossy().to_string();
                                if fname.ends_with(".jar") && !active_dev_jars.contains(&fname) {
                                    let _ = fs::remove_file(entry.path());
                                }
                            }
                        }
                    }

                    // 4. Limpiar entradas huérfanas en lockfile
                    let mut declared_all = std::collections::HashSet::new();
                    if let Some(deps) = &manifest.dependencies {
                        for k in deps.keys() {
                            declared_all.insert(k.clone());
                        }
                    }
                    if let Some(dev_deps) = &manifest.dev_dependencies {
                        for k in dev_deps.keys() {
                            declared_all.insert(k.clone());
                        }
                    }

                    lock.packages.retain(|pkg| declared_all.contains(&pkg.name));
                    let _ = lock.save_to_file(lock_path);


                    // 5. Generar / actualizar toda la configuración de IDE
                    if let Err(e) = scaffold::ensure_ide_configuration(Path::new("."), Some(&manifest.project.name)) {
                        eprintln!("[WARN] No se pudo generar la configuracion de IDE completa: {}", e);
                    }

                    println!("[OK] Sincronizacion completada para '{}':", manifest.project.name);
                    println!("     • {} dependencias en .jolt/modules/", prod_count);
                    println!("     • {} dependencias en .jolt/dev-modules/", dev_count);
                    println!("     • Lockfile 'jolt.lock' actualizado.");
                    println!("     • Soporte VS Code (.vscode/settings.json, .vscode/extensions.json) y Language Server (.classpath, .project) configurados.");
                }
                Err(e) => eprintln!("[ERROR] Error al leer jolt.toml: {}", e),
            }
        }
        cli::Commands::Check => {
            if let Err(e) = checker::SystemChecker::run_check(Path::new("."), &cache_manager, &toolchain_manager).await {
                eprintln!("[ERROR] Error durante el diagnostico: {}", e);
            }
        }
    }
}


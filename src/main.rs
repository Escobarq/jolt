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
use std::path::{Path, PathBuf};
use toolchain::ToolchainManager;

fn resolve_target_directories(
    member: Option<&str>,
    all: bool,
    default_all: bool,
) -> Result<Vec<PathBuf>, String> {
    let root_manifest_path = Path::new("jolt.toml");
    if !root_manifest_path.exists() {
        return Err("No se encontro 'jolt.toml'. Ejecuta este comando dentro de un proyecto o workspace.".to_string());
    }

    let manifest = manifest::JoltManifest::load_from_file(root_manifest_path)
        .map_err(|e| format!("Error al leer jolt.toml: {}", e))?;

    if manifest.is_workspace() {
        let ws_members = manifest.workspace.unwrap().members;
        if let Some(m) = member {
            if ws_members.contains(&m.to_string()) || Path::new(m).join("jolt.toml").exists() {
                Ok(vec![PathBuf::from(m)])
            } else {
                Err(format!(
                    "El modulo '{}' no esta registrado en [workspace].members ni existe.",
                    m
                ))
            }
        } else if all || default_all {
            if ws_members.is_empty() {
                Err("El workspace no tiene miembros registrados en [workspace].members".to_string())
            } else {
                Ok(ws_members.into_iter().map(PathBuf::from).collect())
            }
        } else {
            if ws_members.len() == 1 {
                Ok(vec![PathBuf::from(&ws_members[0])])
            } else {
                Err("Estas en la raiz de un Workspace Monorepo. Especifica un modulo con '-p <modulo>' o usa '--all'.".to_string())
            }
        }
    } else {
        Ok(vec![PathBuf::from(".")])
    }
}

async fn install_in_dir(
    project_dir: &Path,
    locked: bool,
    cache_manager: &CacheManager,
    maven_client: &MavenClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manifest_path = project_dir.join("jolt.toml");
    if !manifest_path.exists() {
        return Err(format!("No se encontro 'jolt.toml' en {}", project_dir.display()).into());
    }

    let _ = scaffold::ensure_ide_configuration(project_dir, None);
    let lock_path = project_dir.join("jolt.lock");

    if locked {
        if !lock_path.exists() {
            return Err(format!("Modo --locked activo pero no existe 'jolt.lock' en {}", project_dir.display()).into());
        }

        let manifest = manifest::JoltManifest::load_from_file(&manifest_path).ok();
        let dev_dep_names: std::collections::HashSet<String> = manifest
            .as_ref()
            .and_then(|m| m.dev_dependencies.as_ref())
            .map(|d| d.keys().cloned().collect())
            .unwrap_or_default();

        let lock = JoltLock::load_from_file(&lock_path)?;
        println!("[INFO] Instalando {} dependencias fijadas desde jolt.lock en {}...", lock.packages.len(), project_dir.display());
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

                if let Ok(_) = cache_manager.link_to_project_dir_with_classifier(project_dir, target_folder, group_id, artifact_id, version, classifier) {
                    count += 1;
                }
            }
        }
        println!("[OK] Instalacion determinista completada en {}: {} dependencias vinculadas.", project_dir.display(), count);
        return Ok(());
    }

    let manifest = manifest::JoltManifest::load_from_file(&manifest_path)?;
    let mut prod_count = 0;
    let mut dev_count = 0;
    let mut lock = JoltLock::load_from_file(&lock_path).unwrap_or_default();

    // 1. Instalar dependencias de producción (modules/)
    if let Some(deps) = manifest.dependencies {
        println!("[INFO] Sincronizando dependencias de produccion en {}...", project_dir.display());
        for (dep_name, spec) in deps {
            let (version_opt, local_path) = manifest::JoltManifest::parse_dependency_spec(&spec);
            if local_path.is_some() {
                continue; // Dependencia local inter-módulo
            }

            if let Some(version_spec) = version_opt {
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

                    if let Ok(_) = cache_manager.link_to_project_dir_with_classifier(project_dir, "modules", group_id, artifact_id, version, classifier) {
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
    }

    // 2. Instalar dependencias de desarrollo/testing (dev-modules/)
    if let Some(dev_deps) = manifest.dev_dependencies {
        println!("[INFO] Sincronizando dependencias de desarrollo en {}...", project_dir.display());
        for (dep_name, spec) in dev_deps {
            let (version_opt, local_path) = manifest::JoltManifest::parse_dependency_spec(&spec);
            if local_path.is_some() {
                continue;
            }

            if let Some(version_spec) = version_opt {
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

                    if let Ok(_) = cache_manager.link_to_project_dir_with_classifier(project_dir, "dev-modules", group_id, artifact_id, version, classifier) {
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
    }

    let _ = lock.save_to_file(&lock_path);
    let _ = scaffold::ensure_ide_configuration(project_dir, None);
    println!("[OK] Instalacion completa en {}: {} en .jolt/modules/ y {} en .jolt/dev-modules/", project_dir.display(), prod_count, dev_count);

    Ok(())
}

async fn sync_in_dir(
    project_dir: &Path,
    cache_manager: &CacheManager,
    maven_client: &MavenClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manifest_path = project_dir.join("jolt.toml");
    if !manifest_path.exists() {
        return Err(format!("No se encontro 'jolt.toml' en {}", project_dir.display()).into());
    }

    let manifest = manifest::JoltManifest::load_from_file(&manifest_path)?;
    let lock_path = project_dir.join("jolt.lock");
    let mut lock = JoltLock::load_from_file(&lock_path).unwrap_or_default();
    let mut prod_count = 0;
    let mut dev_count = 0;

    let mut active_prod_jars = std::collections::HashSet::new();
    let mut active_dev_jars = std::collections::HashSet::new();

    // 1. Sincronizar dependencias de producción (modules/)
    if let Some(deps) = &manifest.dependencies {
        for (dep_name, spec) in deps {
            let (version_opt, local_path) = manifest::JoltManifest::parse_dependency_spec(spec);
            if local_path.is_some() {
                continue;
            }

            if let Some(version_spec) = version_opt {
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
                        if let Ok(bytes) = maven_client.download_jar_with_classifier(group_id, artifact_id, version, classifier).await {
                            let _ = cache_manager.save_jar_with_classifier(group_id, artifact_id, version, classifier, &bytes);
                        }
                    }

                    if let Ok(_) = cache_manager.link_to_project_dir_with_classifier(project_dir, "modules", group_id, artifact_id, version, classifier) {
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
    }

    // 2. Sincronizar dependencias de desarrollo (dev-modules/)
    if let Some(dev_deps) = &manifest.dev_dependencies {
        for (dep_name, spec) in dev_deps {
            let (version_opt, local_path) = manifest::JoltManifest::parse_dependency_spec(spec);
            if local_path.is_some() {
                continue;
            }

            if let Some(version_spec) = version_opt {
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
                        if let Ok(bytes) = maven_client.download_jar_with_classifier(group_id, artifact_id, version, classifier).await {
                            let _ = cache_manager.save_jar_with_classifier(group_id, artifact_id, version, classifier, &bytes);
                        }
                    }

                    if let Ok(_) = cache_manager.link_to_project_dir_with_classifier(project_dir, "dev-modules", group_id, artifact_id, version, classifier) {
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
    }

    // 3. Limpiar JARs huérfanos
    let modules_dir = project_dir.join(".jolt").join("modules");
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
    let dev_modules_dir = project_dir.join(".jolt").join("dev-modules");
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

    let _ = lock.save_to_file(&lock_path);
    let proj_name = manifest.project.as_ref().map(|p| p.name.as_str());
    let _ = scaffold::ensure_ide_configuration(project_dir, proj_name);

    println!("[OK] Sincronizacion completada para '{}':", proj_name.unwrap_or("app"));
    println!("     • {} dependencias en .jolt/modules/", prod_count);
    println!("     • {} dependencias en .jolt/dev-modules/", dev_count);

    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    let maven_client = MavenClient::new();
    let cache_manager = CacheManager::new();
    let toolchain_manager = ToolchainManager::new();

    match &cli.command {
        cli::Commands::Init {
            name,
            template,
            list_templates,
            package,
            group_id,
            workspace,
        } => {
            if *list_templates {
                scaffold::print_available_templates();
                return;
            }
            if let Err(e) = scaffold::init_project(
                name.as_deref(),
                template.as_deref(),
                package.as_deref(),
                group_id.as_deref(),
                *workspace,
            ) {
                eprintln!("[ERROR] Error al inicializar: {}", e);
            }
        }
        cli::Commands::Add { dependency, dev, member } => {
            let target_dir = if let Some(m) = member {
                PathBuf::from(m)
            } else {
                PathBuf::from(".")
            };

            let manifest_path = target_dir.join("jolt.toml");
            if !manifest_path.exists() {
                eprintln!("[ERROR] No se encontro 'jolt.toml' en {}. Ejecuta este comando dentro de un proyecto o usa --member <nombre>.", target_dir.display());
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

            match manifest::JoltManifest::add_dependency_to_file(&manifest_path, &dep_key, &version_value, *dev) {
                Ok(_) => {
                    let scope_label = if *dev { "dev-dependencies" } else { "dependencies" };
                    println!("[OK] Dependencia '{} = \"{}\"' anadida a {} [{}]", dep_key, version_value, manifest_path.display(), scope_label);

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

                    // Enlazar al proyecto local
                    if let Ok(linked) = cache_manager.link_to_project_dir_with_classifier(&target_dir, target_folder, group_id, artifact_id, &raw_version, classifier) {
                        println!("[OK] Enlazado a {}", linked.display());
                    }

                    let _ = scaffold::ensure_ide_configuration(&target_dir, None);

                    let cached_jar = cache_manager.get_jar_path_with_classifier(group_id, artifact_id, &raw_version, classifier);
                    let checksum = CacheManager::compute_file_sha256(&cached_jar).unwrap_or_else(|_| "sha256:unknown".to_string());

                    let mut transitive_names = Vec::new();
                    if let Ok(tree) = maven_client.fetch_dependency_tree(group_id, artifact_id, &raw_version).await {
                        if !tree.dependencies.is_empty() {
                            println!("[INFO] Dependencias transitivas detectadas ({}):", tree.dependencies.len());
                            for child in &tree.dependencies {
                                println!("       └── {}:{} ({})", child.group_id, child.artifact_id, child.version);
                                transitive_names.push(format!("{}:{}", child.group_id, child.artifact_id));
                            }
                        }
                    }

                    let lock_path = target_dir.join("jolt.lock");
                    let mut lock = JoltLock::load_from_file(&lock_path).unwrap_or_default();
                    lock.add_or_update_package(LockedPackage {
                        name: dep_key.clone(),
                        version: version_value,
                        checksum,
                        dependencies: transitive_names,
                    });
                    let _ = lock.save_to_file(&lock_path);
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
                }
                Err(e) => eprintln!("[ERROR] Error al consultar Maven Central: {}", e),
            }
        }
        cli::Commands::Remove { dependency, member } => {
            let target_dir = if let Some(m) = member {
                PathBuf::from(m)
            } else {
                PathBuf::from(".")
            };

            let manifest_path = target_dir.join("jolt.toml");
            if !manifest_path.exists() {
                eprintln!("[ERROR] No se encontro 'jolt.toml' en {}.", target_dir.display());
                return;
            }

            match manifest::JoltManifest::remove_dependency_from_file(&manifest_path, dependency) {
                Ok(removed) => {
                    if removed {
                        println!("[OK] Dependencia '{}' eliminada de {}", dependency, manifest_path.display());

                        let parts: Vec<&str> = dependency.split(':').collect();
                        if parts.len() == 2 {
                            let artifact_id = parts[1];
                            let search_folders = [
                                target_dir.join(".jolt").join("modules"),
                                target_dir.join(".jolt").join("dev-modules"),
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

                        let lock_path = target_dir.join("jolt.lock");
                        if let Ok(mut lock) = JoltLock::load_from_file(&lock_path) {
                            if lock.remove_package(dependency) {
                                let _ = lock.save_to_file(&lock_path);
                            }
                        }
                        let _ = scaffold::ensure_ide_configuration(&target_dir, None);
                    } else {
                        println!("[WARN] La dependencia '{}' no se encontro en {}", dependency, manifest_path.display());
                    }
                }
                Err(e) => eprintln!("[ERROR] Error al modificar jolt.toml: {}", e),
            }
        }
        cli::Commands::Install { locked, all, member } => {
            let target_dirs = match resolve_target_directories(member.as_deref(), *all, true) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[ERROR] {}", e);
                    return;
                }
            };

            for dir in target_dirs {
                if let Err(e) = install_in_dir(&dir, *locked, &cache_manager, &maven_client).await {
                    eprintln!("[ERROR] Error en {}: {}", dir.display(), e);
                }
            }
        }
        cli::Commands::Build { standalone, package, installer, upx, add_to_path, scope, all, member } => {
            let target_dirs = match resolve_target_directories(member.as_deref(), *all, false) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[ERROR] {}", e);
                    return;
                }
            };

            for dir in target_dirs {
                let manifest_path = dir.join("jolt.toml");
                match manifest::JoltManifest::load_from_file(&manifest_path) {
                    Ok(manifest) => {
                        let proj = match &manifest.project {
                            Some(p) => p,
                            None => continue,
                        };

                        let java_ver = proj.java_version.as_deref().unwrap_or("21");
                        let toolchain = toolchain_manager.get_or_download_toolchain(java_ver).await.ok();

                        let main_class = proj.main_class
                            .as_deref()
                            .or_else(|| manifest.package.as_ref().and_then(|p| p.main_class.as_deref()))
                            .map(|s| s.to_string())
                            .or_else(|| engine::BuildEngine::detect_main_class(&dir))
                            .unwrap_or_else(|| "Main".to_string());

                        if *installer || *package {
                            let pkg_type = if *installer {
                                if cfg!(target_os = "windows") { "msi" } else { "app-image" }
                            } else {
                                "app-image"
                            };
                            let opt_upx = if *upx { Some(true) } else { None };
                            let opt_add_to_path = if *add_to_path { Some(true) } else { None };

                            match engine::BuildEngine::package_native_app(
                                &dir,
                                &manifest,
                                Some(pkg_type),
                                None,
                                None,
                                None,
                                Some(&main_class),
                                None,
                                None,
                                opt_upx,
                                opt_add_to_path,
                                scope.as_deref(),
                                false,
                                toolchain.as_ref(),
                            ) {
                                Ok(output_path) => {
                                    println!("[OK] Paquete / Instalador generado exitosamente en: {}", output_path.display());
                                }
                                Err(e) => eprintln!("[ERROR] Error al empaquetar en {}: {}", dir.display(), e),
                            }
                        } else if *standalone {
                            println!("[INFO] Empaquetando Fat-JAR autonomo para '{}'...", proj.name);
                            match engine::BuildEngine::build_standalone_jar(
                                &dir,
                                &proj.name,
                                &proj.version,
                                &main_class,
                                toolchain.as_ref(),
                            ) {
                                Ok(jar_path) => println!("[OK] Fat-JAR creado exitosamente en: {}", jar_path.display()),
                                Err(e) => eprintln!("[ERROR] Error al crear Fat-JAR en {}: {}", dir.display(), e),
                            }
                        } else {
                            println!("[INFO] Compilando '{}' con Java {}...", proj.name, java_ver);
                            match engine::BuildEngine::build_jar(
                                &dir,
                                &proj.name,
                                &proj.version,
                                &main_class,
                                toolchain.as_ref(),
                            ) {
                                Ok(jar_path) => println!("[OK] JAR estandar creado en: {}", jar_path.display()),
                                Err(e) => eprintln!("[ERROR] Error en {}: {}", dir.display(), e),
                            }
                        }
                    }
                    Err(e) => eprintln!("[ERROR] Error al leer {}: {}", manifest_path.display(), e),
                }
            }
        }
        cli::Commands::Package {
            r#type,
            dest,
            name,
            app_version,
            main_class,
            icon,
            java_options,
            upx,
            no_upx,
            add_to_path,
            scope,
            member,
            verbose,
        } => {
            let target_dirs = match resolve_target_directories(member.as_deref(), false, false) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[ERROR] {}", e);
                    return;
                }
            };

            let dir = &target_dirs[0];
            let manifest_path = dir.join("jolt.toml");
            match manifest::JoltManifest::load_from_file(&manifest_path) {
                Ok(manifest) => {
                    let java_ver = manifest.project.as_ref().and_then(|p| p.java_version.as_deref()).unwrap_or("21");
                    let toolchain = toolchain_manager.get_or_download_toolchain(java_ver).await.ok();

                    let opt_upx = if *no_upx {
                        Some(false)
                    } else if *upx {
                        Some(true)
                    } else {
                        None
                    };
                    let opt_add_to_path = if *add_to_path {
                        Some(true)
                    } else {
                        None
                    };

                    match engine::BuildEngine::package_native_app(
                        dir,
                        &manifest,
                        r#type.as_deref(),
                        dest.as_deref(),
                        name.as_deref(),
                        app_version.as_deref(),
                        main_class.as_deref(),
                        icon.as_deref(),
                        java_options.as_deref(),
                        opt_upx,
                        opt_add_to_path,
                        scope.as_deref(),
                        *verbose,
                        toolchain.as_ref(),
                    ) {
                        Ok(output_path) => {
                            println!("[OK] Paquete / Lanzador binario nativo generado exitosamente en: {}", output_path.display());
                            if output_path.is_file() || output_path.join("bin").exists() {
                                println!("[TIP] Puedes ejecutar la aplicacion directamente con: {}", output_path.display());
                            }
                        }
                        Err(e) => eprintln!("[ERROR] Error al empaquetar: {}", e),
                    }
                }
                Err(e) => eprintln!("[ERROR] Error al leer jolt.toml en {}: {}", dir.display(), e),
            }
        }
        cli::Commands::Run { watch, member } => {
            let target_dirs = match resolve_target_directories(member.as_deref(), false, false) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[ERROR] {}", e);
                    return;
                }
            };

            let dir = &target_dirs[0];
            let manifest_path = dir.join("jolt.toml");
            match manifest::JoltManifest::load_from_file(&manifest_path) {
                Ok(manifest) => {
                    let proj = match &manifest.project {
                        Some(p) => p,
                        None => {
                            eprintln!("[ERROR] El archivo '{}' no define un [project] ejecutable.", manifest_path.display());
                            return;
                        }
                    };

                    let java_ver = proj.java_version.as_deref().unwrap_or("21");
                    let toolchain = toolchain_manager.get_or_download_toolchain(java_ver).await.ok();

                    let main_class = proj.main_class
                        .as_deref()
                        .or_else(|| manifest.package.as_ref().and_then(|p| p.main_class.as_deref()))
                        .map(|s| s.to_string())
                        .or_else(|| engine::BuildEngine::detect_main_class(dir))
                        .unwrap_or_else(|| "Main".to_string());

                    if *watch {
                        if let Err(e) = engine::BuildEngine::run_watch(dir, &main_class, toolchain.as_ref()) {
                            eprintln!("[ERROR] {}", e);
                        }
                    } else {
                        println!("[INFO] Compilando y ejecutando '{}' con Java {}...", proj.name, java_ver);
                        if let Err(e) = engine::BuildEngine::run(dir, &main_class, toolchain.as_ref()) {
                            eprintln!("[ERROR] {}", e);
                        }
                    }
                }
                Err(e) => eprintln!("[ERROR] Error al leer {}: {}", manifest_path.display(), e),
            }
        }
        cli::Commands::Test { all, member } => {
            let target_dirs = match resolve_target_directories(member.as_deref(), *all, true) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[ERROR] {}", e);
                    return;
                }
            };

            const JUNIT_GROUP: &str = "org.junit.platform";
            const JUNIT_ARTIFACT: &str = "junit-platform-console-standalone";
            const JUNIT_VERSION: &str = "1.10.2";

            if !cache_manager.has_jar(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION) {
                println!("[INFO] Aprovisionando JUnit 5 Platform Console Launcher ({}) a la cache...", JUNIT_VERSION);
                if let Ok(bytes) = maven_client.download_jar_with_classifier(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION, None).await {
                    let _ = cache_manager.save_jar_with_classifier(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION, None, &bytes);
                }
            }
            let junit_jar_path = cache_manager.get_jar_path(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION);

            for dir in target_dirs {
                let manifest_path = dir.join("jolt.toml");
                if let Ok(manifest) = manifest::JoltManifest::load_from_file(&manifest_path) {
                    let java_ver = manifest.project.as_ref().and_then(|p| p.java_version.as_deref()).unwrap_or("21");
                    let toolchain = toolchain_manager.get_or_download_toolchain(java_ver).await.ok();
                    let proj_name = manifest.project.as_ref().map(|p| p.name.as_str()).unwrap_or("app");

                    println!("[INFO] Ejecutando suite de pruebas unitarias (JUnit 5) para '{}'...", proj_name);
                    if let Err(e) = engine::BuildEngine::run_tests(&dir, toolchain.as_ref(), &junit_jar_path) {
                        eprintln!("[ERROR] Pruebas fallidas en {}: {}", dir.display(), e);
                    }
                }
            }
        }
        cli::Commands::Sync { all, member } => {
            let target_dirs = match resolve_target_directories(member.as_deref(), *all, true) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[ERROR] {}", e);
                    return;
                }
            };

            for dir in target_dirs {
                if let Err(e) = sync_in_dir(&dir, &cache_manager, &maven_client).await {
                    eprintln!("[ERROR] Error al sincronizar {}: {}", dir.display(), e);
                }
            }
        }
        cli::Commands::Check => {
            if let Err(e) = checker::SystemChecker::run_check(Path::new("."), &cache_manager, &toolchain_manager).await {
                eprintln!("[ERROR] Error durante el diagnostico: {}", e);
            }
        }
    }
}



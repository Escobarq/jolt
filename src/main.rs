mod cache;
mod cli;
mod engine;
mod manifest;
mod maven;
mod scaffold;
mod toolchain;

use cache::CacheManager;
use clap::Parser;
use maven::MavenClient;
use std::path::Path;
use toolchain::ToolchainManager;

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    let maven_client = MavenClient::new();
    let cache_manager = CacheManager::new();
    let toolchain_manager = ToolchainManager::new();

    match &cli.command {
        cli::Commands::Init { name } => {
            if let Err(e) = scaffold::init_project(name.as_deref()) {
                eprintln!("❌ Error al inicializar el proyecto: {}", e);
            }
        }
        cli::Commands::Add { dependency } => {
            let manifest_path = Path::new("jolt.toml");

            if !manifest_path.exists() {
                eprintln!("❌ No se encontró 'jolt.toml'. ¿Estás en un proyecto inicializado con 'jolt init'?");
                return;
            }

            let parts: Vec<&str> = dependency.split(':').collect();
            if parts.len() < 2 {
                eprintln!("❌ Formato de dependencia inválido. Usa: 'groupId:artifactId' o 'groupId:artifactId:version'");
                return;
            }

            let group_id = parts[0];
            let artifact_id = parts[1];

            let version = if parts.len() >= 3 {
                parts[2].to_string()
            } else {
                println!("🔍 Buscando última versión para '{}:{}' en Maven Central...", group_id, artifact_id);
                match maven_client.fetch_latest_version(group_id, artifact_id).await {
                    Ok(ver) => {
                        println!("✨ Última versión encontrada: {}", ver);
                        ver
                    }
                    Err(e) => {
                        eprintln!("❌ {}", e);
                        return;
                    }
                }
            };

            let dep_key = format!("{}:{}", group_id, artifact_id);
            match manifest::JoltManifest::add_dependency_to_file(manifest_path, &dep_key, &version) {
                Ok(_) => {
                    println!("✅ Dependencia '{} = \"{}\"' añadida a jolt.toml", dep_key, version);
                    
                    // Descargar a caché global si no existe
                    if !cache_manager.has_jar(group_id, artifact_id, &version) {
                        println!("📥 Descargando {}-{}.jar a la caché global...", artifact_id, version);
                        match maven_client.download_jar(group_id, artifact_id, &version).await {
                            Ok(bytes) => {
                                if let Err(e) = cache_manager.save_jar(group_id, artifact_id, &version, &bytes) {
                                    eprintln!("⚠️ Error al guardar en caché: {}", e);
                                }
                            }
                            Err(e) => eprintln!("⚠️ Error al descargar binario JAR: {}", e),
                        }
                    } else {
                        println!("⚡ Usando {}-{}.jar desde la caché global", artifact_id, version);
                    }

                    // Enlazar al proyecto local
                    if let Ok(linked) = cache_manager.link_to_project(Path::new("."), group_id, artifact_id, &version) {
                        println!("🔗 Enlazado a {}", linked.display());
                    }

                    // Mostrar árbol de dependencias transitivas
                    match maven_client.fetch_dependency_tree(group_id, artifact_id, &version).await {
                        Ok(tree) => {
                            if !tree.dependencies.is_empty() {
                                println!("📦 Dependencias transitivas detectadas ({}):", tree.dependencies.len());
                                for child in tree.dependencies {
                                    println!("   └── {}:{} ({})", child.group_id, child.artifact_id, child.version);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️  No se pudieron resolver las dependencias transitivas: {}", e);
                        }
                    }
                }
                Err(e) => eprintln!("❌ Error al actualizar jolt.toml: {}", e),
            }
        }
        cli::Commands::Install => {
            let manifest_path = Path::new("jolt.toml");
            if !manifest_path.exists() {
                eprintln!("❌ No se encontró 'jolt.toml'. Ejecuta este comando dentro de un proyecto.");
                return;
            }

            match manifest::JoltManifest::load_from_file(manifest_path) {
                Ok(manifest) => {
                    let mut count = 0;
                    if let Some(deps) = manifest.dependencies {
                        println!("⚡ Sincronizando {} dependencias...", deps.len());
                        for (dep_name, version) in deps {
                            let parts: Vec<&str> = dep_name.split(':').collect();
                            if parts.len() == 2 {
                                let group_id = parts[0];
                                let artifact_id = parts[1];

                                if !cache_manager.has_jar(group_id, artifact_id, &version) {
                                    println!("📥 Descargando {}:{}:{}...", group_id, artifact_id, version);
                                    if let Ok(bytes) = maven_client.download_jar(group_id, artifact_id, &version).await {
                                        let _ = cache_manager.save_jar(group_id, artifact_id, &version, &bytes);
                                    }
                                }

                                if let Ok(_) = cache_manager.link_to_project(Path::new("."), group_id, artifact_id, &version) {
                                    count += 1;
                                }
                            }
                        }
                    }
                    println!("✨ ¡Instalación completa! {} dependencias vinculadas en .jolt/modules/", count);
                }
                Err(e) => eprintln!("❌ Error al leer jolt.toml: {}", e),
            }
        }
        cli::Commands::Build { standalone } => {
            let manifest_path = Path::new("jolt.toml");
            if !manifest_path.exists() {
                eprintln!("❌ No se encontró 'jolt.toml'. Ejecuta este comando dentro de un proyecto.");
                return;
            }

            match manifest::JoltManifest::load_from_file(manifest_path) {
                Ok(manifest) => {
                    let java_ver = manifest.project.java_version.as_deref().unwrap_or("21");
                    let toolchain = match toolchain_manager.get_or_download_toolchain(java_ver).await {
                        Ok(tc) => Some(tc),
                        Err(e) => {
                            eprintln!("⚠️  No se pudo aprovisionar JDK {}: {}. Usando JDK por defecto del sistema.", java_ver, e);
                            None
                        }
                    };

                    if *standalone {
                        println!("📦 Empaquetando Fat-JAR autónomo para '{}'...", manifest.project.name);
                        match engine::BuildEngine::build_standalone_jar(
                            Path::new("."),
                            &manifest.project.name,
                            &manifest.project.version,
                            "Main",
                            toolchain.as_ref(),
                        ) {
                            Ok(jar_path) => println!("✨ ¡Fat-JAR creado exitosamente en: {}!", jar_path.display()),
                            Err(e) => eprintln!("❌ Error al crear Fat-JAR: {}", e),
                        }
                    } else {
                        println!("🔨 Compilando '{}' con Java {}...", manifest.project.name, java_ver);
                        match engine::BuildEngine::build_jar(
                            Path::new("."),
                            &manifest.project.name,
                            &manifest.project.version,
                            "Main",
                            toolchain.as_ref(),
                        ) {
                            Ok(jar_path) => println!("📦 JAR estándar creado en: {}", jar_path.display()),
                            Err(e) => eprintln!("❌ {}", e),
                        }
                    }
                }
                Err(e) => eprintln!("❌ Error al leer jolt.toml: {}", e),
            }
        }
        cli::Commands::Run => {
            let manifest_path = Path::new("jolt.toml");
            if !manifest_path.exists() {
                eprintln!("❌ No se encontró 'jolt.toml'. Ejecuta este comando dentro de un proyecto.");
                return;
            }

            match manifest::JoltManifest::load_from_file(manifest_path) {
                Ok(manifest) => {
                    let java_ver = manifest.project.java_version.as_deref().unwrap_or("21");
                    let toolchain = match toolchain_manager.get_or_download_toolchain(java_ver).await {
                        Ok(tc) => Some(tc),
                        Err(e) => {
                            eprintln!("⚠️  No se pudo aprovisionar JDK {}: {}. Usando JDK por defecto del sistema.", java_ver, e);
                            None
                        }
                    };

                    println!("⚡ Compilando y ejecutando con Java {}...", java_ver);
                    if let Err(e) = engine::BuildEngine::run(Path::new("."), "Main", toolchain.as_ref()) {
                        eprintln!("❌ {}", e);
                    }
                }
                Err(e) => eprintln!("❌ Error al leer jolt.toml: {}", e),
            }
        }
    }
}

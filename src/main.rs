mod cache;
mod checker;
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
            let classifier = if parts.len() >= 4 { Some(parts[3]) } else { None };

            let raw_version = if parts.len() >= 3 {
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

            let version_value = if let Some(c) = classifier {
                format!("{}:{}", raw_version, c)
            } else {
                raw_version.clone()
            };

            let dep_key = format!("{}:{}", group_id, artifact_id);
            match manifest::JoltManifest::add_dependency_to_file(manifest_path, &dep_key, &version_value) {
                Ok(_) => {
                    println!("✅ Dependencia '{} = \"{}\"' añadida a jolt.toml", dep_key, version_value);
                    
                    // Descargar a caché global si no existe
                    if !cache_manager.has_jar_with_classifier(group_id, artifact_id, &raw_version, classifier) {
                        let label = match classifier {
                            Some(c) => format!("{}-{}-{}.jar", artifact_id, raw_version, c),
                            None => format!("{}-{}.jar", artifact_id, raw_version),
                        };
                        println!("📥 Descargando {} a la caché global...", label);
                        match maven_client.download_jar_with_classifier(group_id, artifact_id, &raw_version, classifier).await {
                            Ok(bytes) => {
                                if let Err(e) = cache_manager.save_jar_with_classifier(group_id, artifact_id, &raw_version, classifier, &bytes) {
                                    eprintln!("⚠️ Error al guardar en caché: {}", e);
                                }
                            }
                            Err(e) => eprintln!("⚠️ Error al descargar binario JAR: {}", e),
                        }
                    } else {
                        println!("⚡ Usando {}-{} desde la caché global", artifact_id, raw_version);
                    }

                    // Enlazar al proyecto local
                    if let Ok(linked) = cache_manager.link_to_project_with_classifier(Path::new("."), group_id, artifact_id, &raw_version, classifier) {
                        println!("🔗 Enlazado a {}", linked.display());
                    }

                    // Mostrar árbol de dependencias transitivas
                    match maven_client.fetch_dependency_tree(group_id, artifact_id, &raw_version).await {
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
                        for (dep_name, version_spec) in deps {
                            let parts: Vec<&str> = dep_name.split(':').collect();
                            if parts.len() == 2 {
                                let group_id = parts[0];
                                let artifact_id = parts[1];

                                let ver_parts: Vec<&str> = version_spec.split(':').collect();
                                let version = ver_parts[0];
                                let classifier = if ver_parts.len() > 1 { Some(ver_parts[1]) } else { None };

                                if !cache_manager.has_jar_with_classifier(group_id, artifact_id, version, classifier) {
                                    println!("📥 Descargando {}:{}:{}{:?}...", group_id, artifact_id, version, classifier);
                                    if let Ok(bytes) = maven_client.download_jar_with_classifier(group_id, artifact_id, version, classifier).await {
                                        let _ = cache_manager.save_jar_with_classifier(group_id, artifact_id, version, classifier, &bytes);
                                    }
                                }

                                if let Ok(_) = cache_manager.link_to_project_with_classifier(Path::new("."), group_id, artifact_id, version, classifier) {
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
        cli::Commands::Run { watch } => {
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

                    if *watch {
                        if let Err(e) = engine::BuildEngine::run_watch(Path::new("."), "Main", toolchain.as_ref()) {
                            eprintln!("❌ {}", e);
                        }
                    } else {
                        println!("⚡ Compilando y ejecutando con Java {}...", java_ver);
                        if let Err(e) = engine::BuildEngine::run(Path::new("."), "Main", toolchain.as_ref()) {
                            eprintln!("❌ {}", e);
                        }
                    }
                }
                Err(e) => eprintln!("❌ Error al leer jolt.toml: {}", e),
            }
        }
        cli::Commands::Test => {
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

                    const JUNIT_GROUP: &str = "org.junit.platform";
                    const JUNIT_ARTIFACT: &str = "junit-platform-console-standalone";
                    const JUNIT_VERSION: &str = "1.10.2";

                    if !cache_manager.has_jar(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION) {
                        println!("📥 Aprovisionando JUnit 5 Platform Console Launcher ({}) a la caché...", JUNIT_VERSION);
                        match maven_client.download_jar_with_classifier(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION, None).await {
                            Ok(bytes) => {
                                let _ = cache_manager.save_jar_with_classifier(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION, None, &bytes);
                            }
                            Err(e) => {
                                eprintln!("❌ No se pudo descargar JUnit runner: {}", e);
                                return;
                            }
                        }
                    }

                    let junit_jar_path = cache_manager.get_jar_path(JUNIT_GROUP, JUNIT_ARTIFACT, JUNIT_VERSION);

                    println!("🧪 Ejecutando suite de pruebas unitarias (JUnit 5)...");
                    if let Err(e) = engine::BuildEngine::run_tests(Path::new("."), toolchain.as_ref(), &junit_jar_path) {
                        eprintln!("❌ {}", e);
                    }
                }
                Err(e) => eprintln!("❌ Error al leer jolt.toml: {}", e),
            }
        }
        cli::Commands::Check => {
            if let Err(e) = checker::SystemChecker::run_check(Path::new("."), &cache_manager, &toolchain_manager).await {
                eprintln!("❌ Error durante el diagnóstico: {}", e);
            }
        }
    }
}

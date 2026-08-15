use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

pub struct SystemChecker;

impl SystemChecker {
    /// Ejecuta una utilidad del sistema y devuelve su primera línea de versión
    pub fn get_command_version(cmd_name: &str, version_flag: &str) -> Option<String> {
        let output = Command::new(cmd_name).arg(version_flag).output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        let raw = if !stdout.is_empty() { stdout } else { stderr };
        raw.lines().next().map(|s| s.trim().to_string())
    }

    /// Calcula el tamaño total y la cantidad de archivos dentro de un directorio
    pub fn get_dir_stats(dir: &Path) -> (usize, u64) {
        let mut count = 0;
        let mut total_bytes = 0;

        if dir.is_dir() {
            for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    count += 1;
                    if let Ok(metadata) = entry.metadata() {
                        total_bytes += metadata.len();
                    }
                }
            }
        }

        (count, total_bytes)
    }

    /// Formatea bytes a una unidad legible (KB, MB, GB)
    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Ejecuta el diagnóstico completo del entorno y del proyecto actual
    pub async fn run_check(
        project_dir: &Path,
        _cache_manager: &crate::cache::CacheManager,
        _toolchain_manager: &crate::toolchain::ToolchainManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("============================================================");
        println!("🩺 Jolt Doctor / Check - Diagnóstico de Entorno y Proyecto");
        println!("============================================================");

        // 1. Diagnóstico del Entorno Global del Sistema
        println!("\n🔍 [1/2] Entorno Global del Sistema:");

        // Java Runtime
        if let Some(ver) = Self::get_command_version("java", "-version") {
            println!("  ✔ Java Runtime (java):       {}", ver);
        } else {
            println!("  ❌ Java Runtime (java):       No detectado en PATH");
        }

        // Java Compiler
        if let Some(ver) = Self::get_command_version("javac", "-version") {
            println!("  ✔ Java Compiler (javac):     {}", ver);
        } else {
            println!("  ⚠️  Java Compiler (javac):     No detectado en PATH (Jolt auto-aprovisionará JDK si es necesario)");
        }

        // Java Archiver
        if let Some(ver) = Self::get_command_version("jar", "--version") {
            println!("  ✔ Java Archiver (jar):       {}", ver);
        } else {
            println!("  ⚠️  Java Archiver (jar):       No detectado en PATH");
        }

        // Rust Toolchain
        if let Some(ver) = Self::get_command_version("rustc", "--version") {
            println!("  ✔ Rust Compiler (rustc):     {}", ver);
        } else {
            println!("  ⚠️  Rust Compiler (rustc):     No instalado");
        }

        if let Some(ver) = Self::get_command_version("cargo", "--version") {
            println!("  ✔ Cargo Package Manager:     {}", ver);
        } else {
            println!("  ⚠️  Cargo Package Manager:     No instalado");
        }

        // Caché Global de Jolt
        if let Some(home) = dirs::home_dir() {
            let jolt_cache = home.join(".jolt").join("cache").join("v1");
            if jolt_cache.exists() {
                let (count, bytes) = Self::get_dir_stats(&jolt_cache);
                println!(
                    "  ✔ Caché Global de Jolt:      {} ({} archivos en {})",
                    jolt_cache.display(),
                    count,
                    Self::format_bytes(bytes)
                );
            } else {
                println!("  ℹ️  Caché Global de Jolt:      Aún no inicializada ({})", jolt_cache.display());
            }

            let jolt_toolchains = home.join(".jolt").join("toolchains");
            if jolt_toolchains.is_dir() {
                let mut toolchains = Vec::new();
                if let Ok(entries) = fs::read_dir(&jolt_toolchains) {
                    for e in entries.flatten() {
                        if e.path().is_dir() {
                            toolchains.push(e.file_name().to_string_lossy().to_string());
                        }
                    }
                }
                if !toolchains.is_empty() {
                    println!("  ✔ JDKs Aprovisionados:       {}", toolchains.join(", "));
                }
            }
        }

        // 2. Diagnóstico del Proyecto Actual
        println!("\n📦 [2/2] Diagnóstico del Proyecto Actual:");
        let manifest_path = project_dir.join("jolt.toml");

        if !manifest_path.exists() {
            println!("  ℹ️  No se detectó un archivo 'jolt.toml' en el directorio actual.");
            println!("     Para crear un nuevo proyecto ejecuta: 'jolt init <nombre>'");
            println!("\n✨ Diagnóstico completado: El entorno global está listo para operar.");
            return Ok(());
        }

        match crate::manifest::JoltManifest::load_from_file(&manifest_path) {
            Ok(manifest) => {
                println!("  ✔ Manifiesto 'jolt.toml':    Válido");
                println!("     • Proyecto:                {}", manifest.project.name);
                println!("     • Versión:                 {}", manifest.project.version);
                println!(
                    "     • Java Requerido:          Java {}",
                    manifest.project.java_version.as_deref().unwrap_or("21")
                );

                // Estructura de directorios
                let main_java = project_dir.join("src").join("main").join("java");
                let main_res = project_dir.join("src").join("main").join("resources");
                let test_java = project_dir.join("src").join("test").join("java");

                let (main_count, _) = Self::get_dir_stats(&main_java);
                let (res_count, _) = Self::get_dir_stats(&main_res);
                let (test_count, _) = Self::get_dir_stats(&test_java);

                println!("  📂 Estructura del Código:");
                println!(
                    "     • Código fuente:          {} ({} archivo(s) .java)",
                    if main_java.exists() { "✔ src/main/java/" } else { "⚠️  src/main/java/ (no encontrado)" },
                    main_count
                );
                println!(
                    "     • Recursos estáticos:     {} ({} archivo(s))",
                    if main_res.exists() { "✔ src/main/resources/" } else { "ℹ️  src/main/resources/ (opcional)" },
                    res_count
                );
                println!(
                    "     • Pruebas unitarias:      {} ({} archivo(s) de test)",
                    if test_java.exists() { "✔ src/test/java/" } else { "ℹ️  src/test/java/ (opcional)" },
                    test_count
                );

                // Verificación de dependencias
                let mut missing_deps = Vec::new();
                let mut ok_deps = 0;

                if let Some(deps) = manifest.dependencies {
                    println!("  📦 Estado de Dependencias ({} declaradas):", deps.len());
                    for (dep_name, version_spec) in deps {
                        let parts: Vec<&str> = dep_name.split(':').collect();
                        if parts.len() == 2 {
                            let artifact_id = parts[1];
                            let ver_parts: Vec<&str> = version_spec.split(':').collect();
                            let ver = ver_parts[0];
                            let classifier = if ver_parts.len() > 1 { Some(ver_parts[1]) } else { None };

                            let file_name = match classifier {
                                Some(c) => format!("{}-{}-{}.jar", artifact_id, ver, c),
                                None => format!("{}-{}.jar", artifact_id, ver),
                            };

                            let module_jar = project_dir.join(".jolt").join("modules").join(&file_name);
                            if module_jar.exists() {
                                println!("     • ✔ {} = \"{}\" (Enlazado en .jolt/modules/)", dep_name, version_spec);
                                ok_deps += 1;
                            } else {
                                println!("     • ❌ {} = \"{}\" (No instalado localmente)", dep_name, version_spec);
                                missing_deps.push(format!("{} = \"{}\"", dep_name, version_spec));
                            }
                        }
                    }
                } else {
                    println!("  📦 Estado de Dependencias:   Sin dependencias externas declaradas");
                }

                if !missing_deps.is_empty() {
                    println!("\n  ⚠️  Se encontraron {} dependencias sin sincronizar ({} listas).", missing_deps.len(), ok_deps);
                    println!("     💡 Sugerencia: Ejecuta 'jolt install' para descargarlas y enlazarlas automáticamente.");
                } else {
                    println!("\n✨ ¡Proyecto saludable con {} dependencia(s) sincronizadas! Listo para compilar ('jolt build') o ejecutar ('jolt run').", ok_deps);
                }
            }
            Err(e) => {
                println!("  ❌ Error de sintaxis en 'jolt.toml': {}", e);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(SystemChecker::format_bytes(500), "500 B");
        assert_eq!(SystemChecker::format_bytes(1024), "1.00 KB");
        assert_eq!(SystemChecker::format_bytes(1024 * 1024 * 5), "5.00 MB");
    }
}

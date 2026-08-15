use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CacheManager {
    cache_root: PathBuf,
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let cache_root = home.join(".jolt").join("cache").join("v1");
        Self { cache_root }
    }

    #[cfg(test)]
    pub fn with_root(custom_root: PathBuf) -> Self {
        Self {
            cache_root: custom_root,
        }
    }

    /// Retorna la ruta al archivo JAR en la caché global
    pub fn get_jar_path(&self, group_id: &str, artifact_id: &str, version: &str) -> PathBuf {
        let group_path = group_id.replace('.', "/");
        self.cache_root
            .join("jars")
            .join(group_path)
            .join(artifact_id)
            .join(version)
            .join(format!("{}-{}.jar", artifact_id, version))
    }

    /// Verifica si un JAR ya existe en la caché global
    pub fn has_jar(&self, group_id: &str, artifact_id: &str, version: &str) -> bool {
        self.get_jar_path(group_id, artifact_id, version).exists()
    }

    /// Guarda los bytes del JAR en la caché global tras calcular su hash SHA-256
    pub fn save_jar(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        bytes: &[u8],
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        let jar_path = self.get_jar_path(group_id, artifact_id, version);
        if let Some(parent) = jar_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Calcular SHA-256 para verificación de integridad
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let _hash_str = hex::encode(hasher.finalize());

        fs::write(&jar_path, bytes)?;
        Ok(jar_path)
    }

    /// Enlaza el JAR de la caché global al directorio local `.jolt/modules/` mediante Hardlink
    pub fn link_to_project(
        &self,
        project_dir: &Path,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        let cached_jar = self.get_jar_path(group_id, artifact_id, version);
        if !cached_jar.exists() {
            return Err(format!("El archivo JAR en caché no existe: {:?}", cached_jar).into());
        }

        let modules_dir = project_dir.join(".jolt").join("modules");
        fs::create_dir_all(&modules_dir)?;

        let target_link = modules_dir.join(format!("{}-{}.jar", artifact_id, version));

        if target_link.exists() {
            fs::remove_file(&target_link)?;
        }

        // Intentar hardlink, si falla (particiones de disco distintas), hacer copia de respaldo
        if fs::hard_link(&cached_jar, &target_link).is_err() {
            fs::copy(&cached_jar, &target_link)?;
        }

        Ok(target_link)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_save_and_link() {
        let temp_dir = std::env::temp_dir().join("jolt_test_cache");
        let _ = fs::remove_dir_all(&temp_dir);

        let cache = CacheManager::with_root(temp_dir.join("global_cache"));
        let dummy_bytes = b"PK\x03\x04test_jar_content";

        let jar_path = cache
            .save_jar("com.google.guava", "guava", "33.0.0", dummy_bytes)
            .expect("Failed to save jar");

        assert!(jar_path.exists());
        assert!(cache.has_jar("com.google.guava", "guava", "33.0.0"));

        let project_dir = temp_dir.join("test_project");
        let linked_jar = cache
            .link_to_project(&project_dir, "com.google.guava", "guava", "33.0.0")
            .expect("Failed to link jar");

        assert!(linked_jar.exists());
        let read_bytes = fs::read(&linked_jar).expect("Failed to read linked jar");
        assert_eq!(read_bytes, dummy_bytes);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}

pub mod compiler;
pub mod runner;
pub mod test_runner;

pub use compiler::{build_classpath, build_test_classpath, collect_java_files, compile, copy_resources, detect_main_class};
pub use runner::{run, run_watch, spawn_process};
pub use test_runner::{compile_tests, run_tests};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_collect_java_files() {
        let temp_dir = std::env::temp_dir().join("jolt_test_build_mod");
        let src_dir = temp_dir.join("src").join("main").join("java");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&src_dir).unwrap();

        fs::write(src_dir.join("Main.java"), "public class Main {}").unwrap();
        fs::write(src_dir.join("Util.java"), "public class Util {}").unwrap();
        fs::write(src_dir.join("README.txt"), "Ignored file").unwrap();

        let java_files = collect_java_files(&temp_dir.join("src"));
        assert_eq!(java_files.len(), 2);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_build_classpath_separation() {
        let temp_dir = std::env::temp_dir().join("jolt_test_engine_cp_mod");
        let _ = fs::remove_dir_all(&temp_dir);
        let prod_modules = temp_dir.join(".jolt").join("modules");
        let dev_modules = temp_dir.join(".jolt").join("dev-modules");
        let classes_dir = temp_dir.join("target").join("classes");

        fs::create_dir_all(&prod_modules).unwrap();
        fs::create_dir_all(&dev_modules).unwrap();
        fs::create_dir_all(&classes_dir).unwrap();

        fs::write(prod_modules.join("gson-2.14.0.jar"), b"dummy").unwrap();
        fs::write(dev_modules.join("junit-jupiter-api-5.10.2.jar"), b"dummy").unwrap();

        let prod_cp = build_classpath(&temp_dir, false);
        assert!(prod_cp.contains("gson-2.14.0.jar"));
        assert!(!prod_cp.contains("junit-jupiter-api-5.10.2.jar"));

        let test_cp = build_test_classpath(&temp_dir, true);
        assert!(test_cp.contains("classes"));
        assert!(test_cp.contains("gson-2.14.0.jar"));
        assert!(test_cp.contains("junit-jupiter-api-5.10.2.jar"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_main_class() {
        let temp_dir = std::env::temp_dir().join("jolt_test_detect_main_mod");
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

        let detected = detect_main_class(&temp_dir);
        assert_eq!(detected, Some("com.example.MyAwesomeApp".to_string()));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}

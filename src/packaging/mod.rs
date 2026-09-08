pub mod jar;
pub mod jpackage;
pub mod native_image;
pub mod nsis;
pub mod upx;

pub use jar::{build_jar, build_standalone_jar};
pub use jpackage::package_native_app;
pub use native_image::build_native_image;
pub use nsis::{find_makensis_binary, generate_nsis_script};
pub use upx::{compress_with_upx, find_upx_binary};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_generate_nsis_script_content() {
        let staging = Path::new("target/staging");
        let output = Path::new("dist/MyApp-setup.exe");
        let script = generate_nsis_script(
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

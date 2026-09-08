use std::path::{Path, PathBuf};
use std::process::Command;

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

; ==============================================================================
; Jolt Windows Installer (NSIS Script)
; Gestor de paquetes y herramientas para Java ultrarrápido desarrollado en Rust
; ==============================================================================

Unicode True
SetCompressor /SOLID lzma

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "WinMessages.nsh"

; ------------------------------------------------------------------------------
; Constantes y Metadatos de la Aplicación
; ------------------------------------------------------------------------------
!define PRODUCT_NAME "Jolt"
!define PRODUCT_DESCRIPTION "Gestor de paquetes, herramientas y proyectos para Java"
!define PRODUCT_PUBLISHER "Jolt Project"
!define PRODUCT_WEB_SITE "https://github.com/Escobarq/jolt"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKCU"

!ifndef PRODUCT_VERSION
  !define PRODUCT_VERSION "0.7.1"
!endif

!ifndef BINARY_PATH
  !define BINARY_PATH "..\..\target\release\jolt.exe"
!endif

!ifndef OUTPUT_DIR
  !define OUTPUT_DIR "..\..\dist"
!endif

!ifndef OUTFILE_NAME
  !define OUTFILE_NAME "jolt-v${PRODUCT_VERSION}-windows-x86_64-setup.exe"
!endif

Name "${PRODUCT_NAME} v${PRODUCT_VERSION}"
OutFile "${OUTPUT_DIR}\${OUTFILE_NAME}"
InstallDir "$LOCALAPPDATA\Programs\Jolt"
InstallDirRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString"
RequestExecutionLevel user

; ------------------------------------------------------------------------------
; Configuración de Interfaz Moderna (MUI2)
; ------------------------------------------------------------------------------
!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TITLE "Bienvenido a la instalación de Jolt"
!define MUI_WELCOMEPAGE_TEXT "Jolt es un gestor de paquetes y herramientas para Java ultrarrápido desarrollado en Rust.$\r$\n$\r$\nEste asistente instalará Jolt en tu sistema y lo configurará automáticamente en tu variable PATH para su uso inmediato en la línea de comandos."

; Páginas del Instalador
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

!define MUI_FINISHPAGE_TITLE "Instalación de Jolt completada"
!define MUI_FINISHPAGE_TEXT "Jolt se ha instalado correctamente y se ha agregado a tu variable de entorno PATH.$\r$\n$\r$\nPuedes abrir una nueva ventana de PowerShell o Command Prompt y ejecutar 'jolt --help'."
!insertmacro MUI_PAGE_FINISH

; Páginas del Desinstalador
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Idiomas
!insertmacro MUI_LANGUAGE "Spanish"
!insertmacro MUI_LANGUAGE "English"

; ------------------------------------------------------------------------------
; Funciones auxiliares para PATH y Cadenas
; ------------------------------------------------------------------------------
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

; ------------------------------------------------------------------------------
; Sección de Instalación Principal
; ------------------------------------------------------------------------------
Section "Jolt CLI (requerido)" SecCore
  SectionIn RO
  SetOutPath "$INSTDIR"
  
  ; Copiar binario principal
  File "/oname=jolt.exe" "${BINARY_PATH}"
  
  ; Crear desinstalador
  WriteUninstaller "$INSTDIR\uninstall.exe"
  
  ; Crear accesos en Menú Inicio
  CreateDirectory "$SMPROGRAMS\Jolt"
  CreateShortCut "$SMPROGRAMS\Jolt\Desinstalar Jolt.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0
  
  ; Registrar en Aplicaciones Instaladas de Windows (Add/Remove Programs)
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_NAME} (Java Build Tool)"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$\"$INSTDIR\jolt.exe$\""
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoModify" 1
  WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoRepair" 1

  ; Agregar a la variable PATH del usuario
  DetailPrint "Configurando variable de entorno PATH..."
  ReadRegStr $0 HKCU "Environment" "PATH"
  ${If} $0 == ""
    WriteRegExpandStr HKCU "Environment" "PATH" "$INSTDIR"
  ${Else}
    Push "$0"
    Push "$INSTDIR"
    Call StrContains
    Pop $1
    ${If} $1 == "0"
      WriteRegExpandStr HKCU "Environment" "PATH" "$0;$INSTDIR"
    ${EndIf}
  ${EndIf}
  
  ; Notificar a Windows del cambio en variables de entorno (WM_SETTINGCHANGE)
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
SectionEnd

; ------------------------------------------------------------------------------
; Sección del Desinstalador
; ------------------------------------------------------------------------------
Section "Uninstall"
  ; Remover del PATH del usuario
  DetailPrint "Removiendo Jolt de la variable PATH..."
  ReadRegStr $0 HKCU "Environment" "PATH"
  
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

  WriteRegExpandStr HKCU "Environment" "PATH" "$0"
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000

  ; Eliminar accesos directos
  Delete "$SMPROGRAMS\Jolt\Desinstalar Jolt.lnk"
  RMDir "$SMPROGRAMS\Jolt"
  
  ; Eliminar archivos y directorio de instalación
  Delete "$INSTDIR\jolt.exe"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  ; Eliminar claves del registro
  DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"
SectionEnd

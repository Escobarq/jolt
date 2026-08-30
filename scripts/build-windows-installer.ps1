# ==============================================================================
# Script de Compilación y Generación del Instalador de Jolt para Windows
# ==============================================================================
param (
    [switch]$NoUpx,
    [switch]$VerboseOutput
)

$ErrorActionPreference = "Stop"

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "  Jolt CLI - Generador de Instalador Oficial para Windows       " -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan

# 1. Detectar herramientas
Write-Host "[1/4] Comprobando herramientas necesarias..." -ForegroundColor Yellow

$MakensisPath = $null
if (Get-Command makensis -ErrorAction SilentlyContinue) {
    $MakensisPath = "makensis"
} else {
    $CommonPaths = @(
        "C:\Program Files (x86)\NSIS\makensis.exe",
        "C:\Program Files\NSIS\makensis.exe",
        "$env:USERPROFILE\scoop\apps\nsis\current\makensis.exe",
        "$env:USERPROFILE\scoop\shims\makensis.exe"
    )
    foreach ($p in $CommonPaths) {
        if (Test-Path $p) {
            $MakensisPath = $p
            break
        }
    }
}

if (-not $MakensisPath) {
    Write-Error "makensis (NSIS) no fue encontrado en PATH ni en las rutas estándar. Por favor instala NSIS (ej: scoop install nsis o https://nsis.sourceforge.io/)."
}
Write-Host "  [OK] makensis detectado: $MakensisPath" -ForegroundColor Green

$UpxPath = $null
if (-not $NoUpx) {
    if (Get-Command upx -ErrorAction SilentlyContinue) {
        $UpxPath = "upx"
    } else {
        $ScoopUpx = "$env:USERPROFILE\scoop\shims\upx.exe"
        if (Test-Path $ScoopUpx) {
            $UpxPath = $ScoopUpx
        }
    }
    if ($UpxPath) {
        Write-Host "  [OK] upx detectado: $UpxPath" -ForegroundColor Green
    } else {
        Write-Host "  [WARN] upx no encontrado. Se continuará sin compresión UPX." -ForegroundColor DarkYellow
    }
}

# 2. Compilar binario de Jolt en modo Release
Write-Host "`n[2/4] Compilando Jolt CLI en modo release con Cargo..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Fallo al compilar Jolt con cargo build --release"
}

$JoltBin = "target\release\jolt.exe"
if (-not (Test-Path $JoltBin)) {
    Write-Error "No se encontró el binario compilado en $JoltBin"
}

$OrigSize = (Get-Item $JoltBin).Length
Write-Host "  [OK] Binario generado: $JoltBin ($([math]::Round($OrigSize / 1MB, 2)) MB)" -ForegroundColor Green

# 3. Optimización con UPX
if ($UpxPath -and (-not $NoUpx)) {
    Write-Host "`n[3/4] Comprimiendo $JoltBin con UPX (--best --lzma)..." -ForegroundColor Yellow
    & $UpxPath --best --lzma $JoltBin
    $CompSize = (Get-Item $JoltBin).Length
    $SavingsPct = [math]::Round((1.0 - ($CompSize / $OrigSize)) * 100, 1)
    Write-Host "  [OK] Tamaño optimizado: $([math]::Round($OrigSize / 1MB, 2)) MB -> $([math]::Round($CompSize / 1MB, 2)) MB (Ahorro: -$SavingsPct%)" -ForegroundColor Green
} else {
    Write-Host "`n[3/4] Omitiendo compresión UPX." -ForegroundColor DarkGray
}

# 4. Generar instalador con NSIS
Write-Host "`n[4/4] Compilando instalador con NSIS (makensis)..." -ForegroundColor Yellow
$DistDir = "dist"
if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
}

$NsisScript = "installer\windows\jolt_installer.nsi"
& $MakensisPath $NsisScript
if ($LASTEXITCODE -ne 0) {
    Write-Error "makensis falló al generar el instalador."
}

$Installer = Get-ChildItem -Path $DistDir -Filter "jolt-*-windows-x86_64-setup.exe" | Sort-Object LastWriteTime -Descending | Select-Object -First 1

if ($Installer) {
    $InstSize = [math]::Round($Installer.Length / 1MB, 2)
    Write-Host "`n=================================================================" -ForegroundColor Green
    Write-Host "  [EXITO] Instalador creado con éxito!                          " -ForegroundColor Green
    Write-Host "  Ubicación: $($Installer.FullName) ($InstSize MB)              " -ForegroundColor Green
    Write-Host "=================================================================" -ForegroundColor Green
} else {
    Write-Host "`n[OK] Instalador compilado en carpeta $DistDir" -ForegroundColor Green
}

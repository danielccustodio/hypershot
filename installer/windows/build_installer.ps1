# PowerShell script to build the Windows Installer (.exe) and Portable (.zip) for HyperShot
param(
    [switch]$SkipBuild = $false
)

$ErrorActionPreference = "Stop"
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $projectRoot

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Building HyperShot Windows Installer and Portable Packages" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

# 1. Terminate any running hypershot instance so binary is not locked
Stop-Process -Name "hypershot" -Force -ErrorAction SilentlyContinue

# 2. Build Release Binary
if (-not $SkipBuild) {
    Write-Host "`n[1/4] Compiling optimized release binary (cargo build --release)..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Cargo build failed!"
    }
}

$releaseExe = Join-Path $projectRoot "target\release\hypershot.exe"
if (-not (Test-Path $releaseExe)) {
    Write-Error "Release binary not found at $releaseExe!"
}

# 3. Create dist directory
$distDir = Join-Path $projectRoot "dist"
if (-not (Test-Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir -Force | Out-Null
}

# 4. Locate Inno Setup Compiler (ISCC.exe)
Write-Host "`n[2/4] Locating Inno Setup Compiler (ISCC.exe)..." -ForegroundColor Yellow
$possiblePaths = @(
    "C:\Users\$env:USERNAME\AppData\Local\Programs\Inno Setup 6\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe"
)

$isccPath = $null
foreach ($path in $possiblePaths) {
    if (Test-Path $path) {
        $isccPath = $path
        break
    }
}

if (-not $isccPath) {
    $found = Get-Command "ISCC.exe" -ErrorAction SilentlyContinue
    if ($found) {
        $isccPath = $found.Source
    }
}

if (-not $isccPath) {
    Write-Error "Inno Setup Compiler (ISCC.exe) was not found. Please install Inno Setup 6."
}

Write-Host "Found ISCC at: $isccPath" -ForegroundColor Green

# 5. Compile Installer
Write-Host "`n[3/4] Compiling Windows Setup Installer with Inno Setup..." -ForegroundColor Yellow
$issScript = Join-Path $projectRoot "installer\windows\hypershot_setup.iss"
& "$isccPath" /Q "$issScript"
if ($LASTEXITCODE -ne 0) {
    Write-Error "Inno Setup compilation failed!"
}

# 6. Create Portable ZIP
Write-Host "`n[4/4] Generating Portable ZIP package..." -ForegroundColor Yellow
$zipPath = Join-Path $distDir "HyperShot-v0.1.0-Windows-x64-Portable.zip"
if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}

$tempPortableDir = Join-Path $distDir "HyperShot-Portable"
if (Test-Path $tempPortableDir) {
    Remove-Item $tempPortableDir -Recurse -Force
}
New-Item -ItemType Directory -Path $tempPortableDir -Force | Out-Null

Copy-Item $releaseExe -Destination $tempPortableDir
Copy-Item (Join-Path $projectRoot "assets\icon.ico") -Destination $tempPortableDir
Copy-Item (Join-Path $projectRoot "LICENSE-MIT") -Destination (Join-Path $tempPortableDir "LICENSE.txt")
Copy-Item (Join-Path $projectRoot "README.md") -Destination $tempPortableDir

Compress-Archive -Path "$tempPortableDir\*" -DestinationPath $zipPath
Remove-Item $tempPortableDir -Recurse -Force

Write-Host "`n============================================================" -ForegroundColor Green
Write-Host "  HyperShot Windows Distribution Packages Ready!" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
Get-ChildItem -Path $distDir | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize

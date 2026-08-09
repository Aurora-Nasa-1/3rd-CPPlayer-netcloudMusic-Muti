<#
.SYNOPSIS
    Build CPPlayer Rust JNI backend for Desktop (Windows) and package it as a module zip.

.DESCRIPTION
    PowerShell 版本的 build_module_desktop.sh，用于在 Windows 上构建 cp_api 的 JNI 动态库，
    并打包为 CPPlayer Desktop 可导入的模块 zip（包含 manifest.json）。

.PARAMETER Target
    Rust 编译目标三元组，默认 x86_64-pc-windows-msvc。
    如果你在 MSYS2/MinGW 环境下工作，可传入 x86_64-pc-windows-gnu。

.EXAMPLE
    .\build_module_desktop.ps1
    .\build_module_desktop.ps1 -Target x86_64-pc-windows-gnu
#>

param(
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"

Write-Host "Building CPPlayer Rust JNI backend for Desktop (Windows)..."

# 切换到脚本所在目录
# 注意：Set-Location 只会改变 PowerShell 自身的当前目录($PWD)，
# 不会同步 .NET 的 Environment.CurrentDirectory，导致 [System.IO.File] 等
# .NET API 使用相对路径时可能仍解析到进程启动时的旧目录（例如 system32）。
# 因此这里显式同步一下，确保后续所有相对路径解析一致。
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir
[System.IO.Directory]::SetCurrentDirectory($ScriptDir)

$OS = "windows"
$ARCH = "x86_64"

Write-Host "Target: $Target"

# Build
Write-Host "Running cargo build for $Target..."
cargo build --release --features jni --target $Target
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
}

# 自动检测产物 dll 文件名（Cargo 会把包名的 '-' 转成 '_' 作为库名，
# 不同版本/包名可能不一致，因此不再硬编码为 cp_api.dll）
$ReleaseDir = "target\$Target\release"
$DllCandidate = Get-ChildItem -Path $ReleaseDir -Filter "*.dll" -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -notlike "*deps*" } |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
if (-not $DllCandidate) {
    throw "未在 $ReleaseDir 中找到任何 .dll 产物，请检查 cargo build 是否成功生成 cdylib"
}
$LibName = $DllCandidate.Name
Write-Host "Library: $LibName"

# Prepare package directory
$ModuleDir = ".\target\cp_module"
if (Test-Path $ModuleDir) {
    Remove-Item -Recurse -Force $ModuleDir
}
New-Item -ItemType Directory -Path $ModuleDir | Out-Null

# Copy library
$LibSrc = $DllCandidate.FullName
Copy-Item $LibSrc -Destination $ModuleDir

# Detect platform ABI for manifest
$PlatformAbi = "x86_64"

# Generate manifest
$ManifestPath = Join-Path $ModuleDir "manifest.json"
$Manifest = @{
    id            = "cp_api"
    name          = "NeteaseCloudMusicApi-RS"
    version       = "1.0.0"
    type          = "jni"
    entryPoint    = $LibName
    supportedAbis = @($PlatformAbi)
    apiMap        = @{}
}
# 不能直接用 Set-Content -Encoding utf8，Windows PowerShell 5.1 会写入 BOM，
# 导致 kotlinx.serialization 解析 manifest.json 时报 "unexpected json token at offset 0"
$ManifestJson = $Manifest | ConvertTo-Json -Depth 5
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($ManifestPath, $ManifestJson, $Utf8NoBom)

# Create zip
$ZipName = "cp_api_desktop_${OS}_${ARCH}.zip"
$ZipPath = Join-Path (Resolve-Path "target") $ZipName
if (Test-Path $ZipPath) {
    Remove-Item -Force $ZipPath
}
Compress-Archive -Path (Join-Path $ModuleDir "*") -DestinationPath $ZipPath

Write-Host ""
Write-Host "Done! Package: target\$ZipName"
Write-Host "Import this zip into CPPlayer Desktop to use the JNI backend."

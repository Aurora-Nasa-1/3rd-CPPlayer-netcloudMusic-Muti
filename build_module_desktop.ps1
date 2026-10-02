<#
.SYNOPSIS
    在 Windows 上构建 ncm-api-rs 的 JNI 动态库，并打包成 CPPlayer 模块 zip。

.DESCRIPTION
    与 build_module.sh 的 windows 分支等价，只是用 PowerShell 跑 cargo ——
    Git Bash 的 /usr/bin/link.exe 会顶掉 MSVC 的 link.exe，在 bash 里构建会
    报 "link: extra operand"，所以 Windows 本地构建走这个脚本。

.PARAMETER Target
    Rust 目标三元组，默认 x86_64-pc-windows-msvc。MinGW 环境可传 x86_64-pc-windows-gnu。

.PARAMETER Legacy
    额外导出历史宿主包名（cp.player.core / cp.player.kmp / cp.player）的 JNI 符号，
    让同一份 dll 也能被旧版 CPPlayer 加载。

.EXAMPLE
    .\build_module_desktop.ps1
    .\build_module_desktop.ps1 -Target x86_64-pc-windows-gnu
    .\build_module_desktop.ps1 -Legacy
#>

param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$Legacy
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir
# Set-Location 不会同步 .NET 的当前目录，显式同步一次，保证相对路径解析一致
[System.IO.Directory]::SetCurrentDirectory($ScriptDir)

$Features = if ($Legacy) { "jni,legacy-jni-symbols" } else { "jni" }

Write-Host "Building ncm-api-rs JNI module for Windows ($Target), features=$Features ..."
cargo build --release --features $Features --target $Target
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
}

$Dll = "target/$Target/release/ncm_api_rs.dll"
if (-not (Test-Path $Dll)) {
    throw "未找到产物 $Dll，请检查 cargo build 是否生成了 cdylib"
}

# 打包逻辑与 Linux/Android 共用 scripts/package_module.py（manifest 不带 BOM）
python scripts/package_module.py --platform windows --lib "x86_64=$Dll"
if ($LASTEXITCODE -ne 0) {
    throw "打包失败，exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "Done! 把 target/ncm-api-rs-windows.zip 导入 CPPlayer 即可。"

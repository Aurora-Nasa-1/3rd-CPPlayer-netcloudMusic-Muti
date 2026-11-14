<#
.SYNOPSIS
    在 Windows 上构建 ncm-api-rs 的 JNI 动态库，并打包成 CPPlayer 模块 zip。

.DESCRIPTION
    与 build_module.sh 的 windows 分支等价，只是用 PowerShell 跑 cargo ——
    Git Bash 的 /usr/bin/link.exe 会顶掉 MSVC 的 link.exe，在 bash 里构建会
    报 "link: extra operand"，所以 Windows 本地构建走这个脚本。

    脚本会自动定位并导入 MSVC 环境（vswhere 查找 VS，回退本机非标准路径
    E:\project\build，见仓库根 rust-msvc-env.sh），不需要事先手动跑
    vcvarsall 或 source rust-msvc-env.sh。

    注意：本文件必须以「UTF-8 with BOM」保存。PowerShell 5.1 对无 BOM 的
    .ps1 按系统 ANSI（GBK）解码，中文字符串会吞掉右引号并报
    "字符串缺少终止符"。若用编辑器改动本文件，请保持 BOM。

.PARAMETER Target
    Rust 目标三元组，默认 x86_64-pc-windows-msvc。MinGW 环境可传 x86_64-pc-windows-gnu。

.PARAMETER Legacy
    额外导出旧宿主（1.8.7 及更早）使用的 JniProvider 入口与历史包名符号。
    仅用于显式兼容旧客户端；市场发布包应保持默认关闭。

.PARAMETER Offline
    给 cargo 传 --offline，只用本地 crate 缓存构建，不访问网络。

.EXAMPLE
    .\build_module_desktop.ps1
    .\build_module_desktop.ps1 -Offline
    .\build_module_desktop.ps1 -Target x86_64-pc-windows-gnu
    .\build_module_desktop.ps1 -Legacy
#>

param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$Legacy,
    [switch]$Offline
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir
# Set-Location 不会同步 .NET 的当前目录，显式同步一次，保证相对路径解析一致
[System.IO.Directory]::SetCurrentDirectory($ScriptDir)

function Import-MsvcEnvironment {
    # 已有可用 cl.exe（开发者命令行、或 bash 里 source 过 rust-msvc-env.sh 后
    # 在同一环境启动的 PowerShell）则直接复用
    if (Get-Command cl.exe -ErrorAction SilentlyContinue) {
        Write-Host "使用现有 MSVC 环境: $((Get-Command cl.exe).Source)"
    } else {
        $vcvarsCandidates = @()
        $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
        if (Test-Path $vswhere) {
            $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
            if ($vsPath) {
                $vcvarsCandidates += (Join-Path ($vsPath | Select-Object -First 1) "VC\Auxiliary\Build\vcvarsall.bat")
            }
        }
        # 本机 VS 装在非标准路径（MSVC 14.51，见仓库根 rust-msvc-env.sh）
        $vcvarsCandidates += "E:\project\build\VC\Auxiliary\Build\vcvarsall.bat"

        $vcvars = $vcvarsCandidates | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
        if (-not $vcvars) {
            throw "找不到 vcvarsall.bat，无法初始化 MSVC 环境。请在「x64 Native Tools 命令提示符」里运行本脚本，或安装 VS C++ 工具链。"
        }

        Write-Host "导入 MSVC 环境: $vcvars"
        # 跑一遍 vcvarsall x64，把它的环境变量快照导入当前进程
        $envDump = cmd /c "call `"$vcvars`" x64 >nul 2>&1 && set"
        foreach ($line in $envDump) {
            if ($line -match '^([^=]+)=(.*)$') {
                [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], "Process")
            }
        }
        if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
            throw "vcvarsall 导入后仍找不到 cl.exe，请检查 $vcvars 是否包含 x64 工具链。"
        }
        Write-Host "MSVC 环境就绪: $((Get-Command cl.exe).Source)"
    }

    # vcvarsall 依赖 reg.exe 探测 Windows SDK；reg.exe 被安全策略拦截时 LIB 会缺
    # SDK 库目录，编译能过但链接报 LNK1181（找不到 bcrypt.lib 等）。逐项校验必需
    # 目录，缺失则手动补全（布局与仓库根 rust-msvc-env.sh 一致）。
    $clPath = (Get-Command cl.exe).Source
    $msvcRoot = Split-Path (Split-Path (Split-Path (Split-Path $clPath)))
    $vcLib = Join-Path $msvcRoot "lib\x64"
    $vcInc = Join-Path $msvcRoot "include"
    $sdkRoot = "C:\Program Files (x86)\Windows Kits\10"
    $sdkVer = "10.0.26100.0"
    $sdkLibs = @(
        (Join-Path $sdkRoot "Lib\$sdkVer\um\x64"),
        (Join-Path $sdkRoot "Lib\$sdkVer\ucrt\x64")
    )
    $sdkIncs = @(
        (Join-Path $sdkRoot "Include\$sdkVer\ucrt"),
        (Join-Path $sdkRoot "Include\$sdkVer\shared"),
        (Join-Path $sdkRoot "Include\$sdkVer\um"),
        (Join-Path $sdkRoot "Include\$sdkVer\winrt"),
        (Join-Path $sdkRoot "Include\$sdkVer\cppwinrt")
    )

    $curLib = @()
    if ($env:LIB) { $curLib = @($env:LIB -split ';' | Where-Object { $_ }) }
    $missingLib = (@($vcLib) + $sdkLibs) | Where-Object { $curLib -notcontains $_ }
    if ($missingLib) {
        if (-not (Test-Path (Join-Path $sdkLibs[0] "bcrypt.lib"))) {
            throw "Windows SDK 库缺失: $(Join-Path $sdkLibs[0] 'bcrypt.lib') 不存在，请安装 Windows SDK $sdkVer"
        }
        Write-Host ("补全 LIB/INCLUDE（缺: " + ($missingLib -join '; ') + "）")
        $env:LIB = (@($missingLib) + $curLib) -join ';'
        $curInc = @()
        if ($env:INCLUDE) { $curInc = @($env:INCLUDE -split ';' | Where-Object { $_ }) }
        $missingInc = (@($vcInc) + $sdkIncs) | Where-Object { $curInc -notcontains $_ }
        if ($missingInc) {
            $env:INCLUDE = (@($missingInc) + $curInc) -join ';'
        }
    }
}

# GNU 目标走 MinGW 工具链，不需要 MSVC 环境
if ($Target -like "*-msvc") {
    Import-MsvcEnvironment
}

$Features = if ($Legacy) { "jni,legacy-jni-symbols" } else { "jni" }

$cargoArgs = @("build", "--release", "--features", $Features, "--target", $Target)
if ($Offline) { $cargoArgs += "--offline" }

Write-Host "Building ncm-api-rs JNI module for Windows ($Target), features=$Features ..."
& cargo @cargoArgs
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
}

$Dll = "target/$Target/release/ncm_api_rs.dll"
if (-not (Test-Path $Dll)) {
    throw "未找到产物 $Dll，请检查 cargo build 是否生成了 cdylib"
}

# 打包逻辑优先调用 scripts/package_module.py，若本地无可用 Python 则使用 PowerShell 原生打包
$hasPython = (Get-Command python -ErrorAction SilentlyContinue) -and (& python -c "print('ok')" 2>$null) -eq "ok"
if ($hasPython) {
    python scripts/package_module.py --platform windows --lib "x86_64=$Dll"
} else {
    $staging = "target/cp_module/windows"
    if (Test-Path $staging) { Remove-Item $staging -Recurse -Force }
    $libDir = "$staging/lib/x86_64"
    New-Item -ItemType Directory -Path $libDir -Force | Out-Null
    Copy-Item $Dll "$libDir/ncm_api_rs.dll" -Force
    Copy-Item $Dll "$staging/ncm_api_rs.dll" -Force
    # 版本以 Cargo.toml 为唯一真相源，避免回退分支与真实包版本漂移。
    $crateVersion = (Select-String -Path "Cargo.toml" -Pattern '^\s*version\s*=\s*"([^"]+)"' |
        Select-Object -First 1).Matches.Groups[1].Value
    if (-not $crateVersion) { throw "无法从 Cargo.toml 读取 version" }
    $manifest = @{
        id = "netease-cloudmusic-rs"
        name = "NeteaseCloudMusicApi-RS"
        version = $crateVersion
        # 与 scripts/package_module.py 一致：旧宿主（<= 1.8.7）不认 native，
        # 会在类型阶段明确拒绝，而不是把 SDK 包当旧 JNI Provider 静默加载。
        type = "native"
        contract = "kmp-sdk-v1"
        entryPoint = "ncm_api_rs.dll"
        supportedAbis = @("x86_64")
        sdkOrigins = @("https://music.163.com", "https://interface.music.163.com", "https://interface3.music.163.com", "https://interfacepc.music.163.com")
        capabilities = @("search", "playback", "qrLogin", "wordSyncedLyrics", "comments", "cloudDrive", "recommend", "lyricsSearch", "charts", "userPlaylists", "artistExplorer", "albumExplorer")
        apiMap = @{ "msg/private/mark/read" = "unsupported" }
    } | ConvertTo-Json -Depth 5
    [System.IO.File]::WriteAllText("$staging/manifest.json", $manifest, (New-Object System.Text.UTF8Encoding($false)))
    $outZip = "target/ncm-api-rs-windows.zip"
    if (Test-Path $outZip) { Remove-Item $outZip -Force }
    Compress-Archive -Path "$staging/*" -DestinationPath $outZip -Force
}

Write-Host ""
Write-Host "Done! 把 target/ncm-api-rs-windows.zip 导入 CPPlayer 即可。"

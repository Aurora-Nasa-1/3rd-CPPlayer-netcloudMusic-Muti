## 第三方NCM API提供者
### 原链接 [NCM API RUST](https://github.com/SPlayer-Dev/ncm-api-rs)
为CPPlayer修改的JNI自定义后端，本项目仅参考学习。

---

## 适配的宿主包名

JNI 的导出符号与宿主类的**全限定名**硬绑定：虚拟机按 `Java_<包名下划线化>_<类名>_<方法名>`
查找符号。本模块当前导出：

```
Java_cp_cpplayer_core_provider_JniProvider_startNativeServer
Java_cp_cpplayer_core_provider_JniProvider_nativeCallApi
Java_cp_cpplayer_core_provider_JniProvider_analyzeAudioFile
```

对应宿主类 `cp.cpplayer.core.provider.JniProvider`。

> ⚠️ 用旧前缀编译的模块在新宿主上 **System.load() 仍能成功**，但首次方法调用会抛
> `UnsatisfiedLinkError`（症状：「模块显示已加载，一调用就崩」）。宿主改包名后必须用
> 新前缀重新编译，见 `src/util/jni.rs` 末尾的 `export_jni!` 调用。
>
> 历史沿革：`cp.player.provider` → `cp.player.kmp.provider` → `cp.player.core.provider`
> → `cp.cpplayer.core.provider`（当前）。
>
> 需要一份二进制同时兼容旧宿主时，加 `--features legacy-jni-symbols`
> （或 `./build_module.sh --legacy`）把历史前缀一起导出。

## 构建

| 平台 | 命令 | 产物 |
|------|------|------|
| Android（3 ABI） | `./build_module.sh android` | `target/ncm-api-rs-android.zip` |
| Linux x86_64 | `./build_module.sh linux` | `target/ncm-api-rs-linux.zip` |
| Windows x86_64 | `.\build_module_desktop.ps1`（或 `./build_module.sh windows`） | `target/ncm-api-rs-windows.zip` |
| macOS | `./build_module.sh macos` | `target/ncm-api-rs-macos.zip` |

- Android 需要 `cargo install cargo-ndk` 与 NDK（`ANDROID_NDK_HOME`）。
- Windows 建议用 PowerShell 脚本：Git Bash 的 `/usr/bin/link.exe` 会顶掉 MSVC 的
  `link.exe`，在 bash 里构建会报 `link: extra operand`。
- 三端打包统一由 `scripts/package_module.py` 完成（零依赖，标准库），shell 只负责调 cargo。

也可直接调用打包脚本（例如只重新打包不重新编译）：

```bash
python scripts/package_module.py --platform linux \
  --lib x86_64=target/x86_64-unknown-linux-gnu/release/libncm_api_rs.so
```

## 产物结构

```
ncm-api-rs-android.zip
├── manifest.json
└── lib/
    ├── arm64-v8a/libncm_api_rs.so
    ├── armeabi-v7a/libncm_api_rs.so
    └── x86_64/libncm_api_rs.so
```

宿主 `PlatformSupport.resolveEntryPoint` 先按平台 ABI 查 `lib/<abi>/<entryPoint>`
（Android 取 `Build.SUPPORTED_ABIS`，桌面端为 `x86_64`/`amd64`/`x86`），
找不到才回退根目录，因此库必须放在 `lib/<abi>/` 下。

`manifest.json` 的 `version` 直接取自 `Cargo.toml`，不会与代码版本漂移。

## CI

`.github/workflows/build.yml` 在每次 push / PR 构建三端：

- **Android**：`arm64-v8a` / `armeabi-v7a` / `x86_64`（一个 zip 装三个 ABI）
- **Linux**：`x86_64-unknown-linux-gnu`
- **Windows**：`x86_64-pc-windows-msvc`

产物作为 workflow artifact 上传；打 tag 时自动创建 Release 并附上三个 zip。

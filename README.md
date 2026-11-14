## 第三方NCM API提供者
### 原链接 [NCM API RUST](https://github.com/SPlayer-Dev/ncm-api-rs)
为CPPlayer修改的JNI自定义后端，本项目仅参考学习。

---

## 适配的宿主包名与 SDK 接口

### 1. 现代化 KMP SDK (`kmp-sdk-v1` 标准 Native Session)
本模块纯 Rust 完整实现了 CPPlayer `kmp-sdk-v1` 契约，支持全部 161 项业务操作（包括曲库检索、解析、歌词、批量可用性、歌单、专辑、歌手、探索推荐、用户与账号会话、动态与社交、评论、一起听、MV与视频、DJ电台、云盘上传等）。
SDK 的 `AuthActionRequest.countryCode` 是手机号登录和 `captcha.send` 的 typed 可选字段，接受 1–4 位数字；省略时沿用上游默认 `86`。当前 Native v1 positional wire 为兼容旧 Provider，Host 会在 auth-only `extraJson` 槽位传递该字段；Provider 在 wire 解码边界桥接为 typed 字段，业务逻辑不解析 `extraJson`。

JNI 符号（导出至 `src/sdk/jni_session.rs`）：
```
Java_cp_player_core_provider_netease_NeteaseJniSession_createNative
Java_cp_player_core_provider_netease_NeteaseJniSession_invokeNative
Java_cp_player_core_provider_netease_NeteaseJniSession_cancelNative
Java_cp_player_core_provider_netease_NeteaseJniSession_closeNative
```
同时导出 `cp.player.sdk.netease.NeteaseJniSession` 对应符号供通用 SDK 接入。

### 1.1 C ABI（GraalVM Native Image 宿主）

Windows 的 GraalVM Native Image 构建**不能**走 JNI：JNI 绑定挂在类对象上，宿主为了「每包独立绑定」
要按包 `defineClass` 一份 `cp.player.sdk.jni.NativeSession`，而 native image 既不提供类文件资源
（`getResource("...class")` 恒为 `null`），也不支持运行时定义类。

所以本库**同时导出**一套 C ABI（`src/sdk/cabi.rs`），宿主用 FFM 按符号直调，一个包一个库句柄即可：

```
cpp_abi_version   -> 1                      # 宿主先探测这个，没有就回落到 JNI（JVM）
cpp_create        (providerId, accountId, credential?, settingsJson, err, errCap) -> int64 句柄（0 = 失败，err 里是原因）
cpp_invoke        (handle, operation, req, reqLen, out, outCap, outLen*) -> 0 成功 / 1 容量不足 / 2 ABI 层失败
cpp_cancel        (handle)                  # 线程安全、立即返回，不释放句柄
cpp_close         (handle)
```

规范见宿主仓库 `docs/dev/PROVIDER_NATIVE_ABI.md`。要点：

- 请求/响应都是 **Wire 字节**，与 JNI 版完全同一份编解码（`src/sdk/jni_session.rs` 的共用实现）；
- **内存全部由宿主分配**，本库不 `free` 宿主的指针、也不返回需要宿主 `free` 的指针（Windows 下 CRT 堆不同）；
- `outCap` 不足时返回 `1`，并把所需长度写进 `outLen`，宿主扩容后重试（幂等）。

两套符号留在同一个 DLL 里，JVM / Android 继续走 JNI。确认导出：

```powershell
dumpbin /EXPORTS target/x86_64-pc-windows-msvc/release/ncm_api_rs.dll | Select-String "cpp_|NativeSession_"
```

### 2. 传统 Legacy JNI 接口（兼容旧版）
```
Java_cp_player_core_provider_JniProvider_startNativeServer
Java_cp_player_core_provider_JniProvider_nativeCallApi
Java_cp_player_core_provider_JniProvider_analyzeAudioFile
```

对应宿主类 `cp.player.core.provider.JniProvider`。

> ⚠️ 用旧前缀编译的模块在新宿主上 **System.load() 仍能成功**，但首次方法调用会抛
> `UnsatisfiedLinkError`（症状：「模块显示已加载，一调用就崩」）。宿主改包名后必须用
> 新前缀重新编译，见 `src/util/jni.rs` 末尾的 `export_jni!` 调用。
>
> 改包名前先到宿主仓库核实 `JniProvider.kt` 的真实 `package` 声明，不要凭仓库名猜测。
>
> 历史沿革：`cp.player.provider` → `cp.player.kmp.provider` → `cp.player.core.provider`（当前）。
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

## 发版（全自动版本号）

tag 是版本的唯一真相源，**打 tag 即发版，不需要手动改 Cargo.toml**：

```
git tag v0.3.0 && git push origin v0.3.0
```

CI 会自动：把 Cargo.toml 同步成 tag 版本（`scripts/set_cargo_version.py`，
在构建前做，二进制内嵌的 `CARGO_PKG_VERSION` 也跟随）→ 三端构建 →
manifest.json 写入 tag 版本 → 产物命名 `ncm-api-rs-<平台>-<版本>.zip` →
创建 GitHub Release 并上传三个 zip。

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

`manifest.json` 的 `apiMap` 是**能力声明**（唯一声明处：`scripts/package_module.py` 的
`API_MAP`，三端共用）。宿主 `ProviderManager` 会把标了 `"unsupported"` 的方法**直接短路**成
`{"code":-1}`、**不发请求**。当前声明 `msg/private/mark/read` 为 `unsupported` ——
上游没有这个端点（2026-10-05 实测 404），不声明的话宿主每开一次私信会话都会拿到一个 500。

## CI

`.github/workflows/build.yml` 在每次 push / PR 构建三端：

- **Android**：`arm64-v8a` / `armeabi-v7a` / `x86_64`（一个 zip 装三个 ABI）
- **Linux**：`x86_64-unknown-linux-gnu`
- **Windows**：`x86_64-pc-windows-msvc`

产物作为 workflow artifact 上传；打 tag 时自动创建 Release 并附上三个 zip。

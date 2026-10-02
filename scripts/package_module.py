#!/usr/bin/env python3
"""把编译好的 JNI 动态库打包成 CPPlayer 可导入的模块 zip。

打包布局（与宿主 `PlatformSupport.resolveEntryPoint` 的查找顺序一致）：

    module.zip
    ├── manifest.json
    └── lib/<abi>/<entryPoint>

宿主会依次尝试 `lib/<平台ABI>/<entryPoint>`（Android 的 ABI 取自
`Build.SUPPORTED_ABIS`，桌面端固定为 x86_64 / amd64 / x86），找不到才回退到
`<entryPoint>`。所以这里必须放进 `lib/<abi>/` 子目录，不能平铺在根目录。

用法：

    # Android：一个 zip 带 3 个 ABI
    python scripts/package_module.py --platform android \
        --lib arm64-v8a=target/jniLibs/arm64-v8a/libncm_api_rs.so \
        --lib armeabi-v7a=target/jniLibs/armeabi-v7a/libncm_api_rs.so \
        --lib x86_64=target/jniLibs/x86_64/libncm_api_rs.so

    # 桌面：单 ABI
    python scripts/package_module.py --platform linux \
        --lib x86_64=target/x86_64-unknown-linux-gnu/release/libncm_api_rs.so

零依赖，只用标准库；UTF-8 无 BOM 写 manifest（BOM 会让
kotlinx.serialization 在 offset 0 报 unexpected json token）。
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import zipfile

# 与 Cargo.toml 的 [lib] 产物名保持一致：包名 ncm-api-rs -> 库名 ncm_api_rs
DEFAULT_ID = "netease-cloudmusic-rs"
DEFAULT_NAME = "NeteaseCloudMusicApi-RS"

# 各平台动态库的预期后缀，用来在打包前拦下「拿错平台的产物」
EXPECTED_SUFFIX = {
    "android": ".so",
    "linux": ".so",
    "windows": ".dll",
    "macos": ".dylib",
}


def die(msg: str) -> "None":
    print(f"Error: {msg}", file=sys.stderr)
    raise SystemExit(1)


def read_crate_version(root: str) -> str:
    """从 Cargo.toml 读取 version，避免 manifest 与代码版本漂移。"""
    cargo_toml = os.path.join(root, "Cargo.toml")
    try:
        with open(cargo_toml, encoding="utf-8") as f:
            for line in f:
                m = re.match(r'\s*version\s*=\s*"([^"]+)"', line)
                if m:
                    return m.group(1)
    except OSError:
        pass
    return "0.0.0"


def parse_lib_args(pairs: "list[str]") -> "list[tuple[str, str]]":
    """解析 `--lib <abi>=<path>`（可重复）。"""
    out = []
    for item in pairs:
        if "=" not in item:
            die(f"--lib 需要写成 <abi>=<path>，收到: {item!r}")
        abi, path = item.split("=", 1)
        abi = abi.strip()
        if not abi:
            die(f"--lib 的 abi 为空: {item!r}")
        out.append((abi, path))
    if not out:
        die("至少需要一个 --lib <abi>=<path>")
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description="打包 CPPlayer JNI 模块 zip")
    parser.add_argument(
        "--platform",
        required=True,
        choices=sorted(EXPECTED_SUFFIX),
        help="目标平台，决定产物后缀与 zip 命名",
    )
    parser.add_argument(
        "--lib",
        action="append",
        default=[],
        metavar="ABI=PATH",
        help="待打包的动态库，可重复（Android 一次传 3 个 ABI）",
    )
    parser.add_argument("--module-id", default=DEFAULT_ID, help="manifest 的 id")
    parser.add_argument("--name", default=DEFAULT_NAME, help="manifest 的 name")
    parser.add_argument("--version", default=None, help="manifest 版本，默认读 Cargo.toml")
    parser.add_argument(
        "--expected-version",
        default=None,
        help="校验 Cargo.toml 的 version 与之一致（tag 构建时防版本漂移），不一致直接失败",
    )
    parser.add_argument(
        "--out",
        default=None,
        help="输出 zip 路径，默认 target/ncm-api-rs-<platform>.zip",
    )
    parser.add_argument(
        "--staging-dir",
        default=None,
        help="中间目录，默认 target/cp_module/<platform>（打包后保留便于检查）",
    )
    args = parser.parse_args()

    root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    entries = parse_lib_args(args.lib)

    # tag 构建时防版本漂移：tag（去掉 v 前缀）必须与 Cargo.toml 一致。
    # 放在打包最前面，避免构建产物都齐了才发现版本对不上。
    if args.expected_version:
        crate = read_crate_version(root)
        if crate != args.expected_version:
            die(
                f"版本漂移：期望 {args.expected_version}，但 Cargo.toml 的 version 是 {crate}。"
                f"请把 Cargo.toml 的 version 改为 {args.expected_version} 并提交后重新打 tag。"
            )

    # 同一个 zip 内所有 ABI 必须用同一个库文件名：manifest 只有一个 entryPoint
    lib_names = {os.path.basename(path) for _, path in entries}
    if len(lib_names) != 1:
        die(f"各 ABI 的库文件名必须一致（manifest 只有一个 entryPoint），收到: {sorted(lib_names)}")
    entry_point = lib_names.pop()

    suffix = EXPECTED_SUFFIX[args.platform]
    if not entry_point.endswith(suffix):
        die(
            f"{args.platform} 的产物应为 *{suffix}，收到 {entry_point!r} "
            f"—— 多半是拷错了平台的构建目录"
        )

    staging = args.staging_dir or os.path.join(root, "target", "cp_module", args.platform)
    out_zip = args.out or os.path.join(root, "target", f"ncm-api-rs-{args.platform}.zip")
    os.makedirs(os.path.dirname(out_zip), exist_ok=True)

    if os.path.isdir(staging):
        for dirpath, _dirnames, filenames in os.walk(staging, topdown=False):
            for name in filenames:
                os.remove(os.path.join(dirpath, name))
            os.rmdir(dirpath)
    os.makedirs(staging, exist_ok=True)

    abis = []
    for abi, path in entries:
        if not os.path.isfile(path):
            die(f"找不到动态库: {path}")
        size = os.path.getsize(path)
        if size < 1024:
            die(f"动态库过小 ({size} bytes)，可能构建失败或已损坏: {path}")
        abis.append(abi)
        dest_dir = os.path.join(staging, "lib", abi)
        os.makedirs(dest_dir, exist_ok=True)
        dest = os.path.join(dest_dir, entry_point)
        with open(path, "rb") as src, open(dest, "wb") as dst:
            dst.write(src.read())
        print(f"  + lib/{abi}/{entry_point}  ({size} bytes)")

    version = args.version or read_crate_version(root)
    manifest = {
        "id": args.module_id,
        "name": args.name,
        "version": version,
        "type": "jni",
        "entryPoint": entry_point,
        "supportedAbis": abis,
        "apiMap": {},
    }
    manifest_path = os.path.join(staging, "manifest.json")
    # newline="" + encoding utf-8：不走平台默认编码，也不写 BOM
    with open(manifest_path, "w", encoding="utf-8", newline="") as f:
        json.dump(manifest, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print(f"  + manifest.json  id={manifest['id']} version={version} abis={abis}")

    if os.path.exists(out_zip):
        os.remove(out_zip)
    with zipfile.ZipFile(out_zip, "w", zipfile.ZIP_DEFLATED) as zf:
        for dirpath, _dirnames, filenames in os.walk(staging):
            for name in sorted(filenames):
                full = os.path.join(dirpath, name)
                zf.write(full, os.path.relpath(full, staging))

    print(f"\nOK -> {os.path.relpath(out_zip, root)}")
    with zipfile.ZipFile(out_zip) as zf:
        for info in zf.infolist():
            print(f"    {info.filename}  ({info.file_size} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""把 Cargo.toml 的 [package].version 改写为指定版本（零依赖，只用标准库）。

CI 打 tag 构建时用：tag 是版本的唯一真相源，构建前先把 Cargo.toml
同步成 tag 版本，这样 manifest.json 和二进制内嵌的
`env!("CARGO_PKG_VERSION")` 都自动跟随 tag，不需要手动改 Cargo.toml。

只替换 [package] 段的 version（文件顶部第一处 `version = "..."`），
不会碰到 [dependencies] 里各依赖自己的 version 字段。

用法：
    python scripts/set_cargo_version.py 0.3.0
"""

from __future__ import annotations

import os
import re
import sys

CARGO_TOML = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "Cargo.toml")


def main() -> int:
    if len(sys.argv) != 2:
        print("用法: python scripts/set_cargo_version.py <version>", file=sys.stderr)
        return 1
    version = sys.argv[1].strip()
    if not re.fullmatch(r"\d+\.\d+\.\d+(-[\w.-]+)?", version):
        print(f"Error: 非法版本号 {version!r}（期望 semver，如 0.3.0）", file=sys.stderr)
        return 1

    path = os.path.abspath(CARGO_TOML)
    with open(path, encoding="utf-8") as f:
        text = f.read()

    # 只改文件里第一处 version = "..."，即 [package] 段（它在文件最顶部）
    new_text, n = re.subn(
        r'(?m)^(\s*version\s*=\s*)"[^"]*"',
        rf'\g<1>"{version}"',
        text,
        count=1,
    )
    if n == 0:
        print("Error: Cargo.toml 里找不到 [package] 的 version 行", file=sys.stderr)
        return 1

    with open(path, "w", encoding="utf-8", newline="") as f:
        f.write(new_text)

    print(f"Cargo.toml version -> {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

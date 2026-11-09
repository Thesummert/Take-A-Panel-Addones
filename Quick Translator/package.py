#!/usr/bin/env python3
"""Build a self-contained plugin archive for the current Rust host."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parent
host_info = subprocess.check_output(["rustc", "-vV"], text=True)
host = next(line.split(": ", 1)[1] for line in host_info.splitlines() if line.startswith("host: "))
exe = "quick-translator.exe" if os.name == "nt" else "quick-translator"
target = ROOT / "target"
subprocess.run(["cargo", "build", "--release", "--locked", "--target-dir", str(target)], cwd=ROOT, check=True)
built = target / "release" / exe
bin_dir = ROOT / "bin"
bin_dir.mkdir(exist_ok=True)
shutil.copy2(built, bin_dir / exe)
manifest = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))
manifest["executable"] = f"bin/{exe}"
dist = ROOT / "dist"
dist.mkdir(exist_ok=True)
archive = dist / f"quick-translator-{host}.zip"
with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as bundle:
    bundle.writestr("Quick Translator/manifest.json", json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    bundle.write(ROOT / "README.md", "Quick Translator/README.md")
    bundle.write(bin_dir / exe, f"Quick Translator/bin/{exe}")
print(f"插件压缩包：{archive}")

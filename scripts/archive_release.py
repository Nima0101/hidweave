"""Package native release binary, license and README with a platform label."""
import os
from pathlib import Path
import shutil
import subprocess
import json

root = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=root))
version = metadata["packages"][0]["version"]
platform = os.environ["RELEASE_PLATFORM"]
if platform not in {"linux-x86_64", "macos-arm64", "windows-x86_64"}:
    raise SystemExit("unsupported release platform")
expected = {"linux-x86_64": "x86_64-unknown-linux-gnu", "macos-arm64": "aarch64-apple-darwin", "windows-x86_64": "x86_64-pc-windows-msvc"}[platform]
compiler = subprocess.check_output(["rustc", "-vV"], text=True)
if f"host: {expected}" not in compiler:
    raise SystemExit("compiler host does not match release label")
name = f"hidweave-{version}-{platform}"
stage = root / "target" / "release-stage" / name
stage.mkdir(parents=True, exist_ok=True)
binary = "hidweave.exe" if platform.startswith("windows") else "hidweave"
for source in (root / "target" / "release" / binary, *(root / name for name in ("LICENSE", "README.md", "SECURITY.md", "CONTRIBUTING.md", "CODE_OF_CONDUCT.md", "CHANGELOG.md"))):
    shutil.copy2(source, stage / source.name)
for directory in ("docs", "examples", "benchmarks", "scripts", "fuzz"):
    # Copy only explicitly public text assets; never local launcher or generated corpus.
    destination = stage / directory
    destination.mkdir(exist_ok=True)
    for source in (root / directory).iterdir():
        if source.is_file() and source.suffix in {".md", ".hex", ".rs", ".py"}:
            shutil.copy2(source, destination / source.name)
(stage / "BINARY-QUICKSTART.txt").write_text(
    "Run ./hidweave demo (Windows: .\\hidweave.exe demo).\n"
    "No Rust toolchain is needed for the binary.\n"
    "Compare the bundled fixtures: ./hidweave compare examples/axis-old.hex examples/axis-swapped.hex --hex\n"
    "Exit 1 is expected for this intentional regression. See docs/contract.md.\n"
    "For building/testing from source, clone the repository using README.md.\n",
    encoding="utf-8",
)
dist = root / "dist"
dist.mkdir(exist_ok=True)
shutil.make_archive(str(dist / name), "zip" if platform.startswith("windows") else "gztar", stage.parent, name)

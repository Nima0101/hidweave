"""Build the packaged crate as an external consumer, not as a workspace member."""
import json
import shutil
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
# Package an exact manifest-selected source snapshot so the pre-commit gate does
# not require committing unchecked changes. The package build itself stays strict.
source_snapshot = tempfile.TemporaryDirectory(prefix="hidweave-package-source-")
source = Path(source_snapshot.name)
listing = subprocess.check_output(["cargo", "package", "--list", "--locked", "--offline", "--allow-dirty"], cwd=root, text=True)
for name in listing.splitlines():
    if name in ("Cargo.toml.orig", ".cargo_vcs_info.json"):
        continue
    relative = Path(name)
    if relative.is_absolute() or ".." in relative.parts or not (root / relative).is_file():
        raise ValueError("invalid source inventory")
    destination = source / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(root / relative, destination)
subprocess.run(["cargo", "package", "--locked", "--offline"], cwd=source, check=True)
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=root))
version = metadata["packages"][0]["version"]
package = source / "target" / "package" / f"hidweave-{version}"
with tempfile.TemporaryDirectory(prefix="hidweave-consumer-") as directory:
    consumer = Path(directory)
    (consumer / "src").mkdir()
    # JSON quoting is valid for this TOML basic string, including Windows paths.
    (consumer / "Cargo.toml").write_text(
        '[package]\nname="contract-consumer"\nversion="0.0.0"\nedition="2024"\n'
        '[dependencies]\nhidweave={path=' + json.dumps(str(package)) + '}\n', encoding="utf-8"
    )
    (consumer / "src" / "main.rs").write_text(
        'use hidweave::{parse,parse_hex,compare,decode,ReportKind};\n'
        'fn main(){let l=parse(&parse_hex("05 01 09 04 a1 01 15 00 25 7f 75 08 95 01 09 30 81 02 c0").unwrap()).unwrap();'
        'assert!(compare(&l,&l).is_empty());assert_eq!(decode(&l,ReportKind::Input,&[42]).unwrap()[0].value,42);}\n', encoding="utf-8"
    )
    subprocess.run(["cargo", "run", "--offline"], cwd=consumer, check=True)
source_snapshot.cleanup()
print("Packaged external library consumer passed")

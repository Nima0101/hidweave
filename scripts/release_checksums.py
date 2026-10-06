"""Hash release archives in stable filename order."""
import hashlib
from pathlib import Path

dist = Path(__file__).resolve().parents[1] / "dist"
archives = sorted(path for path in dist.iterdir() if path.name.endswith((".tar.gz", ".zip")))
if len(archives) != 3:
    raise SystemExit("expected exactly three native release archives")
(dist / "SHA256SUMS").write_text("".join(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n" for path in archives), encoding="utf-8")

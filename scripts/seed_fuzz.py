"""Create original binary seeds from the checked-in synthetic hex fixtures."""
from pathlib import Path

root = Path(__file__).resolve().parents[1]
corpus = root / "fuzz" / "corpus" / "descriptor"
corpus.mkdir(parents=True, exist_ok=True)
for source in sorted((root / "examples").glob("*.hex")):
    text = " ".join(line.split("#", 1)[0] for line in source.read_text().splitlines())
    (corpus / source.stem).write_bytes(bytes.fromhex(text))

"""Exercise CLI and the exact CI runner against independent descriptor cases."""
import json
import pathlib
import subprocess
import sys
import tempfile

root = pathlib.Path(__file__).resolve().parents[2]
binary = root / "target" / "debug" / ("hidweave.exe" if sys.platform == "win32" else "hidweave")
with tempfile.TemporaryDirectory() as tmp:
    tmp = pathlib.Path(tmp)
    # A new Output report is separate from the existing unnumbered Input report.
    additive = tmp / "added.hex"
    additive.write_text((root / "examples/axis-old.hex").read_text() + "\n05 01 09 04 a1 01 09 30 15 00 25 7f 75 08 95 01 91 02 c0\n")
    cases = [("axis-equivalent.hex", "strict", 0), ("axis-swapped.hex", "strict", 1),
             ("delimiter.hex", "strict", 2), (str(additive), "strict", 1),
             (str(additive), "add-reports", 0), ("axis-swapped.hex", "add-reports", 1),
             ("delimiter.hex", "add-reports", 2)]
    for candidate, policy, code in cases:
        candidate = root / "examples" / candidate
        command = [sys.executable, str(root / "integrations/ci_gate.py"), str(binary),
                   str(root / "examples/axis-old.hex"), str(candidate), str(tmp / "review.json"), "--hex", "--policy", policy]
        previous = None
        for _ in range(2):
            result = subprocess.run(command, capture_output=True, text=True)
            assert result.returncode == code, (command, result.stdout, result.stderr)
            raw = (tmp / "review.json").read_bytes()
            if previous is not None:
                assert raw == previous
            previous = raw
            document = json.loads(raw)
            assert set(document) == {"schema_version", "contract_version", "policy", "status", "error", "findings"}
            assert document["schema_version"] == document["contract_version"] == 1
            assert document["status"] == {0: "pass", 1: "deny", 2: "invalid"}[code]
            assert document["policy"] == policy
            if code == 2:
                assert document["error"] and document["findings"] == []
            elif code == 0:
                assert document["error"] is None and all(f["allowed"] for f in document["findings"])
            else:
                assert document["error"] is None and any(not f["allowed"] for f in document["findings"])
            for finding in document["findings"]:
                assert set(finding) == {"category", "allowed", "report_kind", "report_id", "properties"}
                assert type(finding["allowed"]) is bool
                assert finding["report_kind"] in ("input", "output", "feature")
                assert type(finding["report_id"]) is int and 0 <= finding["report_id"] <= 255
                assert all(isinstance(p, str) for p in finding["properties"])
    # Allowing additions must never allow removals (reverse comparison).
    removed = subprocess.run([str(binary), "gate", str(additive), str(root / "examples/axis-old.hex"), "--hex", "--policy", "add-reports"], capture_output=True, text=True)
    assert removed.returncode == 1 and json.loads(removed.stdout)["status"] == "deny"
    bad_policy = subprocess.run([str(binary), "gate", "a", "b", "--policy", "allow-all"], capture_output=True)
    assert bad_policy.returncode == 2
print("CI runner: equivalence, denied semantics, explicit additions, denied removals, unsupported input, deterministic artifacts passed")

"""Independently parse CLI JSON and check its public success/error contract."""
import json
import os
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
subprocess.run(["cargo", "build", "--locked", "--offline"], cwd=root, check=True)
binary = root / "target" / "debug" / ("hidweave.exe" if os.name == "nt" else "hidweave")


def run(*args, code=0):
    result = subprocess.run([str(binary), *args], cwd=root, capture_output=True, check=False)
    assert result.returncode == code, (args, result.returncode, result.stderr)
    assert not result.stderr, result.stderr
    return json.loads(result.stdout)


old = "examples/axis-old.hex"
same = "examples/axis-equivalent.hex"
new = "examples/axis-swapped.hex"
report = "examples/axis-report.hex"
inspection = run("inspect", old, "--hex", "--json")
assert inspection["contract_version"] == 1
assert inspection["reports"][0]["wire_bytes"] == 2
assert inspection["reports"][0]["fields"][0]["meaning"]["usage"] == 65584
assert run("compare", old, same, "--hex", "--json")["equal"]
change = run("compare", old, new, "--hex", "--json", "--report", report, "--kind", "input", code=1)
assert len(change["changes"]) == 2
assert change["evidence"]["old"]["values"][0]["usage"] == 65584
assert change["evidence"]["new"]["values"][1]["usage"] == 65584
assert run("decode", old, report, "--hex", "--json", "--kind", "input")["decoded"]["values"][0]["value"] == 1
bad = run("decode", old, old, "--hex", "--json", "--kind", "input", code=2)
assert bad["decoded"]["error"]["code"] == "report_length"
bad_evidence = run("compare", old, same, "--hex", "--json", "--report", old, "--kind", "input")
assert bad_evidence["equal"]
assert bad_evidence["evidence"]["old"]["error"]["code"] == "report_length"
unsupported = subprocess.run([str(binary), "inspect", "examples/delimiter.hex", "--hex", "--json"], cwd=root, capture_output=True)
assert unsupported.returncode == 2 and not unsupported.stdout
assert b"local_tag" in unsupported.stderr
print("JSON contract: inspection, equality, differences, evidence, decode, and errors passed")

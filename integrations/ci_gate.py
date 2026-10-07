"""Run a pinned hidweave binary, retain its review artifact, propagate its status."""
import argparse
import json
import pathlib
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary")
    parser.add_argument("baseline")
    parser.add_argument("candidate")
    parser.add_argument("artifact")
    parser.add_argument("--policy", choices=("strict", "add-reports"), default="strict")
    parser.add_argument("--hex", action="store_true")
    args = parser.parse_args()
    # Resolve paths so filenames starting with '-' cannot be interpreted as CLI options.
    command = [str(pathlib.Path(args.binary).resolve()), "gate", str(pathlib.Path(args.baseline).resolve()),
               str(pathlib.Path(args.candidate).resolve()), "--policy", args.policy]
    if args.hex:
        command.append("--hex")
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    if result.returncode not in (0, 1, 2):
        raise RuntimeError(f"hidweave failed with status {result.returncode}")
    document = json.loads(result.stdout)
    if (document["schema_version"], document["contract_version"]) != (1, 1):
        raise ValueError("unsupported artifact contract")
    expected = {"pass": 0, "deny": 1, "invalid": 2}[document["status"]]
    if expected != result.returncode or document["policy"] != args.policy:
        raise ValueError("inconsistent artifact")
    pathlib.Path(args.artifact).write_text(result.stdout, encoding="utf-8")
    print(f"hidweave: {document['status']} ({len(document['findings'])} findings)")
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())

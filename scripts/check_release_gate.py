"""Fail release publication unless both required workflows passed this commit."""
import json
import os
import subprocess

sha = os.environ["GITHUB_SHA"]
repository = os.environ["GITHUB_REPOSITORY"]
runs = json.loads(subprocess.check_output([
    "gh", "run", "list", "--repo", repository, "--commit", sha, "--branch", "main",
    "--limit", "100", "--json", "workflowName,status,conclusion,headSha,event",
]))
for workflow in ("CI", "Security"):
    # gh returns newest first. A newer failed or pending run must not be hidden by
    # an earlier passing run for the same commit.
    candidates = [run for run in runs if run["workflowName"] == workflow and run["headSha"] == sha and run["event"] in {"push", "workflow_dispatch"}]
    if not candidates or candidates[0]["status"] != "completed" or candidates[0]["conclusion"] != "success":
        raise SystemExit(f"Release blocked: {workflow} has not passed on the tagged main commit")
print("Exact-commit CI and Security release gate passed")

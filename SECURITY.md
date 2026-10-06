# Security policy

The latest 0.1.x release receives security fixes. This is a new project with no
guaranteed response SLA. See [the threat model](docs/threat-model.md).

Report exploitable parser bugs, resource-limit bypasses or security-relevant false
equality through [GitHub private vulnerability reporting](https://github.com/Nima0101/hidweave/security/advisories/new).
If that channel is unavailable, open an issue asking for a private contact without
including exploit details or sensitive captures. Never upload credentials or real
keystroke reports. A minimal synthetic descriptor is preferred.

Include the version, command, platform, expected outcome and a minimized input
when safe. Please allow coordinated remediation before public disclosure.

Ordinary unsupported descriptors and conservative comparison differences can be
reported publicly. Security and correctness fixes should include a regression
test and assessment of whether the public comparison contract changes.

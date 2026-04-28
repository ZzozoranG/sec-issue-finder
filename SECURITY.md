# Security Policy

## Supported Versions

The project is preparing an initial `0.1.0` release. Until a stable release process is established, security fixes target the default branch.

## Reporting a Vulnerability

Please do not open a public issue for suspected vulnerabilities in `sec-issue-finder`.

Use the repository security advisory workflow if available. If not available, contact the maintainers privately using the security contact listed by the repository owner.

Include:

- Affected version or commit.
- Description of the issue.
- Reproduction steps or proof of concept, if safe to share.
- Potential impact.
- Any suggested mitigation.

## Security Model

`sec-issue-finder` reads lockfiles as untrusted input, normalizes dependency data, queries public advisory sources, and reports known advisory matches.

It does not execute package manager commands, dependency scripts, or auto-fix logic. It does not claim complete vulnerability coverage. Results depend on public advisory databases and available package/version metadata.

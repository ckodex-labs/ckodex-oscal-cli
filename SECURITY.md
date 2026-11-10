# Security Policy

## Supported Versions

| Version | Supported |
| :--- | :--- |
| `1.0.x` | Yes |
| `< 1.0.0` | No |

## Reporting a Vulnerability

Please **do not** open public GitHub issues for suspected security vulnerabilities.

Report vulnerabilities privately through **GitHub Security Advisories**:
- https://github.com/ckodex-labs/ckodex-oscal-cli/security/advisories/new

Include:
1. Affected version, subcommand, or surface (`mizan` CLI, MCP server, UTCP adapter, gRPC transport, or Workbench).
2. Minimal reproduction steps or input fixture.
3. Expected behavior vs observed behavior and security impact.

We acknowledge private reports within 72 hours and publish coordinated advisories once a verified fix is tagged.

## Built-In Security Controls & Verification

- **Artifact Provenance**: Release archives (`mizan-v*`) and `SHA256SUMS.txt` are attested in GitHub Actions with `actions/attest-build-provenance@v2` (Sigstore / SLSA). Verify downloaded binaries with:
  ```bash
  gh attestation verify mizan-v1.0.0-x86_64-unknown-linux-gnu.tar.gz --repo ckodex-labs/ckodex-oscal-cli
  ```
- **Memory & Wire Safety**: `#![deny(unsafe_code)]` enforced across the Rust crate, `64 MiB` regular-file input bounds (rejecting FIFOs, device nodes, and symlink traversal), `16 MiB` MCP/gRPC frame caps, and canonicalized workspace path confinement across MCP, Workbench HTTP routes, and Tauri IPC.
- **Supply-Chain Audits**: Continuous `cargo deny check` and `npm audit` gates run on every commit and pull request.

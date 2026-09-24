# Mizan · High-Assurance OSCAL Compliance Kernel & Workbench

<p align="center">
  <a href="https://github.com/MChorfa/shieldcn-zig"><img src="https://shieldcn.dev/badge/badge%20engine-shieldcn--zig-2dd4bf.svg?variant=secondary&wcag=3" alt="Badge Engine: shieldcn-zig" /></a>
  <img src="https://shieldcn.dev/badge/rust-2024%20edition-black.svg?variant=secondary&wcag=3&logo=rust" alt="Rust 2024 Edition" />
  <img src="https://shieldcn.dev/badge/oscal-v1.2.3%20metaschema-blue.svg?variant=secondary&wcag=3" alt="OSCAL v1.2.3 Metaschema" />
  <img src="https://shieldcn.dev/badge/next.js-v16.3.6%20turbopack-blue.svg?variant=secondary&wcag=3&logo=nextdotjs" alt="Next.js 16 Turbopack" />
  <img src="https://shieldcn.dev/badge/provenance-slsa%20v1.2%20%26%20v1.0-emerald.svg?variant=secondary&wcag=3" alt="SLSA v1.2 & v1.0 Provenance Generator" />
  <img src="https://shieldcn.dev/badge/baselines-fedramp%20%C2%B7%20itsg--33%20%C2%B7%20iso27001-green.svg?variant=secondary&wcag=3&logo=shield" alt="FedRAMP, ITSG-33, ISO 27001 Baselines" />
  <img src="https://shieldcn.dev/badge/tests-160%20passed-green.svg?variant=secondary&wcag=3" alt="Tests: 160 passed" />
  <img src="https://shieldcn.dev/badge/license-Apache--2.0-gray.svg?variant=secondary&wcag=3" alt="License: Apache-2.0" />
</p>

> **Live Interactive Workbench**: [https://ckodex-labs.github.io/ckodex-oscal-cli/](https://ckodex-labs.github.io/ckodex-oscal-cli/)  
> **Rustdoc API Documentation**: [https://ckodex-labs.github.io/ckodex-oscal-cli/docs/api/mizan/index.html](https://ckodex-labs.github.io/ckodex-oscal-cli/docs/api/mizan/index.html)  
> **Offline Merkle Evidence Capsule**: [https://ckodex-labs.github.io/ckodex-oscal-cli/capsule.html](https://ckodex-labs.github.io/ckodex-oscal-cli/capsule.html)  
> **Multi-Platform Binary Releases**: [GitHub Releases v0.1.0](https://github.com/ckodex-labs/ckodex-oscal-cli/releases)

---

## Overview

**Mizan** is a high-assurance OSCAL compliance kernel, headless CLI, terminal TUI, and Next.js 16 workbench designed for modern security engineering.

It provides complete drop-in parity with NIST `oscal-cli`, augmented with:
- **3-Way GitOps AST Synchronization**: Bidirectional conflict-free merges across distributed control baselines.
- **Tri-Jurisdiction Harmonization**: Continuous alignment across **US FedRAMP Rev 5 (NIST SP 800-53)**, **Canada CCCS ITSG-33 (PBMM)**, and **EU EUCS / ISO/IEC 27001:2022**.
- **Cryptographic Evidence Capsules**: Air-gap verifiable standalone HTML audit packages with in-browser WebCrypto Merkle proofs.
- **Continuous Assurance Gates**: Pre-commit policy validation (Rego/Kyverno), in-toto SLSA provenance generation, and blast-radius impact analysis.
- **Prominent-Language Dagger Pipeline**: Containerized CI/CD orchestrated natively with the official **Dagger Rust SDK** (`dagger-sdk = "0.21.9"`).
- **shieldcn-zig Badging**: APCA WCAG 3.0-compliant contrast badge engine (`?wcag=3`, $|L_c| \ge 60$).

---

## Quickstart

### Pre-Built Binaries (Linux, macOS, Windows)

Download the release binary matching your target architecture directly from [GitHub Releases](https://github.com/ckodex-labs/ckodex-oscal-cli/releases):

```bash
# Linux x86_64 (glibc)
curl -sLO https://github.com/ckodex-labs/ckodex-oscal-cli/releases/download/v0.1.0/mizan-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
tar -xzf mizan-v0.1.0-x86_64-unknown-linux-gnu.tar.gz && sudo mv mizan-v0.1.0-x86_64-unknown-linux-gnu/mizan /usr/local/bin/

# Linux ARM64
curl -sLO https://github.com/ckodex-labs/ckodex-oscal-cli/releases/download/v0.1.0/mizan-v0.1.0-aarch64-unknown-linux-gnu.tar.gz
tar -xzf mizan-v0.1.0-aarch64-unknown-linux-gnu.tar.gz && sudo mv mizan-v0.1.0-aarch64-unknown-linux-gnu/mizan /usr/local/bin/

# macOS Apple Silicon (ARM64)
curl -sLO https://github.com/ckodex-labs/ckodex-oscal-cli/releases/download/v0.1.0/mizan-v0.1.0-aarch64-apple-darwin.tar.gz
tar -xzf mizan-v0.1.0-aarch64-apple-darwin.tar.gz && sudo mv mizan-v0.1.0-aarch64-apple-darwin/mizan /usr/local/bin/

# macOS Intel (x86_64)
curl -sLO https://github.com/ckodex-labs/ckodex-oscal-cli/releases/download/v0.1.0/mizan-v0.1.0-x86_64-apple-darwin.tar.gz
tar -xzf mizan-v0.1.0-x86_64-apple-darwin.tar.gz && sudo mv mizan-v0.1.0-x86_64-apple-darwin/mizan /usr/local/bin/
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/ckodex-labs/ckodex-oscal-cli.git
cd ckodex-oscal-cli

# Build release binaries (Fat-LTO optimized)
cargo build --release

# Run validation directly
./target/release/mizan --help
```

### 30-Second Verification

```bash
# 1. Validate an OSCAL document against official v1.2.3 metaschemas
cargo run --bin mizan -- validate examples/sample-catalog.json

# 2. Convert between JSON, YAML, CSV compliance matrix, and binary Protobuf
cargo run --bin mizan -- convert examples/sample-catalog.json --to=csv -o matrix.csv

# 3. Analyze compliance blast radius of an access control change
cargo run --bin mizan -- blast-radius examples/sample-ssp.json --target ac-1

# 4. Export a standalone offline cryptographic evidence capsule
cargo run --bin mizan -- export capsule -a <(echo '{"assessment-results":{"uuid":"demo","metadata":{"title":"Audit Proof"}}}') -o capsule.html

# 5. Launch the Next.js 16 Turbopack Workbench
cd apps/workbench && npm install && npm run dev
```

---

## Architectural Delivery Surfaces

Mizan is architected around hexagonal domain boundaries with six primary operational surfaces:

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        MIZAN COMPLIANCE FABRIC                         │
├─────────────────┬──────────────────┬─────────────────┬─────────────────┤
│  1. THE COMPOSER│  2. THE LEDGER   │  3. THE PIPELINE│ 4. ROOT FABRIC  │
│  NIST 800-53    │  Content-Addrs.  │  Continuous     │ SPIFFE/SPIRE    │
│  Agile Markdown │  SHA256 Receipts │  Assurance      │ Workload Auth   │
│  Workspaces     │  & Time-Decay    │  & Blast Radius │ & Multi-Tenant  │
├─────────────────┴──────────────────┴─────────────────┴─────────────────┤
│  5. JURISDICTIONS & SLSA           │  6. THE ATLAS                     │
│  FedRAMP · ITSG-33 · ISO 27001     │  2D/3D Force-Directed Compliance  │
│  SLSA In-Toto Attestations         │  Dependency Topology Graph        │
└────────────────────────────────────────────────────────────────────────┘
```

1. **The Composer (`apps/workbench`)**: Parameter chips with atomic punctuation binding, agile Markdown splitting (`mizan split` / `assemble`), and baseline delta authoring.
2. **The Ledger**: Immutable, content-addressed audit trail tracking SHA-256 digests, signature provenance, and freshness decay gauges.
3. **The Pipeline**: Pre-commit terminal simulation executing `oscal-schema`, `slsa-provenance`, and `responsible-role` gates.
4. **Root Fabric**: Zero-trust multi-tenant partition isolation backed by SPIFFE workload identifiers and partition quotas.
5. **Jurisdictions & SLSA**: Unified cross-walk mapping international defense/enterprise standards with verifiable in-toto build provenance.
6. **The Atlas**: Force-directed topological graph visualization tracing control cascading impacts and parameter alterations.

---

## Command Reference

### 1. Document Operations (NIST `oscal-cli` Parity)

```bash
# Metaschema validation (strict constraint enforcement)
mizan validate path/to/catalog.json --strict
mizan catalog validate path/to/catalog.json
mizan profile validate path/to/profile.json
mizan ssp validate path/to/ssp.json

# AST Format Conversion
mizan convert input.json --to=yaml -o output.yaml
mizan convert input.json --to=csv -o matrix.csv
mizan convert matrix.csv --to=json -o output.json
mizan convert input.json --to=proto -o output.pb

# Profile Resolution & Semantic Diff
mizan resolve baseline-profile.json -o resolved-catalog.json
mizan diff catalog_v1.json catalog_v2.json
mizan lint path/to/doc.json --fix
```

### 2. GitOps Synchronization & Authoring

```bash
# 3-Way AST Merge & Sync
mizan sync --base base.json --upstream upstream.json --local local.json -o merged.json

# Agile Markdown Decomposition (Split & Assemble)
mizan split path/to/catalog.json -o ./catalog-workspace
mizan assemble ./catalog-workspace -o ./assembled-catalog.json

# Control Deduplication
mizan dedup path/to/catalog.json -o path/to/deduped.json
```

### 3. Continuous Assurance, Blast Radius & Waivers

```bash
# Blast Radius Impact Analysis
mizan blast-radius path/to/ssp.json --target ac-1 --context path/to/catalog.json

# Time-Bounded Cryptographic Derogation Leases (Pipeline Waivers)
mizan waive create --rule cis-k8s-5.2.1 --resource deployment.yaml --duration-days 30 --justification "Migration scheduled"
mizan waive list
mizan waive revoke <lease-id>

# Automated Quick-Fix Engine
mizan fix path/to/doc.json --rule require-remarks --apply
```

### 4. Evidence Capsules & Badging

```bash
# Offline Cryptographic Evidence Capsule (Embedded WebCrypto Merkle Proofs)
mizan export capsule -a assessment-results.json -o capsule.html

# shieldcn-zig APCA WCAG 3.0 Contrast Badge Production
mizan export badge --label "FedRAMP" --message "Moderate COMPLIANT" --color green --variant secondary --wcag 3
mizan export badge --from-document examples/sample-catalog.json --variant secondary --wcag 3
```

---

## High-Assurance Dagger Rust SDK CI/CD

Continuous integration and supply-chain attestations are driven natively in Rust using the official **Dagger Rust SDK** (`dagger-sdk = "0.21.9"`):

```bash
# Run the complete hermetic container DAG in Rust:
cargo run -p mizan-dagger-ci -- all

# Run individual pipeline stages:
cargo run -p mizan-dagger-ci -- lint       # Clippy (-D warnings) & Rustfmt checks
cargo run -p mizan-dagger-ci -- test       # 158 workspace invariant tests
cargo run -p mizan-dagger-ci -- build      # Fat-LTO release binaries
cargo run -p mizan-dagger-ci -- workbench  # Next.js 16 Turbopack production build
cargo run -p mizan-dagger-ci -- badge      # APCA WCAG 3.0 shieldcn-zig badge production

# Execute inside a Dagger engine session:
dagger run cargo run -p mizan-dagger-ci -- all
```

---

## Tri-Jurisdiction Baseline Alignment

Mizan embeds baseline catalogs and mapping logic across three primary jurisdictions:

| Jurisdiction | Authority & Standard | Base Controls | Overlay Scope |
| :--- | :--- | :--- | :--- |
| **United States** | NIST SP 800-53 Rev 5 / FedRAMP High & Moderate | AC-2, AC-3, AC-6, AU-2, AU-6, AU-12, SC-7, SC-12, SC-13, SC-28, SR-3, SR-5, SA-12 | FedRAMP PMO parameters & continuous monitoring |
| **Canada** | CCCS ITSG-33 Protected B / Medium / Medium (PBMM) | AC-2, AC-3, AU-6, SC-7 | Canadian federal cloud boundary & data residency |
| **European Union** | EUCS & ISO/IEC 27001:2022 Controls Alignment | A.5.15, A.8.2, A.8.16, A.8.24 | Sovereign cloud boundary, EU key custody, Annex A controls |
| **Enterprise** | Custom Inherited Overlays (`mizan catalog extend`) | User-defined | Custom corporate controls inheriting from base baselines |

---

## Verification & Test Evidence

- **Rust Workspace**: `160 passed; 0 failed` across 126 unit tests and 34 integration tests (`cargo test --workspace`).
- **Linter & Style**: `cargo clippy --workspace --all-targets -- -D warnings` -> **0 warnings, 0 errors**.
- **Next.js 16 Workbench**: Turbopack compiled in **1.7s** (`apps/workbench`).
- **GitHub Pages Portal**: Automated build and deployment in `.github/workflows/pages.yml` hosting interactive Workbench, evidence capsules, and API references.

---

## License & Attestation

Published under the [Apache License, Version 2.0](LICENSE).  
Maintained by [ckodex-labs](https://github.com/ckodex-labs). Built-in attestation engine supports SLSA v1.2 & v1.0 In-Toto predicate generation and verification.

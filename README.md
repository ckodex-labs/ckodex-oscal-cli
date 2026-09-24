# Mizan · High-Assurance OSCAL Compliance Kernel & Workbench

> **Unified, four-layer OSCAL compliance ecosystem for modern security engineering.**  
> Native Tripartite Delivery: **Headless CLI (`mizan` / `oscal-cli`)** · **Terminal TUI (`mizan tui`)** · **Tauri Desktop & Web Workbench (`apps/workbench`)**.

<p align="center">
  <a href="https://github.com/MChorfa/shieldcn-zig"><img src="https://shieldcn.dev/badge/badge%20engine-shieldcn--zig-2dd4bf.svg?variant=secondary&wcag=3" alt="Badge Engine: shieldcn-zig" /></a>
  <img src="https://shieldcn.dev/badge/rust-2024%20edition-orange.svg?variant=secondary&logo=rust" alt="Rust 2024 Edition" />
  <img src="https://shieldcn.dev/badge/oscal-v1.2.3-blue.svg?variant=secondary" alt="OSCAL v1.2.3 Metaschema" />
  <img src="https://shieldcn.dev/badge/next.js-v16.3.6-black.svg?variant=secondary&logo=nextdotjs" alt="Next.js 16" />
  <img src="https://shieldcn.dev/badge/SLSA-Level%203-emerald.svg?variant=secondary&wcag=3" alt="SLSA Level 3 Attestation" />
  <img src="https://shieldcn.dev/badge/fedramp-moderate%20%26%20high-green.svg?variant=secondary&wcag=3" alt="FedRAMP Moderate & High" />
  <img src="https://shieldcn.dev/badge/tests-158%20passed-green.svg?variant=secondary&wcag=3" alt="Tests: 158 passed" />
  <img src="https://shieldcn.dev/badge/license-Apache--2.0-slate.svg?variant=secondary" alt="License: Apache-2.0" />
</p>

---

## 🏛️ The Four-Layer Hexagonal Architecture

Mizan strictly applies clean hexagonal boundaries across four distinct layers:

1. **Layer 1 — Pure Domain Core (`src/document/`)**: Metaschema AST representations, invariant validators, topological blast radius engines, and semantic AST lineage diffs. Zero I/O, zero network dependencies.
2. **Layer 2 — Application / Use Cases (`src/document/{sync,authoring,policy,fedramp}`)**: 3-Way GitOps AST merge, Agile Markdown split/assemble, Compliance-to-Policy (OPA/Rego & Kyverno) compiler, continuous SBOM/POA&M reconciler, and FedRAMP PMO baseline rules evaluator.
3. **Layer 3 — Inbound & Outbound Adapters (`src/{cli,tui,mcp,transport,capture}`)**: 
   - *Inbound Ports*: Clap CLI dispatcher (`src/cli.rs`), Ratatui terminal loop (`src/tui.rs`), JSON-RPC 2.0 MCP server (`src/mcp/`), and Tauri native IPC commands (`src-tauri`).
   - *Outbound Ports*: Multi-format codecs (JSON, YAML, CSV compliance matrix, Protobuf `.pb`), and high-performance read-only gRPC client calling upstream Squarify (`ckodex-oscalify`) services.
4. **Layer 4 — Presentation Surfaces**:
   - **Headless CLI (`mizan` / `oscal-cli`)**: Scriptable GitOps automation, pre-commit validation, and CI/CD pipelines.
   - **Terminal Observer (`mizan tui`)**: Zero-latency Ratatui terminal visualizer.
   - **Native Desktop App (Tauri `Mizan.app`)**: Native window with zero-overhead in-process Rust IPC.
   - **Mizan Workbench (`apps/workbench`)**: Next.js 16 + React 19 + shadcn/ui AI Copilot styled with CKODEX `ledger`, `vault`, and `hc` themes.

---

The client compiles the exact locked OSCAL protocol snapshot vendored under
`proto/oscal` by default. For protocol review or an intentional snapshot
refresh, `OSCALIFY_PROTO_ROOT` can point at an external source tree, including
the read-only sibling checkout:

```text
../ckodex-oscalify/proto/oscal
```

The external source checkout is never modified. `proto.lock` records the exact
SHA-256 snapshot expected by the generated bindings, and both the vendored
default and any explicit override must match it. A changed proto source fails
the build until the snapshot is intentionally refreshed and reviewed.

To audit an external OSCALify checkout without changing it, compare every
locked proto file and record the observed source-worktree state:

```bash
OSCALIFY_ROOT=../ckodex-oscalify \
  ./scripts/protocol-audit.sh --output target/protocol-audit.json
```

The audit exits non-zero when the external source drifts from `proto.lock` and
still writes the report, including the exact unified diff for each drifted
locked file. A drift report is evidence for upstream review, not permission to
refresh or modify the OSCALify checkout.

The audit also pins the OSCAL release metadata behind those protobufs. It
compares OSCALify's version pin and verified official schema manifest with
`proto.lock`, so a schema-only release cannot pass merely because the RPC file
set stayed unchanged. The CLI does not vendor the official JSON Schemas because
it observes protobuf services and does not validate OSCAL documents.

After reviewing a drift report, refresh the vendored snapshot explicitly:

```bash
./scripts/protocol-sync.sh --check
./scripts/protocol-sync.sh --sync
```

For a one-command maintenance loop, use:

```bash
./scripts/protocol-maintain.sh --audit-output target/protocol-audit.json
./scripts/protocol-maintain.sh --sync
./scripts/protocol-maintain.sh --sync --allow-dirty-source
```

Sync refuses a dirty source checkout by default. When the reviewed contract is
intentionally an OSCALify workspace snapshot, `--allow-dirty-source` records
that fact in `proto.lock`. The command stages and verifies all 13 allowlisted
protobufs, validates all nine files in the official OSCAL schema manifest, and
never writes to OSCALify.

The client snapshot includes the currently observed graph projection-event
contract. The source revision is still subject to external ownership and beta
acceptance; future source drift remains fail-closed. See
[`PROTOCOL_ALIGNMENT.md`](PROTOCOL_ALIGNMENT.md).

If a server predates this read path, the client reports an explicit unavailable
capability and points to protocol revision alignment; it never emulates or
silently falls back to another contract.

## Commands

### Local Document Operations (NIST `oscal-cli` Parity & Modern Tooling)

```bash
# Validate any OSCAL document (JSON or YAML) against official OSCAL 1.2 schemas & constraints
cargo run --bin mizan -- validate path/to/catalog.json
cargo run --bin mizan -- validate path/to/ssp.yaml --strict

# Canonical NIST oscal-cli syntax:
cargo run --bin mizan -- catalog validate path/to/catalog.json
cargo run --bin mizan -- profile validate path/to/profile.json
cargo run --bin mizan -- ssp validate path/to/ssp.json

# Convert between JSON, YAML, CSV compliance matrix, and binary Protobuf formats
cargo run --bin mizan -- convert input.json --to=yaml -o output.yaml
cargo run --bin mizan -- convert input.json --to=csv -o matrix.csv
cargo run --bin mizan -- convert matrix.csv --to=json -o output.json
cargo run --bin mizan -- convert input.json --to=proto -o output.pb

# 3-Way GitOps AST merge & synchronization
cargo run --bin mizan -- sync --base base.json --upstream upstream-update.json --local local-edits.json -o merged.json
cargo run --bin mizan -- sync --base base.json --upstream upstream.json --local local.json --strategy manual --split-to ./workspace

# Resolve OSCAL profiles against imported catalogs
cargo run --bin mizan -- resolve baseline-profile.json -o resolved-catalog.json
cargo run --bin mizan -- profile resolve baseline-profile.json -o resolved-catalog.json

# Semantic diff between two OSCAL documents
cargo run --bin mizan -- diff catalog_v1.json catalog_v2.json

# Lint and auto-fix OSCAL metadata and UUID pitfalls
cargo run --bin mizan -- lint path/to/doc.json --fix

# Inspect and summarize OSCAL model metrics in CKODEX-DS-3 ledger view
cargo run --bin mizan -- inspect path/to/catalog.json

# Multi-model compliance blast radius impact analyzer
cargo run --bin mizan -- blast-radius path/to/ssp.json --target ac-1 --context path/to/catalog.json

# Deduplicate controls, components, parties, and resources
cargo run --bin mizan -- dedup path/to/catalog.json -o path/to/deduped.json
cargo run --bin mizan -- dedup path/to/catalog.json --dry-run

# Reconcile declared compliance against observed inventory and assessment findings
cargo run --bin mizan -- reconcile --ssp path/to/ssp.json --inventory path/to/sbom.json
cargo run --bin mizan -- reconcile --results path/to/results.json --poam path/to/poam.json --strict

# Agile Markdown GitOps authoring (split & assemble)
cargo run --bin mizan -- split path/to/catalog.json -o ./catalog-workspace
cargo run --bin mizan -- assemble ./catalog-workspace -o ./assembled-catalog.json

# End-to-End Compliance Pipeline Orchestrator (Single-step CI/CD execution)
cargo run --bin mizan -- pipeline run -j us --sbom cyclonedx.json -w workload.yaml --subject app:v1.0.0 -o ./pipeline-output

# Continuous Compliance Background Daemon & Workspace Watcher
cargo run --bin mizan -- daemon --dir ./workspace --debounce-ms 500
cargo run --bin mizan -- watch ./workspace

# Embedded Tri-Jurisdictional Baselines & Enterprise Customizer
cargo run --bin mizan -- catalog list
cargo run --bin mizan -- catalog export -j us -o nist-catalog.json
cargo run --bin mizan -- catalog export -j ca -o itsg33-catalog.json
cargo run --bin mizan -- catalog export -j eu -o eucs-catalog.json
cargo run --bin mizan -- catalog extend -b us --title "Fintech Cloud Baseline" --add-control-id bank-01 --add-control-title "Hardware HSM" -o ent-catalog.json

# Supply Chain Provenance (SLSA v1.2 & in-toto v1.0 Attestations)
cargo run --bin mizan -- attest slsa --subject app-binary --digest <sha256> --version v1.2 -e evidence.json -o slsa-statement.json
cargo run --bin mizan -- attest verify slsa-statement.json

# CI/CD Security Scanner Exporters (OASIS SARIF v2.1.0 & GitLab Security Report v15.0.0)
cargo run --bin mizan -- export sarif -i path/to/assessment.json -o mizan-sarif.json
cargo run --bin mizan -- export gitlab -i path/to/assessment.json -o gl-security-report.json

# Software Bill of Materials (SBOM) Ingestion (CycloneDX v1.5/v1.6 & SPDX)
cargo run --bin mizan -- sbom import -i cyclonedx.json -o oscal-component-definition.json

# Built-in Regorus Compliance Rulepacks (CIS K8s 1.8, FedRAMP High, CCCS ITSG-33)
cargo run --bin mizan -- policy rulepack list
cargo run --bin mizan -- policy rulepack eval -r cis-k8s-5.2.1 -i workload.json

# Native Model Context Protocol (MCP) stdio server for AI agents (13 Tools)
cargo run --bin mizan -- mcp

# Developer Adoption & Cold-Start Onboarding (Repo Scanner & Workspace Scaffolder)
cargo run --bin mizan -- init --from-repo . --output ./governance --jurisdiction us

# Automated AST & Regex Remediation for CIS Benchmarks & Dockerfiles
cargo run --bin mizan -- fix --dry-run
cargo run --bin mizan -- fix --confirm --rule cis-k8s-5.2.6

# Time-Bounded Derogation Leases (Zero-Friction Pipeline Waivers)
cargo run --bin mizan -- waive create --rule cis-k8s-5.2.1 --resource deployment.yaml --duration-days 30 --justification "Migration to non-root scheduled"
cargo run --bin mizan -- waive list
cargo run --bin mizan -- waive revoke <lease-id>

# Auditor Matrix CSV Export & 3-Way AST Bidirectional Synchronization
cargo run --bin mizan -- catalog export-matrix -j us -o auditor-matrix.csv
cargo run --bin mizan -- sync --base base.json --matrix auditor-matrix.csv -o reconciled-catalog.json

# Zero-Install Offline Cryptographic Evidence Capsule (Embedded WebCrypto Merkle Verification)
cargo run --bin mizan -- export capsule --bundle evidence-bundle.json --output capsule.html

# shieldcn-zig Pure-Zig SVG/PNG/JSON Badge Generator (WCAG 3.0 APCA |Lc| >= 60 Compliant)
cargo run --bin mizan -- export badge --label "FedRAMP" --message "Moderate COMPLIANT" --color green --variant secondary --wcag 3
cargo run --bin mizan -- export badge --from-document ./examples/sample-catalog.json --variant secondary --wcag 3
```

### High-Assurance Dagger CI/CD Pipeline (Official Dagger Rust SDK)

The continuous integration and provenance pipeline is powered natively by the official **Dagger Rust SDK** (`dagger-sdk = "0.21.9"`), matching the prominent language of the project. It executes hermetic, containerized verification with content-addressed cache volumes and automated `shieldcn-zig` badge production:

```bash
# Run the complete Dagger pipeline (Clippy, 158 tests, Fat-LTO build, Next.js 16 Workbench, Badges)
cargo run -p mizan-dagger-ci -- all

# Or execute individual Dagger pipeline stages:
cargo run -p mizan-dagger-ci -- lint       # Clippy (-D warnings) & Rustfmt checks
cargo run -p mizan-dagger-ci -- test       # 158 workspace invariant tests
cargo run -p mizan-dagger-ci -- build      # Fat-LTO release binaries (mizan, oscal-cli)
cargo run -p mizan-dagger-ci -- workbench  # Next.js 16 Turbopack production build
cargo run -p mizan-dagger-ci -- badge      # APCA WCAG 3.0 shieldcn-zig badge generation

# Alternatively, execute under Dagger CLI session:
dagger run cargo run -p mizan-dagger-ci -- all
```

### Remote OSCALify Observer & Transparency Operations

```bash
cargo run --bin mizan -- proto
cargo run --bin mizan -- health
cargo run --bin mizan -- model list catalog
cargo run --bin mizan -- entity list
cargo run --bin mizan -- search "access control"
cargo run --bin mizan -- search "access control" --page-token TOKEN
cargo run --bin mizan -- search --semantic "supply chain evidence"
cargo run --bin mizan -- framework list
cargo run --bin mizan -- snapshot list
cargo run --bin mizan -- release list
cargo run --bin mizan -- claim list --trust-state verified
cargo run --bin mizan -- claim events CLAIM_ID
cargo run --bin mizan -- claim receipt CLAIM_ID
cargo run --bin mizan -- graph traverse NODE --max-depth 3
cargo run --bin mizan -- graph projection-events --claim-id CLAIM_ID
cargo run --bin mizan -- graph node list
cargo run --bin mizan -- completions zsh > ~/.zsh/completions/_oscal-cli
cargo run --bin mizan -- doctor
cargo run --bin mizan -- --capture search "access control"
cargo run --bin mizan -- tui
```

`validate`, `convert`, `resolve`, `diff`, `lint`, `inspect`, `completions`, `doctor`, `proto`, and all `capture` subcommands are local-only commands. They execute offline without dialing a server or requiring endpoint credentials.
command and does not validate endpoint, token, or TLS configuration, so it can
be used during installation before a server is available.

`doctor` is also local-only: it reports endpoint parsing, TLS file presence,
token-file readability, mTLS identity presence, capture-root state, and the
embedded protocol digests without dialing or writing. Its `network_checked`
field is always `false`; it also reports
`capability_negotiation=not_declared_by_protocol` because the current OSCALify
contract publishes no capability-negotiation message. Use `health` for an
explicit server check.

`proto` and all `capture` subcommands are local-only as well. They do not
validate or read endpoint credentials, so protocol inspection and capture
verification remain usable when the server is unavailable or the network
configuration is malformed.

The `proto` and `doctor` reports include the pinned OSCAL schema version,
official release commit and ZIP digest, and schema-manifest digest alongside
the protobuf descriptor and lock digests.

The command surface intentionally excludes create, update, delete, ingest,
upload, sync, publish, propose, resolve, claim verification, and graph-edge
projection calls. Claim verification is excluded because the OSCALify server
records a verification event and updates the claim trust projection; the
client exposes the resulting append-only event history and deterministic
receipt as read paths instead. Analysis calls such as evidence verification,
semantic search, traversal, trust, and closure are retained only where the
server implementation is observational.

Pagination tokens are exposed as `--page-token` and are never silently
discarded.

The TUI loads observed catalog, framework, claim, node, and edge pages at
startup in the background, so the terminal renders an explicit
`connection evaluating` state immediately even when the server is slow or
unavailable. `/` starts a server search, `f` applies a server-side filter on the
Explorer or Graph surface, `j`/`k` select and auto-scroll, `Enter` opens a
read-only detail response, `[`/`]` navigate observed pages, `r` refreshes the
observed pages, `e` opens verification events and `c` opens a deterministic
claim receipt while a claim detail is open, `?` opens the complete keymap,
`1`–`5` changes the surface, and `q` exits. From an edge detail, `p` opens the
observed projection-event timeline. Unavailable pages remain unavailable; no
totals or health claims are synthesized.

`r` always re-evaluates the read-only connection and clears the observed page
state before loading fresh responses, so a failed refresh cannot leave stale
server data presented as current. The edge projection timeline renders
chain-validity state, event sequence, relation, trust state, timestamp, and
compact hash lineage; full protobuf/JSON output remains available from the CLI.
`Ctrl-C` and `q` both exit through the terminal cleanup path.

## Output and captures

All protobuf responses support `--format table|json|jsonl|proto` where the
command supports a table view. `--capture` stores request and response protobuf
bytes plus a manifest under the platform-local application data directory.
`capture show`, `capture verify`, and `capture export` verify both protobuf
digests against the manifest before returning evidence; tampered captures fail
closed. `capture prune --older-than-days N` is a dry run unless `--confirm` is
provided, and it deletes only verified capture directories containing the
expected three files. This local housekeeping does not mutate OSCALify.

`json` is indented JSON, `jsonl` is one compact JSON value per line, and
`proto` writes the raw protobuf bytes to stdout. Collection responses use
`items` plus the protobuf-style `nextPageToken` field; table mode prints the
continuation token as `next_page_token` for shell readability.

Bearer credentials can be supplied with `--token`, `--token-file`, or
`OSCALIFY_TOKEN`; they are attached to requests and never written to capture
manifests. Prefer the environment or token file in shared shells because
command-line arguments can be visible to local process inspection. The token
file is trimmed and must contain a non-empty value; `--token` and
`--token-file` cannot be combined.

TLS uses `https://` endpoints with trusted roots by default. Private roots and
mTLS are available through `--ca-cert`, `--client-cert`, `--client-key`, and
`--tls-domain`; certificate verification cannot be disabled by this client.

## Protocol provenance

The vendored protocol snapshot is build-locked and can be materialized as a
deterministic provenance document:

```bash
./scripts/proto-provenance.sh --output target/proto-provenance.json
```

Release automation may add a cosign Sigstore bundle with
`--signing-key KEY --bundle BUNDLE`, then verify it with
`--verify-key PUBLIC_KEY --verify-bundle BUNDLE`. No signing key is stored in
this checkout.

To create a deterministic installable beta archive with checksums and an
adjacent release-provenance manifest:

```bash
./scripts/release-bundle.sh --output target/oscal-cli-beta.tar.gz
```

The archive contains the release binary, operator documentation (including the
current protocol alignment brief), protocol provenance, generated shell
completions, `bundle-manifest.json`, and `SHA256SUMS`. The adjacent
`*.release-provenance.json` is intentionally marked unsigned until an approved
release signing identity signs it with cosign.

The authenticated transport smoke creates an ephemeral CA, server identity,
and client identity in a temporary directory, then verifies CA validation,
mTLS rejection without a client certificate, and the TUI over mTLS:

```bash
./scripts/tls-smoke.sh
```

Use `OSCALIFY_REGENERATE_PROTO=1` with the TLS smoke as well when the supplied
OSCALify checkout has external generated-SDK drift.

## Build checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```

To run the complete local beta gate and write a machine-readable verdict with
per-gate evidence files:

```bash
./scripts/beta-readiness.sh
```

The manifest is written to `target/beta-readiness.json`; its evidence directory
contains each command, stdout, stderr, and exit code. A passing local verdict
still reports protocol alignment, release signing identity, upstream
authorization, and organization-specific capture-retention policy as external
gates when those inputs are not accepted by the source project or deployment.
The readiness evidence also retains the machine-readable protocol audit and the
explicit override-build result separately. Advisory databases use
`target/beta-policy-cargo-home` by default; set `OSCAL_BETA_CARGO_HOME` to an
approved writable cache when a different location is required.

To package that evidence for human and upstream review as one deterministic
archive:

```bash
./scripts/beta-handoff.sh
```

This writes `target/oscal-cli-beta-handoff.tar.gz` with its checksum and
unsigned provenance sidecar. It includes the readiness manifest, gate
transcripts, protocol diff, coverage summaries, source-custody snapshots, beta
plan, release artifacts, and retained tmux TUI panes under
`readiness-evidence/tui/`. The script refuses failed local readiness and
verifies the archive before returning.

To evaluate the external acceptance record without changing it:

```bash
./scripts/beta-acceptance-check.sh \
  --output target/beta-readiness-evidence/beta-acceptance-check.json
```

This check exits non-zero while any of the four human dispositions is pending,
deferred, rejected, or missing owner/date/evidence fields. It also requires the
local readiness verdict to be green. A pending result is expected until the
external owners complete `BETA_ACCEPTANCE.md`; the checker never changes the
OSCALify checkout or treats local evidence as upstream authorization.

## Live smoke check

The client repository includes a repeatable read-only smoke check. It builds or
uses an OSCALify server binary, runs it against an isolated in-memory database,
exercises representative CLI reads, and inspects the TUI through tmux. The
server checkout is read-only input to the test and is never configured with a
persistent database.

```bash
./scripts/e2e-smoke.sh
```

The default smoke build uses the supplied OSCALify checkout as-is. If that
checkout has external generated-SDK drift, use an explicitly disposable
regenerated copy without modifying the source checkout:

```bash
OSCALIFY_REGENERATE_PROTO=1 ./scripts/e2e-smoke.sh
```

The harness attempts `buf generate` in that disposable copy. If the remote Buf
plugin service is unavailable, it reports the failure and continues only with
the copied checked-in generated files; a server build failure still fails the
smoke. Set `OSCALIFY_E2E_FIXTURES=1` to exercise successful detail and graph
projection flows against a seeded copy of the read-only OSCALify database; the
beta readiness harness enables this mode in its disposable server run. Exact
upstream generated-source consistency remains a separate acceptance gate.

To measure the same live CLI/TUI matrix with LLVM instrumentation:

```bash
./scripts/coverage-smoke.sh
```

The coverage run uses the opt-in fixture database, rejected-input cases,
capture-integrity failures, disconnected TUI startup, and ledger/vault/high-
contrast theme sessions. It writes the measured summary to
`target/coverage-smoke/summary.txt` and fails if repository region or line
coverage is below 80% (override only for diagnosis with
`OSCAL_COVERAGE_MIN_REGIONS` and `OSCAL_COVERAGE_MIN_LINES`).

Set `OSCALIFY_ROOT`, `OSCAL_CLI_BIN`, or `OSCALIFY_SERVER_BIN` to use explicit
checkout and binary paths. `OSCALIFY_E2E_PORT` changes the isolated test port.
For successful detail-path coverage, set `OSCALIFY_E2E_FIXTURES=1`; the script
copies `oscal.db` into its temporary directory and seeds only that copy. The
source OSCALify database is never opened for writes.

The `proto` command exposes the read-only source/service list, descriptor byte
count, descriptor SHA-256, vendored `proto.lock` SHA-256, and snapshot file
count so a release candidate can identify the exact protocol input it carries.

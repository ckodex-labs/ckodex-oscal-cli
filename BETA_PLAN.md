# OSCAL Observer Client — Beta Plan

Status labels follow the project contract: `C` is implemented and verified in
this checkout, `S` is specified with a partial implementation or an external
gate, and `A` is intentionally deferred from beta.

## 1. Boundary and beta outcome

The product is a read-only CLI/TUI observer for OSCALify protobuf services. The
client may list, get, search, verify evidence and graph closures, inspect
verification events and claim receipts, traverse, compute, explain, inspect
local captures, and render evidence. It does not create, update, delete, ingest,
upload, sync, publish, propose, resolve, generate, or project graph state.

The OSCALify checkout is protocol input only. This client may read its proto
files and build an isolated server for smoke testing; it must not edit source,
generated code, configuration, databases, or artifacts there.

Beta is accepted only when the local implementation gates and the live
read-only smoke gate pass. Hosted deployment, real identity, production TLS,
external authorization policy, and upstream server acceptance remain separate
gates and cannot be inferred from local results.

Current boundary audit note: the latest read-only audit observed an external,
unattributed 75-file `proto/oscal` worktree diff, while all thirteen locked
client proto files now match the currently observed source. The client
explicitly refreshed only `services/v1/transparency_graph_service.proto` and
`proto.lock` from read-only source input; it did not modify or regenerate
anything in OSCALify. Broader unattributed changes outside `proto/oscal` are
preserved and treated as external state. The exact source revision, hashes, and
acceptance path are recorded in [`PROTOCOL_ALIGNMENT.md`](PROTOCOL_ALIGNMENT.md).

## 2. Experience assessment

### Verified strengths (`C`)

- The executable surface is read-only; generated mutation-capable modules are
  private to the crate and the transport wrappers are classified by a
  read-only method marker. Claim verification is deliberately absent because
  the current server persists its result; append-only event history and
  receipt export remain available as read paths.
- CLI commands expose explicit pagination tokens and support table, JSON,
  JSONL, and protobuf output where applicable.
- Generated command help labels the operator intent of transport identity,
  output, capture, search-mode, filter, and pagination flags; the E2E gate
  asserts the high-risk search help contract so the scriptable surface is
  discoverable without reading implementation code.
- Standard search forwards opaque `--page-token` values; semantic-only and
  standard-only search flags are rejected instead of being silently ignored.
- TUI pagination retains each page's continuation token when operators move
  backward from a terminal page, and `Ctrl-C` exits through the same terminal
  cleanup path as `q`.
- TUI startup renders before network bootstrap completes, preserving visible
  connection state and a reliable quit path during slow or unavailable-server
  conditions.
- The command grammar generates Bash, Elvish, Fish, PowerShell, and Zsh
  completions locally; completion generation bypasses endpoint and credential
  validation and performs no network or capture operation.
- The offline `doctor` command reports local endpoint, TLS, credential-file,
  mTLS identity, capture-root, and protocol-snapshot readiness without writing
  or dialing; it explicitly marks `network_checked=false`.
- TUI surfaces Overview, Explorer, Search, Graph, and Evidence; it supports
  selection, detail inspection, paging, filtering, refresh, search, a complete
  keymap, and terminal cleanup on exit.
- Captures write request bytes, response bytes, and a manifest with SHA-256
  digests. Writes use a pending directory and atomic rename; capture roots and
  files are permission-restricted on Unix.
- Capture reads fail closed for digest mismatches, byte-count mismatches,
  malformed IDs, symlinked capture directories, and symlinked capture files.
- Endpoint userinfo is removed from errors, manifests, TLS-domain inference,
  and the TUI evidence margin. Bearer tokens are not written to captures.
- The live smoke script starts an isolated in-memory OSCALify server, runs
  representative CLI reads, inspects server logs for mutation RPCs, and
  renders the TUI and help overlay through tmux.
- When the external OSCALify checkout has generated-SDK drift, the readiness
  harness copies it to a temporary directory, runs `buf generate` there, and
  builds the isolated server from that copy; if the remote Buf plugin service is
  unavailable, it reports the failure and uses only the copied checked-in
  generated files. The original checkout remains unchanged and exact upstream
  generated-source consistency stays visible as an external gate.
- The coverage smoke can opt into a seeded fixture mode that copies the
  read-only OSCALify database into a temporary directory and seeds only that
  copy, allowing successful detail and graph paths without writing to the
  source checkout. The beta readiness E2E enables the same mode so retained
  tmux evidence includes claim and projection-event detail flows.
- The E2E harness also covers rejected configuration, connection failure,
  capture tampering, disconnected TUI startup/reconnect, offline doctor
  diagnostics, shell completions, and ledger/vault/high-contrast TUI themes.
  The readiness run retains selected tmux panes and a SHA-256 manifest under
  `target/beta-readiness-evidence/tui/`, so visual interaction evidence survives
  temporary test cleanup and is included in the beta handoff.
  The latest live result clears the enforced 80% regions and 80% lines
  thresholds; exact percentages are recorded in
  `target/coverage-smoke/summary.txt`. The `proto` evidence command reports SHA-256 fingerprints for the
  descriptor set and vendored `proto.lock`, plus the 13-file snapshot count.
- A local TLS/mTLS smoke now creates ephemeral identities, verifies the client
  certificate and CA chain, rejects a connection without the client
  certificate, and renders the TUI over authenticated TLS.
- Protocol provenance is reproducible from the `proto` evidence command and
  can be signed and verified with a cosign bundle without storing a signing
  key in the checkout.
- `proto --format proto` emits the embedded descriptor set bytes; the E2E gate
  verifies those bytes against the reported descriptor SHA-256.
- Projection-event history is a typed read path: the CLI requires a claim or
  edge filter, preserves `chain_valid` and event hashes across
  table/JSON/JSONL/protobuf output, captures the response, and the TUI opens a
  structured timeline from edge detail with chain status, sequence, relation,
  trust, timestamp, and compact hash lineage.
- A server that predates the projection-events RPC is reported as an explicit
  unavailable read path with protocol-revision guidance; the client does not
  emulate or silently fall back to another contract.
- `scripts/release-bundle.sh` creates a deterministic, installable archive
  containing the read-only binary, operator docs, protocol provenance,
  generated shell completions, checksums, and a release-provenance manifest.
  The local artifact is explicit about remaining unsigned status.
- `scripts/beta-readiness.sh` produces a machine-readable local verdict with
  per-gate command, stdout, stderr, and exit-code evidence. Its verdict keeps
  source ownership and protocol acceptance, release signing, upstream authorization, and
  capture-retention approval in the external review lane.
- `scripts/protocol-audit.sh` compares every locked proto file with an external
  OSCALify source tree, verifies the projection message, edge field, RPC
  binding, and HTTP gateway route in checked-in generated Go bindings, records
  source diff/status counts, retains exact unified diffs for drifted files, and
  exits non-zero on drift without modifying the source. Readiness retains this
  report separately from the explicit override compile, so source drift is
  visible rather than hidden behind a regenerated disposable smoke copy.
- The latest readiness manifest is `target/beta-readiness.json`: all 14 local
  gates passed, protocol audit and override compilation passed, and four
  external human gates remain separate. The verdict is
  `local_beta_candidate_requires_human_review` because signing identity,
  upstream authorization, and capture-retention policy are not held here.
- `BETA_ACCEPTANCE.md` turns the four external gates into an owner-disposition
  record with required evidence, while preserving the read-only source boundary.
- Capture integrity can be audited with `capture verify`; retention is bounded
  by an explicit, verified-only `capture prune` dry run with `--confirm`
  required for local deletion.
- CKODEX-DS-3 terminal rules are applied to the evidence margin, digest
  presentation, claim vocabulary, square geometry, and neutral/redacted
  states.

### Remaining beta gaps (`S`)

1. **Protocol packaging:** the default build now compiles the reviewed proto
   snapshot vendored under `proto/oscal` and validates `proto.lock`. The release
   bundle carries descriptor and lock digests; an explicit
   `OSCALIFY_PROTO_ROOT` override remains available for read-only protocol
   review and intentional refreshes. Signed release provenance still requires
   the external signing gate.
2. **External identity acceptance:** token, custom CA, and mTLS paths have
   configuration and transport wiring; the local mTLS harness is green, but
   upstream identity and authorization policy still require an approved
   environment.
3. **Operational discovery:** the client now probes the standard gRPC health
   service and reports observed `SERVING`/non-serving status. The offline
   `doctor` and `proto` evidence surfaces explicitly report
   `capability_negotiation=not_declared_by_protocol`; OSCALify exposes no
   declared capability contract, so the client does not synthesize one.
4. **Capture lifecycle:** local integrity verification and explicit retention
   housekeeping are implemented. Organization-specific retention duration,
   redaction policy configuration, and signed evidence envelopes remain
   deployment-policy inputs.
5. **Dependency graph hygiene:** cargo-deny reports only non-blocking duplicate
   transitive versions (`foldhash`, `getrandom`, `hashbrown`, `syn`, and
   `windows-sys`) after the tonic migration. There are no advisory, license,
   ban, or source failures; deduplication is backlog optimization, not a
   release security exception.
6. **Upstream generated-source ownership:** the current audit proves that the
   observed source proto and checked-in Go message, RPC, and gateway bindings
   are aligned. The source revision is still dirty and source-project
   regeneration/ownership remains external; future mismatches fail the audit.
7. **Projection-event beta acceptance:** the typed CLI/TUI capability is
   implemented and locally verified against the current source snapshot. The
   remaining gate is external: an owner must accept the dirty source revision,
   compatibility with older servers, event-retention/redaction policy, and the
   release snapshot. The audit records those source-custody facts in
   [`PROTOCOL_ALIGNMENT.md`](PROTOCOL_ALIGNMENT.md).

### Deliberately deferred (`A`)

- Any write command or workflow that changes OSCALify state.
- Background subscriptions, live event streaming, or speculative server
  health dashboards without a protocol contract.
- Cross-platform terminal screenshot approval automation beyond the tmux smoke
  path.
- Automatic protocol negotiation, server-side feature emulation, and silent
  fallback to a different proto snapshot.
- Client-triggered claim verification. The current OSCALify implementation
  updates the claim trust projection and appends a verification event, so the
  observer exposes only `claim events` and `claim receipt` reads.

## 3. Implementation sequence

### Milestone 0 — contract and source custody (`C`)

Acceptance:

- Client modules and generated bindings are private except for the executable
  runner.
- `proto.lock` is checked during build.
- Boundary audits record any upstream dirty state and prove that this client
  does not write or overwrite it.

Evidence: `src/lib.rs`, `src/policy.rs`, `build.rs`, `proto.lock`, the current
upstream diff audit, and the read-only boundary audit command in the final
verification transcript.

### Milestone 1 — transport and local security (`C`)

Acceptance:

- Invalid configuration fails before dialing.
- Connection errors identify a redacted endpoint and actionable recovery hint.
- TLS verification remains enabled; custom roots and mTLS require complete
  inputs.
- Tokens can come from an environment variable or trimmed token file, cannot
  be supplied twice, and empty values fail.
- Capture paths cannot escape the configured root and capture files cannot be
  followed through symlinks.

Evidence: `src/config.rs`, `src/error.rs`, `src/transport.rs`,
`src/capture.rs`, and their tests.

### Milestone 2 — CLI contract (`C`)

Acceptance:

- Every exposed command maps to a read-only RPC or local capture/proto
  inspection.
- Collection output uses `items` and `nextPageToken` in JSON modes; raw
  protobuf mode writes bytes only to stdout.
- Pagination inputs are explicit and invalid page sizes are rejected.
- Capture failures fail the CLI command rather than being silently ignored.
- `--help` and `--version` describe the supported surface.
- Command help explains positional identifiers, filters, pagination tokens,
  search mode restrictions, transport identity inputs, output formats, and
  local capture behavior.
- `completions SHELL` emits non-empty, grammar-derived completion scripts for
  every supported shell and rejects unknown shells without dialing.
- `doctor` emits table or JSON diagnostics, reports malformed local inputs as
  `attention`, and never claims server availability.
- `proto` and `capture` commands remain usable without endpoint or credential
  validation because they inspect only embedded protocol data or local
  evidence.
- `graph projection-events` requires `--claim-id` or `--edge-id`, preserves
  `chain_valid` and event hashes in every output mode, and captures the typed
  response.

Evidence: `src/cli.rs`, `src/output.rs`, `src/config.rs`, README examples,
and CLI help/proto command output.

### Milestone 3 — TUI operator loop (`C`)

Acceptance:

- Startup renders observed pages without invented totals or health claims.
- Search/filter/paging/detail actions remain read-only.
- `?` opens a complete keymap; `Esc` closes modal/detail state; `q` exits.
- Raw mode and the alternate screen are restored on normal and setup-failure
  paths.
- `r` re-evaluates the read-only connection and clears observed pages before a
  refresh, preventing stale data from being presented after transport failure.
- Evidence margin labels are operational receipts, not unsupported
  attestation claims; endpoint and snapshot digests are safe to display.
- Graph edge detail exposes `p` in the active footer to open a structured
  projection-event timeline with observed sequence, relation, trust, chain
  status, timestamp, and compact event/previous-hash lineage. The tmux E2E
  waits for the asynchronous detail render before asserting these fields.

Evidence: `src/tui.rs`, render tests, and `scripts/e2e-smoke.sh`.

### Milestone 4 — beta acceptance hardening (`S`)

Acceptance to promote the beta package:

- Preserve the measured coverage artifact at or above the 80% repository
  threshold on every beta release candidate.
- Keep the tonic/prost stack on the current supported line and preserve the
  zero-vulnerability, zero-audit-warning gate.
- Attach a signed release provenance bundle to distribution artifacts when a
  release signing identity is available.
- Run authenticated TLS/mTLS smoke against an approved upstream environment.
- Preserve the OSCALify boundary audit after every protocol refresh.
- Run `scripts/protocol-audit.sh` against the exact upstream checkout and retain
  its JSON report with the beta evidence; a non-zero result requires upstream
  protocol/generated-source review.
- Regenerate `target/beta-readiness.json` and retain its evidence directory
  with each beta release candidate; a green local verdict is necessary but not
  sufficient for external beta acceptance.
- Run `scripts/beta-handoff.sh` and retain its deterministic review archive,
  checksum, and unsigned provenance sidecar with the beta candidate.
- Run `scripts/beta-acceptance-check.sh` against the acceptance record and
  retain its JSON result. Promotion is fail-closed until local readiness is
  green and all four human dispositions are `ACCEPT` with owner, date, and
  evidence reference fields supplied.
- Include the deterministic release archive, checksum, and release-provenance
  manifest from the readiness evidence directory; attach the approved cosign
  bundle only after the signing identity gate passes.

## 4. Verification commands

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo nextest run --all-features
cargo deny check advisories licenses bans sources
cargo audit --json
cargo build --release
OSCALIFY_REGENERATE_PROTO=1 ./scripts/e2e-smoke.sh
OSCALIFY_REGENERATE_PROTO=1 ./scripts/tls-smoke.sh
./scripts/release-bundle.sh --output target/oscal-cli-beta.tar.gz
./scripts/beta-readiness.sh
./scripts/beta-handoff.sh
./scripts/beta-acceptance-check.sh \
  --output target/beta-readiness-evidence/beta-acceptance-check.json
```

Coverage is measured separately so the report is not confused with the live
smoke transcript:

```bash
cargo llvm-cov --all-features --workspace --summary-only
./scripts/coverage-smoke.sh
```

The coverage script fails below 80% repository regions or 80% repository lines;
the threshold is enforced, not merely reported. The complete local beta gate
is:

```bash
./scripts/beta-readiness.sh
```

It writes `target/beta-readiness.json` and
`target/beta-readiness-evidence/`. A green local verdict is a beta candidate,
not external acceptance. The readiness harness keeps cargo advisory databases
in `target/beta-policy-cargo-home` by default (override with
`OSCAL_BETA_CARGO_HOME`) so policy gates do not depend on a writable global
developer home.

## 5. Human review checkpoint

Human review is required before beta promotion for the four external gates:
release signing identity/provenance, upstream authorization policy,
organization-specific capture evidence-retention policy, and upstream source
ownership/compatibility acceptance. The protocol gate includes both the
per-file `protocol-audit.sh` report and the explicit override compile; both are
local evidence, not upstream approval. None of these gates authorizes changes
to the OSCALify project from this checkout.

The fail-closed acceptance checker is the final promotion guard. It records
which dispositions are still pending and exits non-zero until all four are
explicitly accepted with owner, date, and evidence reference fields.

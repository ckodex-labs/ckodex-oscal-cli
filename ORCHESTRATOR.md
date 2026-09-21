# Orchestrator — OSCALify Observer Client v1

## Invariant kernel

- The orchestrator decides; implementation units execute; evidence records. A
  unit does not both decide and verify its own completion.
- The client is an observer. It may read, search, verify observational
  evidence/closures, traverse, capture,
  and render. It must not create, update, delete, ingest, upload, sync,
  publish, propose, resolve, generate, or project graph state.
- The vendored `proto/oscal` tree is the default immutable protocol snapshot.
  The neighboring OSCALify checkout is an optional read-only source for explicit
  protocol review or refresh. No source, generated file, database,
  configuration, or build artifact in that checkout is writable from this
  project.
- Live smoke may copy the source checkout to a disposable temporary directory
  and regenerate SDKs there when explicitly requested. If the remote Buf plugin
  service is unavailable, the smoke reports that condition and builds only from
  the copied checked-in generated files. This never repairs or rewrites the
  original checkout, and exact upstream generated-source consistency remains a
  separate acceptance gate.
- Ambiguity is resolved with 2–4 bounded options. The safe default is the
  smallest read-only slice that preserves protocol evidence.
- A completion claim requires verifier output and file:line evidence. An
  unverified claim is explicitly classified as unverified.
- Executor disagreement is recorded with the competing evidence. Unresolved
  high-stakes disagreement stops at human review.

## Domain bindings

DOMAIN: `coding`
ROLES -> MODELS: resolved at dispatch; never hardcoded

- mechanical: closed transformations, formatting, generated checks
- builder: Rust implementation to the accepted contract
- reasoner: protocol boundaries, security, UX, and failure analysis
- cross-peer: independent gap audit and review when a delegated executor is
  available

ARTIFACT_OF_RECORD: unstaged source diff plus generated `Cargo.lock`, protocol
snapshot lock, test output, live smoke transcript, and release binary hash.
The beta review handoff archive is an evidence projection of those artifacts;
it does not replace the source checkout or external acceptance.
The isolated graph projection-event delta and its acceptance criteria are
recorded in `PROTOCOL_ALIGNMENT.md`.

VERIFIER: `cargo fmt`, strict Clippy, Rust tests, proto snapshot verification,
read-only conformance tests, live read-only smoke, and boundary audit.

EVIDENCE_FORM: tests and gates, live RPC transcripts, capture manifests,
proto snapshot hashes, and conformance vector IDs.

MERGE: one unstaged checkout under `ckodex-oscal-cli`; no worktree merge and no
mutation of OSCALify.

## Product contract

### Read-only surface

The client exposes all safe OSCALify reads and analyses:

- OSCAL model list/get and search.
- Governance entity, framework, snapshot, release, and semantic-search reads.
- Transparency claim/evidence reads, observational evidence verification,
  verification-event history, and deterministic claim receipt reads. Claim
  verification is excluded because the server persists its result.
- Transparency graph node/edge reads and graph analysis.
- Standard gRPC health observation for server liveness/readiness.
- Local capture list/show/verify/export/prune and protocol descriptor
  inspection. Prune is verified-only and dry-run by default.

Every RPC wrapper carries a typed read-only method marker. Mutating RPCs are
absent from the client surface and remain explicit policy-denial test cases.

### Evidence contract

- Captures contain request protobuf, response protobuf, manifest, method,
  endpoint, timestamps, byte counts, and SHA-256 digests.
- Capture reads verify the request and response digests before evidence is
  shown or exported; mismatch is a fail-closed integrity error.
- Credentials never enter capture bytes or manifests.
- Capture IDs are filesystem-safe, collision-resistant, and cannot escape the
  configured capture root.
- Digest rendering uses the `sha256:` prefix and a middle ellipsis; bare hex is
  not presented as an attestation.
- The CLI and TUI distinguish observed, inferred, claimed, attested,
  contradicted, and quarantined states using the CNDL safe-set.
- TUI pagination, filters, selection, scrolling, and detail views dispatch only
  read/list/get RPCs; edge detail exposes a structured projection-event
  timeline without creating or mutating OSCALify state.

### Design and DevEx contract

- Apply CKODEX-DS-3 Evidence Editorial: ledger, vault, high-contrast, and
  forced-colors-safe terminal mappings; square geometry; evidence margin;
  violet only for proof; red only for active quarantine/emergency.
- No invented counts, confidence, health, or provenance. Server-returned values
  are labeled observed and missing values remain unavailable.
- CLI output is scriptable and stable across table, JSON, JSONL, and protobuf
  modes. Errors identify the failed boundary and preserve the read-only posture.
- Shell completions are generated directly from the clap command grammar for
  Bash, Elvish, Fish, PowerShell, and Zsh; completion generation is local-only.
- The local `doctor` command reports configuration and protocol readiness with
  an explicit `network_checked=false`; it never substitutes local checks for
  server health.
- `proto` and `capture` are local-only dispatch paths and bypass endpoint,
  credential, and transport validation; local evidence remains inspectable when
  the server configuration is unavailable.
- `scripts/protocol-audit.sh` is the read-only external protocol verifier: it
  compares the complete `proto.lock` file set, records source worktree counts,
  retains exact unified diffs for drifted files, and emits a non-zero result on
  drift. Readiness also keeps its result separate from the disposable
  regenerated-copy smoke path.
- `scripts/protocol-sync.sh` is the explicit refresh boundary. It verifies the
  OSCAL release schema manifest, stages the 13-file allowlist, refreshes
  `proto.lock`, and never writes to the source checkout. Dirty source input
  requires an explicit operator flag and is recorded in the lock.
- `scripts/beta-handoff.sh` packages the current readiness evidence,
  protocol-diff record, coverage summaries, source-custody snapshots, and
  release artifacts into a deterministic, self-verified review archive.
- `PROTOCOL_ALIGNMENT.md` records the exact graph-proto hashes, the read-only
  refresh decision, the implemented projection-event CLI/TUI capability, and
  the remaining external ownership/compatibility gate; it does not authorize
  OSCALify mutation.
- `BETA_ACCEPTANCE.md` is the human-review record for source ownership,
  signing, authorization, and capture-retention dispositions; it cannot grant
  authority or authorize OSCALify mutation.
- Help, README, examples, prompt, tests, and implementation use the same
  command grammar.

## Dispatch protocol

1. Decompose work into units with acceptance criteria and pinned file:line
   context.
2. For non-trivial units, send the raw problem to an independent cross-peer
   when a delegated executor is available; compare plans before implementation.
3. Mechanical units receive closed briefs. Reasoning units receive constraints
   and evidence, not a preselected implementation.
4. Accept only through the verifier and file:line citations.
5. Retry once with the failure trace. A second failure promotes the unit to a
   deeper reasoning pass; retries are never silent.
6. Stop at the human review checkpoint when the remaining gate requires
   credentials, production authority, external acceptance, or a design choice
   not encoded in this contract.

## Current execution record

- Direct local implementation is a logged exception because no delegated
  executor surface is available in this workspace.
- Scope is limited to `/Users/mchorfa/Documents/projects/runbase/ckodex-oscal-cli`.
- The vendored snapshot and any `OSCALIFY_PROTO_ROOT` override are immutable
  protocol inputs. The current graph snapshot was intentionally refreshed from
  read-only source input; future proto drift fails the client build and audit
  until another explicit snapshot review updates the vendored files and
  `proto.lock`.
- The detailed beta assessment, milestone acceptance criteria, verification
  commands, and human review checkpoint are recorded in `BETA_PLAN.md`.

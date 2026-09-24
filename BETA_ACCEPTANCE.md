# Beta acceptance record

Status: `S` — local candidate verified; external owner decisions are required.
This record is a review instrument, not an approval, signature, authorization,
or instruction to modify OSCALify.

## Candidate identity

The machine-readable files under `target/` are authoritative for the latest
run. The observed candidate facts are:

| Field | Observed value |
| --- | --- |
| Client | `oscal-cli` `0.1.0` |
| Client root | `.` (repository root) |
| OSCALify source HEAD | `6f6b641133201cbb777740cd3b7e1c9ca508aa3a` |
| Last committed graph-proto revision | `f88047e10ea39988a02d33f7114fd01525b48c1f` |
| Graph proto SHA-256 | `4fac7cf7925359c9d3ae63715b62eba8c316eea6dc291e7006fee16fa32b51c1` |
| Protocol files | 13 locked, 13 matching, 0 drifted |
| Source worktree | dirty; graph proto is an uncommitted modification |
| Source mutation | none observed by before/after custody snapshots |

If any source fact changes, regenerate the readiness manifest and this review
record before disposition. Do not edit the source checkout to make the facts
match this document.

## Local evidence already supplied

- `target/beta-readiness.json`: 14/14 local gates passed; protocol audit and
  override compilation passed; four human gates remain pending.
- `target/beta-readiness-evidence/protocol-audit.json`: exact file hashes,
  source revision, and generated message/RPC/gateway probes.
- `target/coverage-smoke/summary.txt`: aggregate coverage and the retained
  below-threshold function diagnostic.
- `target/oscal-cli-beta-handoff.tar.gz`: deterministic review archive with
  checksum and unsigned provenance sidecar.
- `target/beta-readiness-evidence/beta-acceptance-check.json`: fail-closed
  machine check of this record; it remains `pending_human_review` while the
  four dispositions below are blank.
- The E2E evidence includes fixture projection-event reads, capture verification,
  tmux edge-detail-to-timeline navigation, retained pane transcripts, and
  disconnected paths.

## Required external dispositions

Each gate needs an owner, a decision, a timestamp, and an evidence reference.
`ACCEPT` is not valid without the listed evidence.

### 1. Source ownership and compatibility

Required evidence:

- accepted upstream commit, tag, or immutable source digest;
- confirmation that the proto and generated Go message, RPC, and gateway
  bindings come from that same source revision;
- compatibility decision for servers that predate
  `TransparencyGraphService.ListProjectionEvents`;
- decision to use the current client snapshot and `proto.lock`.

Disposition: `[ ] ACCEPT  [ ] DEFER  [ ] REJECT`

Owner: ____________________  Date: ____________________

Evidence reference: _________________________________________________

### 2. Release signing and provenance

Required evidence:

- approved signing identity and verification policy;
- signed release provenance for the exact handoff/archive digest;
- verification output and bundle reference;
- confirmation that no private signing key is stored in this checkout.

Disposition: `[ ] ACCEPT  [ ] DEFER  [ ] REJECT`

Owner: ____________________  Date: ____________________

Evidence reference: _________________________________________________

### 3. Upstream authorization

Required evidence:

- approved endpoint/environment;
- token scope and audience decision, or approved mTLS identity and CA chain;
- successful authenticated read-only smoke against that environment;
- confirmation that write RPCs remain unavailable from this client surface.

Disposition: `[ ] ACCEPT  [ ] DEFER  [ ] REJECT`

Owner: ____________________  Date: ____________________

Evidence reference: _________________________________________________

### 4. Capture retention and redaction

Required evidence:

- retention duration and deletion authority;
- data classification, redaction, and legal-hold rules;
- approval that request/response protobuf bytes and graph hash-chain fields may
  be captured under those rules;
- evidence of verified-only pruning and handling of failed integrity checks.

Disposition: `[ ] ACCEPT  [ ] DEFER  [ ] REJECT`

Owner: ____________________  Date: ____________________

Evidence reference: _________________________________________________

## Promotion rule

The candidate remains `local_beta_candidate_requires_human_review` until all
four dispositions are `ACCEPT`, each has a non-placeholder owner, date, and
evidence reference, and the local readiness verdict is green. Run
`scripts/beta-acceptance-check.sh` to evaluate this rule. A `DEFER` keeps the
feature local-only; a `REJECT` requires recording the remediation or removing
the capability from the beta release notes. The checker does not fabricate or
independently validate external owner evidence; the named owner remains
responsible for the referenced artifact and decision.

No disposition in this record authorizes changes to the OSCALify checkout. The
client boundary remains read-only, and future protocol drift must fail closed.

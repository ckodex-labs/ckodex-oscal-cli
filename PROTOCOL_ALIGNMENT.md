# Protocol alignment brief

Status: client capability implemented against the currently observed source;
external source ownership and beta promotion acceptance remain pending. This
document records the refresh decision and does not change the OSCALify
checkout.

## Scope and custody

The reviewed client contract is the vendored `proto/oscal` tree plus
`proto.lock`. The external OSCALify checkout is read-only protocol input.
`scripts/protocol-audit.sh` compares the two trees and writes a machine-readable
report without editing either source tree.

`scripts/protocol-sync.sh` is the intentional refresh boundary. It validates
the official OSCAL schema manifest in OSCALify, records that schema provenance
in `proto.lock`, stages all allowlisted protobufs, and refuses dirty source
input unless the operator explicitly accepts a workspace snapshot.

The latest audit recorded:

| Item | Reviewed client | Observed external source |
| --- | --- | --- |
| Locked graph proto | `sha256:4fac7cf7925359c9d3ae63715b62eba8c316eea6dc291e7006fee16fa32b51c1` | `sha256:4fac7cf7925359c9d3ae63715b62eba8c316eea6dc291e7006fee16fa32b51c1` |
| Locked files | 13 | 13 |
| Matching files | 13 | 13 |
| Drifted files | 0 | 0 |
| OSCAL schema | `1.2.3` | `1.2.3` |
| Official schema manifest | pinned SHA-256 | verified SHA-256 |
| Generated-source probe | no client-side drift | message, edge field, RPC binding, and HTTP gateway route all present |

Evidence: `target/beta-readiness-evidence/protocol-audit.json` and
`target/beta-readiness-evidence/protocol-override.stderr`.

The same audit records the source provenance needed for acceptance: source
`HEAD` `6f6b641133201cbb777740cd3b7e1c9ca508aa3a`, a dirty source worktree, and
last committed graph-proto revision
`f88047e10ea39988a02d33f7114fd01525b48c1f`; the source worktree currently
contains unrelated untracked site tooling. These values are observations of the supplied
checkout, not an upstream approval.

## Observed delta

The external graph proto adds the following contract surface:

1. `TransparencyGraphService.ListProjectionEvents`, exposed as
   `GET /v1/graph/projection-events`.
2. `ProjectEdgeResponse.projection_event`, populated with the append-only
   event emitted by a successful projection.
3. `GraphProjectionEvent`, with sequence, event, edge, claim, node, relation,
   evidence, trust, previous-hash, event-hash, and projection-time fields.
4. `ListProjectionEventsRequest`, filtered by `claim_id` and `edge_id`.
5. `ListProjectionEventsResponse`, containing events and `chain_valid`.

The OSCAL `1.2.3` reconciliation additionally updates the client-consumed model
schemas:

1. Assessment Plan and Assessment Results control selections now carry the
   OSCAL `include-all` marker.
2. Mapping now exposes the released OSCAL 1.2 `mapping-collection` shape:
   collection provenance, control mappings, back matter, qualifiers, coverage,
   confidence, gap selection, and pattern matching.
3. The Mapping prototype fields remain on their original wire numbers as
   deprecated compatibility fields; table output prefers released mappings and
   falls back to legacy maps for older servers.
4. POA&M and Assessment Results comments no longer claim a hard-coded OSCAL
   `1.1.2` pin.
5. Transparency Exchange now exposes bounded external-evidence retrieval and
   read-only fetch-event history (`FetchExternalEvidence` and `ListFetchEvents`),
   including digest, size, outcome, timing, and policy detail in
   `EvidenceFetchAudit`.

This was a product-relevant read gap. The client snapshot was explicitly
refreshed from the read-only source input, and the complete typed capability is
now implemented locally. The source revision is still dirty and unowned from
this client's perspective, so this is not an upstream compatibility or release
approval claim.

## Current client impact

The client now carries all of the following for the observed contract:

- generated Rust types for `GraphProjectionEvent`;
- a read-only transport method and policy entry for `ListProjectionEvents`;
- a `graph projection-events` CLI command with claim/edge filters and
  table/JSON/JSONL/protobuf output;
- capture support through the existing evidence manifest;
- a TUI projection-event/evidence timeline opened with `p` from edge detail.

The current refresh adds the four OSCAL 1.2.3 model-proto deltas and schema
provenance to the previously reviewed graph contract. No route was hand-written,
no dynamic fallback was added, and the OSCALify checkout was not edited. Future
source or official-schema drift still fails closed through the lock and audit
checks.

## Bounded decision options

| Option | Decision | Trade-off |
| --- | --- | --- |
| A | Hold the refreshed snapshot | Preserves the current implementation and leaves promotion to external review. |
| B | Accept the current source revision | Confirms ownership, compatibility, generated artifacts, and release use of the implemented capability. |
| C | Add a raw or dynamic compatibility path | Rejected for beta: bypasses generated-contract and read-policy review, weakens provenance, and creates ambiguous behavior across server revisions. |

The safe beta posture is the implemented snapshot held for human review until
an OSCALify owner accepts the exact source revision and generated artifacts.

## Acceptance gate for option B

An owner may promote this capability from local implementation to beta use only
when all of the following are supplied:

- authoritative upstream commit, tag, or immutable source digest;
- confirmation that the proto change and generated Go bindings are from the
  same accepted revision;
- compatibility decision for servers that predate `ListProjectionEvents`;
- approved event-retention and redaction handling for hash-chain evidence;
- permission to update this client's vendored proto snapshot and lock.

The local implementation slice is complete and verified as follows:

1. The graph proto and `proto.lock` were refreshed together from a read-only
   source copy; the source checkout remained untouched.
2. The client has a read-only transport policy entry and typed wrapper for
   `ListProjectionEvents`.
3. `graph projection-events` requires a claim or edge filter, supports bounded
   output, and provides JSON/JSONL/table/protobuf rendering. `chain_valid` is
   preserved in every representation.
4. Capture support uses the existing schema version and redaction checks. A false
   `chain_valid` value must remain visible and must not be rendered as an
   attested proof state.
5. TUI navigation opens the projection-event timeline from edge detail and
   preserves chain validity and event hashes in the evidence response.
6. Fixture-server, capture, and tmux E2E coverage exercises successful reads,
   filter validation, and unavailable/disconnected paths.
7. Protocol audit, source-custody checks, all local beta gates, release bundle
   verification, and beta handoff generation are required before review.

## Exit criteria

This brief is resolved only when either:

- the external owner accepts the current source revision and the typed
  CLI/TUI capability ships with evidence from the complete gate sequence; or
- the external owner explicitly defers the feature and the beta release notes
  keep the capability listed as intentionally unavailable.

Until then, the local candidate remains eligible for human review only. The
OSCALify source remains unmodified.

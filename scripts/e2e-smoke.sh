#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
oscalify_root="${OSCALIFY_ROOT:-"$client_root/../ckodex-oscalify"}"
client_bin="${OSCAL_CLI_BIN:-}"
server_bin="${OSCALIFY_SERVER_BIN:-}"
regenerate_proto="${OSCALIFY_REGENERATE_PROTO:-0}"
port="${OSCALIFY_E2E_PORT:-50052}"
session="oscal-cli-e2e-$$"
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/oscal-cli-e2e.XXXXXX")"
server_pid=""
evidence_dir="${OSCALIFY_E2E_EVIDENCE_DIR:-}"

prepare_evidence() {
    if [[ -z "$evidence_dir" ]]; then
        return
    fi
    mkdir -p "$evidence_dir"
    rm -f "$evidence_dir"/tui-*.txt "$evidence_dir"/tui-manifest.json
}

persist_tui_evidence() {
    if [[ -z "$evidence_dir" ]]; then
        return
    fi
    mkdir -p "$evidence_dir"
    for pane in "$tmp_root"/tui-*.txt; do
        [[ -f "$pane" ]] || continue
        cp "$pane" "$evidence_dir/$(basename "$pane")"
    done
    python3 - "$evidence_dir" <<'PY'
import hashlib
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
files = []
for path in sorted(root.glob("tui-*.txt")):
    files.append(
        {
            "path": path.name,
            "size_bytes": path.stat().st_size,
            "sha256": "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest(),
        }
    )
(root / "tui-manifest.json").write_text(
    json.dumps(
        {
            "schema": "urn:oscalify:observer:tui-evidence:v1",
            "read_only": True,
            "renderer": "tmux capture-pane -J -p",
            "files": files,
        },
        indent=2,
        sort_keys=True,
    )
    + "\n"
)
PY
}

prepare_evidence

cleanup() {
    if [[ -n "$server_pid" ]]; then
        kill "$server_pid" 2>/dev/null || true
        wait "$server_pid" 2>/dev/null || true
    fi
    tmux kill-session -t "$session" 2>/dev/null || true
    persist_tui_evidence || true
    rm -rf "$tmp_root"
}
trap cleanup EXIT

if [[ ! -x "$client_bin" ]]; then
    (cd "$client_root" && cargo build --quiet)
    if resolved_bin="$("$client_root/scripts/cargo-binary-path.sh" debug)"; then
        client_bin="$resolved_bin"
    fi
fi
if [[ ! -x "$client_bin" ]]; then
    echo "client binary not found: ${client_bin:-unresolved}" >&2
    exit 1
fi

if [[ -z "$server_bin" ]]; then
    if ! command -v go >/dev/null 2>&1; then
        echo "go is required when OSCALIFY_SERVER_BIN is not supplied" >&2
        exit 1
    fi
    server_source_root="$oscalify_root"
    if [[ "$regenerate_proto" == "1" ]]; then
        if ! command -v buf >/dev/null 2>&1; then
            echo "buf is required when OSCALIFY_REGENERATE_PROTO=1" >&2
            exit 1
        fi
        server_source_root="$tmp_root/oscalify-regenerated"
        mkdir -p "$server_source_root"
        cp -R "$oscalify_root/." "$server_source_root/"
        if (cd "$server_source_root" && buf generate) >"$tmp_root/buf-generate.log" 2>&1; then
            printf '%s\n' 'disposable proto regeneration completed' >&2
        else
            printf '%s\n' 'disposable proto regeneration unavailable; using checked-in generated files in the disposable copy' >&2
            sed -n '1,120p' "$tmp_root/buf-generate.log" >&2 || true
        fi
    fi
    server_bin="$tmp_root/xoscal-server"
    (cd "$server_source_root" && go build -o "$server_bin" ./server/cmd/xoscal-server)
elif [[ ! -x "$server_bin" ]]; then
    echo "OSCALIFY_SERVER_BIN is not executable: $server_bin" >&2
    exit 1
fi

expect_failure() {
    local label="$1"
    shift
    if "$client_bin" "$@" >"$tmp_root/failure-$label.out" 2>"$tmp_root/failure-$label.err"; then
        echo "expected failure was accepted: $label" >&2
        exit 1
    fi
}

expect_failure empty-endpoint --endpoint " " health
expect_failure zero-timeout --timeout-secs 0 health
expect_failure cert-without-key --client-cert "$tmp_root/client.pem" health
expect_failure key-without-cert --client-key "$tmp_root/client.key" health
expect_failure token-conflict --token secret --token-file "$tmp_root/token" health
expect_failure empty-token --token " " health
expect_failure missing-token-file --token-file "$tmp_root/missing-token" health
expect_failure tls-option-on-http --endpoint 127.0.0.1:50051 --tls-domain example.test health
credential_error="$tmp_root/credential-redaction.err"
if "$client_bin" \
    --endpoint 'http://observer:supersecret@127.0.0.1:59997/?token=hidden' \
    --timeout-secs 1 health >"$tmp_root/credential-redaction.out" 2>"$credential_error"; then
    echo "credential-bearing endpoint unexpectedly connected" >&2
    exit 1
fi
if rg --no-config -q 'observer:|supersecret|token=hidden' "$credential_error"; then
    echo "credential or endpoint userinfo leaked in connection error" >&2
    sed -n '1,40p' "$credential_error" >&2
    exit 1
fi

for shell in bash elvish fish powershell zsh; do
    "$client_bin" completions "$shell" >"$tmp_root/completions-$shell.txt"
    [[ -s "$tmp_root/completions-$shell.txt" ]]
    rg --no-config -q 'oscal-cli' "$tmp_root/completions-$shell.txt"
done
expect_failure invalid-completion-shell completions unsupported-shell
"$client_bin" search --help >"$tmp_root/search-help.txt"
rg --no-config -q 'Text query sent to the selected read-only search path' "$tmp_root/search-help.txt"
rg --no-config -q 'Opaque continuation token returned by standard search' "$tmp_root/search-help.txt"
rg --no-config -q 'Semantic-search framework filter; available only with --semantic' "$tmp_root/search-help.txt"
"$client_bin" --format json doctor >"$tmp_root/doctor.json"
rg --no-config -q '"network_checked": false' "$tmp_root/doctor.json"
rg --no-config -q '"capability_negotiation": "not_declared_by_protocol"' "$tmp_root/doctor.json"
"$client_bin" --endpoint "" --format json doctor >"$tmp_root/doctor-invalid-endpoint.json"
rg --no-config -q '"status": "attention"' "$tmp_root/doctor-invalid-endpoint.json"
"$client_bin" --timeout-secs 0 --format json doctor >"$tmp_root/doctor-invalid-timeout.json"
rg --no-config -q '"id": "timeout"' "$tmp_root/doctor-invalid-timeout.json"
"$client_bin" --endpoint 'http://[' --token-file "$tmp_root/missing-token" \
    --format json proto >"$tmp_root/proto-offline.json"
rg --no-config -q '"read_only": true' "$tmp_root/proto-offline.json"
"$client_bin" --endpoint 'http://[' --token-file "$tmp_root/missing-token" \
    --format proto proto >"$tmp_root/proto-descriptor.bin"
python3 - "$tmp_root/proto-offline.json" "$tmp_root/proto-descriptor.bin" <<'PY'
import hashlib
import json
import pathlib
import sys

metadata = json.loads(pathlib.Path(sys.argv[1]).read_text())
descriptor = pathlib.Path(sys.argv[2]).read_bytes()
actual = "sha256:" + hashlib.sha256(descriptor).hexdigest()
if metadata["descriptor_sha256"] != actual:
    raise SystemExit(
        f"descriptor digest mismatch: metadata={metadata['descriptor_sha256']} actual={actual}"
    )
PY
[[ -s "$tmp_root/proto-descriptor.bin" ]]
"$client_bin" --endpoint 'http://[' --token-file "$tmp_root/missing-token" \
    --capture-dir "$tmp_root/local-captures" --format json capture list \
    >"$tmp_root/capture-offline.json"
rg --no-config -q '^\[\]$' "$tmp_root/capture-offline.json"
"$client_bin" --endpoint 'https://example.test:443' --format json doctor \
    >"$tmp_root/doctor-https.json"
"$client_bin" --endpoint 'https://example.test:443' --tls-domain example.test \
    --ca-cert "$tmp_root/doctor-ca.pem" --format json doctor \
    >"$tmp_root/doctor-ca.json"
printf '%s\n' 'not-a-real-certificate' >"$tmp_root/doctor-ca.pem"
"$client_bin" --endpoint 'https://example.test:443' --tls-domain example.test \
    --ca-cert "$tmp_root/doctor-ca.pem" --format json doctor \
    >"$tmp_root/doctor-ca-readable.json"
printf '%s\n' 'token-value' >"$tmp_root/doctor-token"
"$client_bin" --token token-value --format json doctor >"$tmp_root/doctor-token-arg.json"
"$client_bin" --token-file "$tmp_root/doctor-token" --format json doctor \
    >"$tmp_root/doctor-token-file.json"
"$client_bin" --token-file "$tmp_root/missing-token" --format json doctor \
    >"$tmp_root/doctor-token-missing.json"
"$client_bin" --token-file "$tmp_root/doctor-ca.pem" --format jsonl doctor \
    >"$tmp_root/doctor-jsonl.json"
"$client_bin" --endpoint 'https://example.test:443' --tls-domain example.test \
    --client-cert "$tmp_root/doctor-ca.pem" --client-key "$tmp_root/doctor-ca.pem" \
    --format table doctor >"$tmp_root/doctor-mtls.txt"
"$client_bin" --endpoint 'https://example.test:443' --client-cert "$tmp_root/doctor-ca.pem" \
    --format json doctor >"$tmp_root/doctor-mtls-incomplete.json"
doctor_capture_file="$tmp_root/doctor-capture-file"
printf '%s\n' 'not-a-directory' >"$doctor_capture_file"
"$client_bin" --capture-dir "$doctor_capture_file" --format json doctor \
    >"$tmp_root/doctor-capture-file.json"
"$client_bin" --capture-dir "$tmp_root/doctor-capture-missing" --format json doctor \
    >"$tmp_root/doctor-capture-missing.json"
"$client_bin" --endpoint 'http://[' --format json doctor \
    >"$tmp_root/doctor-malformed-endpoint.json"
expect_failure doctor-proto --format proto doctor

database_dsn=":memory:"
fixture_mode="${OSCALIFY_E2E_FIXTURES:-0}"
fixture_catalog_id="does-not-exist"
fixture_profile_id="does-not-exist"
fixture_component_id="does-not-exist"
fixture_ssp_id="does-not-exist"
fixture_assessment_plan_id="does-not-exist"
fixture_assessment_results_id="does-not-exist"
fixture_poam_id="does-not-exist"
fixture_mapping_id="does-not-exist"
fixture_framework_id="does-not-exist"
fixture_entity_id="does-not-exist"
fixture_snapshot_id="does-not-exist"
fixture_claim_id="does-not-exist"
fixture_evidence_id="does-not-exist"
fixture_node_id="does-not-exist"
fixture_edge_id="does-not-exist"
if [[ "$fixture_mode" == "1" ]]; then
    if ! command -v sqlite3 >/dev/null 2>&1; then
        echo "sqlite3 is required for OSCALIFY_E2E_FIXTURES=1" >&2
        exit 1
    fi
    if [[ ! -f "$oscalify_root/oscal.db" ]]; then
        echo "OSCALIFY_E2E_FIXTURES=1 requires the read-only source database at $oscalify_root/oscal.db" >&2
        exit 1
    fi
    # Copy first: the OSCALify source database is never opened for writes.
    fixture_db="$tmp_root/fixture.db"
    cp "$oscalify_root/oscal.db" "$fixture_db"
    projection_event_time="2026-01-01T00:00:00Z"
    projection_event_hash="$(python3 - <<'PY'
import hashlib
import json

payload = {
    "Version": "graph-projection-v1",
    "Sequence": 1,
    "EventID": "projection-event-fixture",
    "EdgeID": "fixture-edge",
    "ClaimID": "fixture-claim",
    "FromNode": "fixture-node",
    "ToNode": "fixture-target-node",
    "Relation": "depends_on",
    "EvidenceDigest": "sha256:e3b0c44298fc1c149af4c8996fb92427ae41e4649b934ca495991b7852b855",
    "TrustState": "candidate",
    "PreviousHash": "",
    "ProjectedAt": "2026-01-01T00:00:00Z",
}
encoded = json.dumps(payload, separators=(",", ":")).encode()
print("sha256:" + hashlib.sha256(encoded).hexdigest())
PY
)"
    sqlite3 "$fixture_db" <<'SQL'
INSERT OR REPLACE INTO catalogs(uuid, title, version, data) VALUES ('fixture-catalog', 'Fixture Catalog', '1.0.0', X'');
INSERT OR REPLACE INTO profiles(uuid, title, version, data) VALUES ('fixture-profile', 'Fixture Profile', '1.0.0', X'');
INSERT OR REPLACE INTO component_definitions(uuid, title, version, data) VALUES ('fixture-component', 'Fixture Component', '1.0.0', X'');
INSERT OR REPLACE INTO ssps(uuid, title, version, data) VALUES ('fixture-ssp', 'Fixture SSP', '1.0.0', X'');
INSERT OR REPLACE INTO assessment_plans(uuid, title, version, data) VALUES ('fixture-assessment-plan', 'Fixture Assessment Plan', '1.0.0', X'');
INSERT OR REPLACE INTO assessment_results(uuid, title, version, data) VALUES ('fixture-assessment-results', 'Fixture Assessment Results', '1.0.0', X'');
INSERT OR REPLACE INTO poams(uuid, title, version, data) VALUES ('fixture-poam', 'Fixture POA&M', '1.0.0', X'');
INSERT OR REPLACE INTO mappings(uuid, title, version, data) VALUES ('fixture-mapping', 'Fixture Mapping', '1.0.0', X'');
INSERT OR REPLACE INTO kg_entities(urn, entity_type, version, status, valid_from, payload) VALUES ('urn:fixture:entity', 'fixture', 1, 'active', datetime('now'), CAST('{}' AS BLOB));
INSERT OR REPLACE INTO kg_entities(urn, entity_type, version, status, valid_from, payload) VALUES ('urn:fixture:framework', 'reg:Framework', 1, 'active', datetime('now'), CAST('{"@id":"urn:fixture:framework","@type":"reg:Framework","ref_id":"fixture-framework","name":"Fixture Framework","provider":"Fixture Provider","version":"1.0","locale":"en-CA"}' AS BLOB));
INSERT OR REPLACE INTO kg_snapshots(name) VALUES ('fixture-snapshot');
INSERT OR REPLACE INTO kg_releases(name, snapshot_name) VALUES ('fixture-release', 'fixture-snapshot');
INSERT OR REPLACE INTO kg_claims(id, claim_type, subject_json, predicate_json, object_json, issuer_json, bom_kind, valid_from, observed_time, source_refs_json, proof_refs_json, policy_refs_json, extensions_json, trust_state, proof_state_json, created_at) VALUES ('fixture-claim', 'fixture', CAST('{"kind":"artifact","id":"fixture"}' AS BLOB), CAST('{"relation":"depends_on"}' AS BLOB), CAST('{"kind":"service","id":"target"}' AS BLOB), CAST('{"kind":"agent","id":"fixture"}' AS BLOB), 'sbom', datetime('now'), datetime('now'), CAST('[{"digest":"sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}]' AS BLOB), CAST('[]' AS BLOB), CAST('[]' AS BLOB), CAST('{}' AS BLOB), 'candidate', CAST('{}' AS BLOB), datetime('now'));
INSERT OR REPLACE INTO kg_evidence(id, media_type, bom_kind, digest, size_bytes, storage_json, produced_by_json, subject_refs_json, predicate_type, spec_json, valid_from, integrity_methods_json, classification, extensions_json, blob) VALUES ('fixture-evidence', 'text/plain', 'sbom', 'sha256:e3b0c44298fc1c149aaf4c8996fb92427ae41e4649b934ca495991b7852b855', 0, CAST('{}' AS BLOB), CAST('{}' AS BLOB), CAST('[]' AS BLOB), '', CAST('{}' AS BLOB), datetime('now'), CAST('[]' AS BLOB), 'internal', CAST('{}' AS BLOB), X'');
INSERT OR REPLACE INTO kg_graph_nodes(id, kind, urn, labels_json) VALUES ('fixture-node', 'artifact', 'urn:fixture:node', CAST('{}' AS BLOB));
INSERT OR REPLACE INTO kg_graph_nodes(id, kind, urn, labels_json) VALUES ('fixture-target-node', 'service', 'urn:fixture:target', CAST('{}' AS BLOB));
INSERT OR REPLACE INTO kg_graph_nodes(id, kind, urn, labels_json) VALUES ('fixture-snapshot', 'snapshot', 'urn:fixture:snapshot', CAST('{}' AS BLOB));
INSERT OR REPLACE INTO kg_graph_edges(id, from_node, to_node, relation, claim_id, evidence_digest, proof_state_json, valid_from, trust_state, weight, extensions_json) VALUES ('fixture-edge', 'fixture-node', 'fixture-target-node', 'depends_on', 'fixture-claim', 'sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855', CAST('{}' AS BLOB), datetime('now'), 'candidate', 1.0, CAST('{}' AS BLOB));
SQL
    sqlite3 "$fixture_db" <<SQL
CREATE TABLE IF NOT EXISTS graph_projection_events (
    sequence INTEGER PRIMARY KEY,
    event_id TEXT NOT NULL UNIQUE,
    edge_id TEXT NOT NULL,
    claim_id TEXT NOT NULL,
    from_node TEXT NOT NULL,
    to_node TEXT NOT NULL,
    relation TEXT NOT NULL,
    evidence_digest TEXT NOT NULL,
    trust_state TEXT NOT NULL,
    previous_hash TEXT NOT NULL,
    event_hash TEXT NOT NULL,
    projected_at DATETIME NOT NULL
);
INSERT OR REPLACE INTO graph_projection_events(
    sequence, event_id, edge_id, claim_id, from_node, to_node, relation,
    evidence_digest, trust_state, previous_hash, event_hash, projected_at
) VALUES (
    1, 'projection-event-fixture', 'fixture-edge', 'fixture-claim',
    'fixture-node', 'fixture-target-node', 'depends_on',
    'sha256:e3b0c44298fc1c149af4c8996fb92427ae41e4649b934ca495991b7852b855',
    'candidate', '', '$projection_event_hash', '$projection_event_time'
);
SQL
    database_dsn="$fixture_db"
    fixture_catalog_id="fixture-catalog"
    fixture_profile_id="fixture-profile"
    fixture_component_id="fixture-component"
    fixture_ssp_id="fixture-ssp"
    fixture_assessment_plan_id="fixture-assessment-plan"
    fixture_assessment_results_id="fixture-assessment-results"
    fixture_poam_id="fixture-poam"
    fixture_mapping_id="fixture-mapping"
    fixture_framework_id="fixture-framework"
    fixture_entity_id="urn:fixture:entity"
    fixture_snapshot_id="fixture-snapshot"
    fixture_claim_id="fixture-claim"
    fixture_evidence_id="fixture-evidence"
    fixture_node_id="fixture-node"
    fixture_edge_id="fixture-edge"
fi

cat >"$tmp_root/server.yaml" <<EOF
server:
  addr: 127.0.0.1:$port
  max_recv_msg_size_mb: 4
  max_send_msg_size_mb: 4
  max_concurrent_streams: 1000
  keepalive_time: 2m
  keepalive_timeout: 20s
  enable_reflection: true
  enable_pprof: false
  pprof_addr: :6060
  shutdown_timeout: 30s
store:
  dsn: "$database_dsn"
  max_open_conn: 10
  max_idle_conn: 2
  conn_max_lifetime: 1h
vector:
  backend: sqlite
  uri: ""
observability:
  metrics_enabled: false
  metrics_addr: :9090
  tracing_enabled: false
  tracing_endpoint: ""
  tracing_sample_rate: 0.1
  log_level: info
  log_format: text
security:
  rate_limit_rps: 100
  rate_limit_burst: 200
  auth_mode: none
EOF

"$server_bin" -config "$tmp_root/server.yaml" >"$tmp_root/server.log" 2>&1 &
server_pid=$!

ready=0
for _ in {1..30}; do
    if "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
        model list catalog --page-size 1 >"$tmp_root/read.json" 2>"$tmp_root/read.err"; then
        ready=1
        break
    fi
    sleep 0.2
done

if [[ "$ready" != 1 ]]; then
    echo "OSCALify test server did not become ready" >&2
    sed -n '1,120p' "$tmp_root/server.log" >&2 || true
    sed -n '1,120p' "$tmp_root/read.err" >&2 || true
    exit 1
fi

"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    health >"$tmp_root/health.json"
rg --no-config -q '"status": "SERVING"' "$tmp_root/health.json"

"$client_bin" --format json proto >"$tmp_root/proto.json"
rg --no-config -q '"proto_lock_sha256": "sha256:' "$tmp_root/proto.json"
rg --no-config -q '"proto_file_count": 13' "$tmp_root/proto.json"
rg --no-config -q '"oscal_schema_version": "1.2.3"' "$tmp_root/proto.json"
rg --no-config -q '"oscal_schema_manifest_sha256": "sha256:' "$tmp_root/proto.json"

"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    framework list --page-size 1 >"$tmp_root/framework.json"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    claim list --page-size 1 >"$tmp_root/claim.json"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    graph node list --page-size 1 >"$tmp_root/graph.json"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    search observer >"$tmp_root/search.json"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    search --semantic observer >"$tmp_root/semantic-search.json"

for model in profile component-definition ssp assessment-plan assessment-results poam mapping; do
    "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
        model list "$model" --page-size 1 >"$tmp_root/model-$model.json"
done
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    entity list --page-size 1 >"$tmp_root/entity.json"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    snapshot list --page-size 1 >"$tmp_root/snapshot.json"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    release list --page-size 1 >"$tmp_root/release.json"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    graph edge list --page-size 1 >"$tmp_root/graph-edge.json"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    graph projection-events --claim-id does-not-exist >"$tmp_root/projection-events.json"
rg --no-config -q '"chainValid": true' "$tmp_root/projection-events.json"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format table \
    model list catalog --page-size 1 >"$tmp_root/table-model.txt"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format table \
    framework list --page-size 1 >"$tmp_root/table-framework.txt"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format table \
    graph node list --page-size 1 >"$tmp_root/table-graph.txt"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format jsonl \
    model list catalog --page-size 1 >"$tmp_root/jsonl-model.txt"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format proto \
    model list catalog --page-size 1 >"$tmp_root/proto-model.pb"
[[ -s "$tmp_root/table-model.txt" && -s "$tmp_root/table-framework.txt" ]]
[[ -f "$tmp_root/proto-model.pb" ]]

run_optional_read() {
    local label="$1"
    shift
    "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json "$@" \
        >"$tmp_root/optional-$label.json" 2>"$tmp_root/optional-$label.err" || true
}

for model in catalog profile component-definition ssp assessment-plan assessment-results poam mapping; do
    case "$model" in
        catalog) model_id="$fixture_catalog_id" ;;
        profile) model_id="$fixture_profile_id" ;;
        component-definition) model_id="$fixture_component_id" ;;
        ssp) model_id="$fixture_ssp_id" ;;
        assessment-plan) model_id="$fixture_assessment_plan_id" ;;
        assessment-results) model_id="$fixture_assessment_results_id" ;;
        poam) model_id="$fixture_poam_id" ;;
        mapping) model_id="$fixture_mapping_id" ;;
    esac
    run_optional_read "get-$model" model get "$model" "$model_id"
done
run_optional_read get-entity entity get "$fixture_entity_id"
run_optional_read get-framework framework get "$fixture_framework_id"
run_optional_read get-snapshot snapshot get "$fixture_snapshot_id"
run_optional_read get-claim claim get "$fixture_claim_id"
run_optional_read claim-events claim events "$fixture_claim_id"
run_optional_read claim-receipt claim receipt "$fixture_claim_id"
run_optional_read get-evidence evidence get "$fixture_evidence_id"
run_optional_read verify-evidence evidence verify "$fixture_evidence_id"
run_optional_read get-node graph node get "$fixture_node_id"
run_optional_read get-edge graph edge get "$fixture_edge_id"
run_optional_read traverse graph traverse "$fixture_node_id" --max-depth 1
run_optional_read shortest-path graph path "$fixture_node_id" fixture-target-node
run_optional_read impact-radius graph impact "$fixture_node_id" --max-depth 1
run_optional_read explain-claim graph explain "$fixture_claim_id"
run_optional_read trust-state graph trust "$fixture_claim_id"
run_optional_read verify-closure graph closure "$fixture_snapshot_id" beta
run_optional_read projection-events graph projection-events --claim-id "$fixture_claim_id"

if [[ "$fixture_mode" == "1" ]]; then
    run_fixture_proto() {
        local label="$1"
        shift
        "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format proto "$@" \
            >"$tmp_root/proto-$label.pb"
        [[ -f "$tmp_root/proto-$label.pb" ]]
    }

    run_fixture_table() {
        local label="$1"
        shift
        "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format table "$@" \
            >"$tmp_root/table-$label.txt"
        [[ -s "$tmp_root/table-$label.txt" ]]
    }

    run_fixture_jsonl() {
        local label="$1"
        shift
        "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format jsonl "$@" \
            >"$tmp_root/jsonl-$label.txt"
        [[ -s "$tmp_root/jsonl-$label.txt" ]]
    }

    run_fixture_proto health health
    run_fixture_proto search search Fixture
    run_fixture_proto semantic-search search --semantic Fixture
    for model in catalog profile component-definition ssp assessment-plan assessment-results poam mapping; do
        case "$model" in
            catalog) model_id="$fixture_catalog_id" ;;
            profile) model_id="$fixture_profile_id" ;;
            component-definition) model_id="$fixture_component_id" ;;
            ssp) model_id="$fixture_ssp_id" ;;
            assessment-plan) model_id="$fixture_assessment_plan_id" ;;
            assessment-results) model_id="$fixture_assessment_results_id" ;;
            poam) model_id="$fixture_poam_id" ;;
            mapping) model_id="$fixture_mapping_id" ;;
        esac
        run_fixture_proto "model-list-$model" model list "$model" --page-size 1
        run_fixture_proto "model-get-$model" model get "$model" "$model_id"
        run_fixture_table "model-list-$model" model list "$model" --page-size 1
        run_fixture_table "model-get-$model" model get "$model" "$model_id"
        run_fixture_jsonl "model-list-$model" model list "$model" --page-size 1
    done
    run_fixture_table entity-list entity list --page-size 1
    run_fixture_table entity-get entity get "$fixture_entity_id"
    run_fixture_table framework-list framework list --page-size 1
    run_fixture_table framework-get framework get "$fixture_framework_id"
    run_fixture_table snapshot-list snapshot list --page-size 1
    run_fixture_table snapshot-get snapshot get "$fixture_snapshot_id"
    run_fixture_table release-list release list --page-size 1
    run_fixture_table claim-list claim list --page-size 1
    run_fixture_table claim-get claim get "$fixture_claim_id"
    run_fixture_table claim-events claim events "$fixture_claim_id"
    run_fixture_table evidence-get evidence get "$fixture_evidence_id"
    run_fixture_table evidence-verify evidence verify "$fixture_evidence_id"
    run_fixture_table node-list graph node list --page-size 1
    run_fixture_table node-get graph node get "$fixture_node_id"
    run_fixture_table edge-list graph edge list --page-size 1
    run_fixture_table edge-get graph edge get "$fixture_edge_id"
    run_fixture_table projection-events graph projection-events --claim-id "$fixture_claim_id"
    rg --no-config -q 'chain_valid  valid' "$tmp_root/table-projection-events.txt"
    rg --no-config -q 'projection-event-fixture' "$tmp_root/table-projection-events.txt"
    run_fixture_table traverse graph traverse "$fixture_node_id" --max-depth 1
    run_fixture_table shortest-path graph path "$fixture_node_id" fixture-target-node
    run_fixture_table impact-radius graph impact "$fixture_node_id" --max-depth 1
    run_fixture_table explain-claim graph explain "$fixture_claim_id"
    run_fixture_table trust-state graph trust "$fixture_claim_id"
    run_fixture_table verify-closure graph closure "$fixture_snapshot_id" beta
    run_fixture_table search search Fixture
    run_fixture_table semantic-search search --semantic Fixture
    run_fixture_proto entity-list entity list --page-size 1
    run_fixture_proto entity-get entity get "$fixture_entity_id"
    run_fixture_proto framework-list framework list --page-size 1
    run_fixture_proto framework-get framework get "$fixture_framework_id"
    run_fixture_proto snapshot-list snapshot list --page-size 1
    run_fixture_proto snapshot-get snapshot get "$fixture_snapshot_id"
    run_fixture_proto release-list release list --page-size 1
    run_fixture_proto claim-list claim list --page-size 1
    run_fixture_proto claim-get claim get "$fixture_claim_id"
    run_fixture_proto claim-events claim events "$fixture_claim_id"
    run_fixture_proto evidence-get evidence get "$fixture_evidence_id"
    run_fixture_proto evidence-verify evidence verify "$fixture_evidence_id"
    run_fixture_proto node-list graph node list --page-size 1
    run_fixture_proto node-get graph node get "$fixture_node_id"
    run_fixture_proto edge-list graph edge list --page-size 1
    run_fixture_proto edge-get graph edge get "$fixture_edge_id"
    run_fixture_proto projection-events graph projection-events --edge-id "$fixture_edge_id"
    run_fixture_proto traverse graph traverse "$fixture_node_id" --max-depth 1
    run_fixture_proto shortest-path graph path "$fixture_node_id" fixture-target-node
    run_fixture_proto impact-radius graph impact "$fixture_node_id" --max-depth 1
    run_fixture_proto explain-claim graph explain "$fixture_claim_id"
    run_fixture_proto trust-state graph trust "$fixture_claim_id"
    run_fixture_proto verify-closure graph closure "$fixture_snapshot_id" beta
    run_fixture_jsonl claim-get claim get "$fixture_claim_id"
    run_fixture_jsonl projection-events graph projection-events --claim-id "$fixture_claim_id"
fi

if "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    search ' ' >"$tmp_root/invalid-search.json" 2>"$tmp_root/invalid-search.err"; then
    echo "invalid search input was accepted" >&2
    exit 1
fi
if "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    search observer --semantic --model catalog >"$tmp_root/invalid-search-model.json" \
    2>"$tmp_root/invalid-search-model.err"; then
    echo "semantic search accepted the standard --model filter" >&2
    exit 1
fi
if "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    search observer --framework framework >"$tmp_root/invalid-search-framework.json" \
    2>"$tmp_root/invalid-search-framework.err"; then
    echo "standard search accepted the semantic --framework filter" >&2
    exit 1
fi
if "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    search observer --semantic --page-token opaque-token >"$tmp_root/invalid-search-page-token.json" \
    2>"$tmp_root/invalid-search-page-token.err"; then
    echo "semantic search accepted a standard-search page token" >&2
    exit 1
fi
if "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    model list catalog --page-size 0 >"$tmp_root/invalid-page-size.json" 2>"$tmp_root/invalid-page-size.err"; then
    echo "invalid page size was accepted" >&2
    exit 1
fi
if "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    graph projection-events >"$tmp_root/invalid-projection-events.json" \
    2>"$tmp_root/invalid-projection-events.err"; then
    echo "projection events accepted without a claim or edge filter" >&2
    exit 1
fi
if "$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    graph projection-events --claim-id " " >"$tmp_root/empty-projection-events.json" \
    2>"$tmp_root/empty-projection-events.err"; then
    echo "projection events accepted an empty claim filter" >&2
    exit 1
fi

capture_root="$tmp_root/captures"
capture_outputs="$tmp_root/capture-outputs"
mkdir -p "$capture_outputs"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    --capture --capture-dir "$capture_root" search observer >"$tmp_root/captured-search.json"
"$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --format json \
    --capture --capture-dir "$capture_root" graph projection-events \
    --claim-id "$fixture_claim_id" >"$tmp_root/captured-projection-events.json"
"$client_bin" --capture-dir "$capture_root" --format json capture list \
    >"$capture_outputs/capture-list.json"
capture_id="$(find "$capture_root" -mindepth 1 -maxdepth 1 -type d -name 'capture-*' -exec basename {} \; | head -1)"
[[ -n "$capture_id" ]]
"$client_bin" --capture-dir "$capture_root" --format json capture show "$capture_id" \
    >"$capture_outputs/capture-show.json"
"$client_bin" --capture-dir "$capture_root" --format json capture verify "$capture_id" \
    >"$capture_outputs/capture-verify.json"
"$client_bin" --capture-dir "$capture_root" --format json capture prune \
    --older-than-days 1 >"$capture_outputs/capture-prune.json"
rg --no-config -q "$capture_id" "$capture_outputs/capture-verify.json"
rg --no-config -q '"dry_run": true' "$capture_outputs/capture-prune.json"
"$client_bin" --capture-dir "$capture_root" --format proto capture export "$capture_id" \
    >"$tmp_root/capture-export.pb"
[[ -f "$tmp_root/capture-export.pb" ]]
expect_failure invalid-capture-id --capture-dir "$capture_root" --format json capture show capture-invalid
cp "$capture_root/$capture_id/request.pb" "$tmp_root/request-original.pb"
cp "$capture_root/$capture_id/response.pb" "$tmp_root/response-original.pb"
printf '%s' 'x' >"$capture_root/$capture_id/request.pb"
expect_failure tampered-request --capture-dir "$capture_root" --format json capture show "$capture_id"
cp "$tmp_root/request-original.pb" "$capture_root/$capture_id/request.pb"
printf '%s' 'tampered' >"$capture_root/$capture_id/response.pb"
expect_failure tampered-capture --capture-dir "$capture_root" --format json capture show "$capture_id"
cp "$tmp_root/response-original.pb" "$capture_root/$capture_id/response.pb"
"$client_bin" --capture-dir "$capture_root" --format json capture verify \
    >"$capture_outputs/capture-verify-all.json"
python3 - "$capture_root/$capture_id/manifest.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
manifest = json.loads(path.read_text())
manifest["captured_at_unix_ms"] = 1
path.write_text(json.dumps(manifest, indent=2) + "\n")
PY
"$client_bin" --capture-dir "$capture_root" --format json capture prune \
    --older-than-days 1 --confirm >"$capture_outputs/capture-prune-confirmed.json"
rg --no-config -q 'deleted_ids' "$capture_outputs/capture-prune-confirmed.json"
[[ ! -e "$capture_root/$capture_id" ]]

for response in "$tmp_root"/*.json; do
    [[ -s "$response" ]]
done
rg --no-config -q 'nextPageToken|items|results' "$tmp_root/read.json"
rg --no-config -q 'results' "$tmp_root/semantic-search.json"

if rg --no-config -q '/(Create|Update|Delete|Ingest|Upload|Sync|Publish|Propose|Resolve|Generate|Project|Import|VerifyClaim)' \
    "$tmp_root/server.log"; then
    echo "mutation RPC observed during read-only smoke check" >&2
    exit 1
fi

if ! command -v tmux >/dev/null 2>&1; then
    echo "tmux is required for the TUI smoke check" >&2
    exit 1
fi

tui_command=(env)
if [[ -n "${LLVM_PROFILE_FILE:-}" ]]; then
    tui_command+=("LLVM_PROFILE_FILE=$LLVM_PROFILE_FILE")
fi
tui_command+=("$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 tui)
tmux new-session -d -s "$session" "${tui_command[*]}"
sleep 1
tmux capture-pane -J -p -t "$session" >"$tmp_root/tui.txt"
rg --no-config -q 'OSCALIFY OBSERVER' "$tmp_root/tui.txt"
rg --no-config -q 'Evidence margin' "$tmp_root/tui.txt"
tmux send-keys -t "$session" '?'
sleep 0.2
tmux capture-pane -J -p -t "$session" >"$tmp_root/tui-help.txt"
rg --no-config -q 'Help — read-only observer' "$tmp_root/tui-help.txt"
tmux send-keys -t "$session" Escape
if [[ "$fixture_mode" == "1" ]]; then
    close_fixture_detail() {
        if ! tmux has-session -t "$session" 2>/dev/null; then
            return
        fi
        tmux capture-pane -J -p -t "$session" >"$tmp_root/tui-state.txt"
        if rg --no-config -q 'j/k scroll' "$tmp_root/tui-state.txt"; then
            tmux send-keys -t "$session" Escape
            sleep 0.2
        fi
    }
    wait_for_tui_text() {
        local pattern="$1"
        local output_file="$2"
        for _ in {1..20}; do
            tmux capture-pane -J -p -t "$session" >"$output_file"
            if rg --no-config -q "$pattern" "$output_file"; then
                return 0
            fi
            sleep 0.1
        done
        echo "TUI did not render expected text: $pattern" >&2
        sed -n '1,120p' "$output_file" >&2 || true
        return 1
    }
    wait_for_tui_without_text() {
        local pattern="$1"
        local output_file="$2"
        for _ in {1..20}; do
            tmux capture-pane -J -p -t "$session" >"$output_file"
            if ! rg --no-config -q "$pattern" "$output_file"; then
                return 0
            fi
            sleep 0.1
        done
        echo "TUI retained unexpected text: $pattern" >&2
        sed -n '1,120p' "$output_file" >&2 || true
        return 1
    }
    wait_for_tui_without_text 'Help — read-only observer' "$tmp_root/tui-ready.txt"

    tmux send-keys -t "$session" 2 Enter
    wait_for_tui_text 'j/k scroll' "$tmp_root/tui-detail.txt"
    tmux send-keys -t "$session" j k PageDown PageUp
    close_fixture_detail
    tmux send-keys -t "$session" 2
    tmux send-keys -t "$session" j Enter
    sleep 0.2
    close_fixture_detail
    tmux send-keys -t "$session" 2
    tmux send-keys -t "$session" j j Enter
    wait_for_tui_text 'j/k scroll' "$tmp_root/tui-detail-3.txt"
    close_fixture_detail
    tmux send-keys -t "$session" 2 f depends_on Enter
    tmux send-keys -t "$session" Enter
    wait_for_tui_text 'Claim ' "$tmp_root/tui-claim-detail.txt"
    tmux send-keys -t "$session" e
    wait_for_tui_text 'Claim events' "$tmp_root/tui-claim-events.txt"
    close_fixture_detail
    tmux send-keys -t "$session" 4
    tmux send-keys -t "$session" Enter
    wait_for_tui_text 'j/k scroll' "$tmp_root/tui-edge-list-detail.txt"
    tmux send-keys -t "$session" j k PageDown PageUp
    close_fixture_detail
    tmux send-keys -t "$session" 3
    tmux send-keys -t "$session" / Fixture Enter
    tmux send-keys -t "$session" Enter
    wait_for_tui_text 'j/k scroll' "$tmp_root/tui-search-detail.txt"
    close_fixture_detail
    tmux send-keys -t "$session" j Enter
    wait_for_tui_text 'j/k scroll' "$tmp_root/tui-search-detail-2.txt"
    close_fixture_detail
    tmux send-keys -t "$session" 4 j Enter
    wait_for_tui_text 'j/k scroll' "$tmp_root/tui-node-detail.txt"
    close_fixture_detail
    tmux send-keys -t "$session" 4 f depends_on Enter
    tmux send-keys -t "$session" Enter
    wait_for_tui_text 'Edge ' "$tmp_root/tui-edge-detail.txt"
    tmux send-keys -t "$session" p
    wait_for_tui_text 'event hash' "$tmp_root/tui-projection-events.txt"
    rg --no-config -q 'Projection events' "$tmp_root/tui-projection-events.txt"
    rg --no-config -q 'chain valid' "$tmp_root/tui-projection-events.txt"
    rg --no-config -q 'projection-event-fixture' "$tmp_root/tui-projection-events.txt"
    rg --no-config -q 'event hash.*sha256:' "$tmp_root/tui-projection-events.txt"
    close_fixture_detail
    tmux send-keys -t "$session" 2 f Fixture Enter
    sleep 0.5
    tmux send-keys -t "$session" 4 f depends_on Enter
    sleep 0.5
    tmux send-keys -t "$session" 3 / z Backspace Escape
    tmux send-keys -t "$session" 2 f z Backspace Escape
fi
tmux send-keys -t "$session" 2 f catalog Enter
tmux send-keys -t "$session" 4 f edge Enter
tmux send-keys -t "$session" / observer Enter
tmux send-keys -t "$session" 1 2 3 4 5 Tab j k '[' ']' r
tmux send-keys -t "$session" q
for _ in {1..20}; do
    if ! tmux has-session -t "$session" 2>/dev/null; then
        break
    fi
    sleep 0.1
done
if tmux has-session -t "$session" 2>/dev/null; then
    echo "TUI did not exit after the read-only quit command" >&2
    exit 1
fi

for theme in vault hc; do
    theme_command=(env)
    if [[ -n "${LLVM_PROFILE_FILE:-}" ]]; then
        theme_command+=("LLVM_PROFILE_FILE=$LLVM_PROFILE_FILE")
    fi
    theme_command+=("$client_bin" --endpoint "127.0.0.1:$port" --timeout-secs 1 --theme "$theme" tui)
    tmux new-session -d -s "$session" "${theme_command[*]}"
    sleep 0.5
    tmux capture-pane -J -p -t "$session" >"$tmp_root/tui-$theme.txt"
    rg --no-config -q 'OSCALIFY OBSERVER' "$tmp_root/tui-$theme.txt"
    tmux send-keys -t "$session" q
    for _ in {1..20}; do
        if ! tmux has-session -t "$session" 2>/dev/null; then
            break
        fi
        sleep 0.1
    done
    if tmux has-session -t "$session" 2>/dev/null; then
        echo "TUI theme did not exit: $theme" >&2
        exit 1
    fi
done

offline_command=(env)
if [[ -n "${LLVM_PROFILE_FILE:-}" ]]; then
    offline_command+=("LLVM_PROFILE_FILE=$LLVM_PROFILE_FILE")
fi
offline_command+=("$client_bin" --endpoint 127.0.0.1:59999 --timeout-secs 1 tui)
tmux new-session -d -s "$session" "${offline_command[*]}"
sleep 1.2
tmux capture-pane -J -p -t "$session" >"$tmp_root/tui-offline.txt"
rg --no-config -q 'OSCALIFY OBSERVER' "$tmp_root/tui-offline.txt"
tmux send-keys -t "$session" r q
for _ in {1..20}; do
    if ! tmux has-session -t "$session" 2>/dev/null; then
        break
    fi
    sleep 0.1
done
if tmux has-session -t "$session" 2>/dev/null; then
    echo "disconnected TUI did not exit" >&2
    exit 1
fi

interrupt_command=(env)
if [[ -n "${LLVM_PROFILE_FILE:-}" ]]; then
    interrupt_command+=("LLVM_PROFILE_FILE=$LLVM_PROFILE_FILE")
fi
interrupt_command+=("$client_bin" --endpoint 127.0.0.1:59998 --timeout-secs 5 tui)
tmux new-session -d -s "$session" "${interrupt_command[*]}"
sleep 0.2
tmux send-keys -t "$session" C-c
for _ in {1..20}; do
    if ! tmux has-session -t "$session" 2>/dev/null; then
        break
    fi
    sleep 0.1
done
if tmux has-session -t "$session" 2>/dev/null; then
    echo "TUI did not exit on Ctrl-C during startup" >&2
    exit 1
fi

persist_tui_evidence
printf '%s\n' 'E2E smoke passed: read-only CLI calls and TUI shell rendered against an isolated OSCALify server.'

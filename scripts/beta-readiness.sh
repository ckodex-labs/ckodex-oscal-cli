#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
oscalify_root="${OSCALIFY_ROOT:-"$client_root/../ckodex-oscalify"}"
output="${OSCAL_BETA_READINESS_OUTPUT:-$client_root/target/beta-readiness.json}"
evidence_root="${OSCAL_BETA_READINESS_EVIDENCE_DIR:-$client_root/target/beta-readiness-evidence}"
release_bin=""
run_root=""
check_ids=()
policy_cargo_home="${OSCAL_BETA_CARGO_HOME:-${CARGO_HOME:-$client_root/target/beta-policy-cargo-home}}"

usage() {
    cat <<'EOF'
usage: beta-readiness.sh [options]

Run the local beta gates and write a machine-readable readiness manifest.
The script never writes to the OSCALify checkout. External identity,
authorization, signing, and retention approvals remain pending in the output.

Options:
  --output FILE       Readiness manifest (default: target/beta-readiness.json)
  --evidence-dir DIR  Per-gate stdout/stderr evidence (default: target/beta-readiness-evidence)
  -h, --help          Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --output)
            output="${2:?missing value for --output}"
            shift 2
            ;;
        --evidence-dir)
            evidence_root="${2:?missing value for --evidence-dir}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "unknown option: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

mkdir -p "$(dirname "$output")" "$evidence_root"
mkdir -p "$policy_cargo_home"
run_root="$(mktemp -d "${TMPDIR:-/tmp}/oscal-cli-beta-readiness.XXXXXX")"
trap 'rm -rf "$run_root"' EXIT

run_check() {
    local id="$1"
    local scope="local"
    if [[ "$id" == "--external" ]]; then
        scope="external"
        id="$2"
        shift 2
    else
        shift
    fi
    local stdout="$evidence_root/$id.stdout"
    local stderr="$evidence_root/$id.stderr"
    local command_file="$evidence_root/$id.command"
    local status

    check_ids+=("$id")
    printf '%q ' "$@" >"$command_file"
    printf '\n' >>"$command_file"
    if "$@" >"$stdout" 2>"$stderr"; then
        status=0
    else
        status=$?
    fi
    printf '%s\n' "$status" >"$evidence_root/$id.exit_code"
    if [[ "$status" == 0 ]]; then
        printf '%s beta gate passed: %s\n' "$scope" "$id"
    else
        printf '%s beta gate failed: %s (exit %s)\n' "$scope" "$id" "$status" >&2
    fi
}

run_external_check() {
    run_check --external "$@"
}

release_bin="${OSCAL_CLI_BIN:-}"

boundary_before="$evidence_root/upstream-proto-before.txt"
status_before="$evidence_root/upstream-status-before.txt"
if [[ -d "$oscalify_root/.git" ]] || git -C "$oscalify_root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git -C "$oscalify_root" diff --name-only -- proto/oscal | sort >"$boundary_before"
    git -C "$oscalify_root" status --short --untracked-files=all | sort >"$status_before"
else
    printf '%s\n' 'source checkout is not a Git worktree' >"$boundary_before"
    printf '%s\n' 'source checkout is not a Git worktree' >"$status_before"
fi

run_check format cargo fmt --all -- --check
run_check shell-syntax bash -n "$client_root"/scripts/*.sh
run_check clippy cargo clippy --all-targets --all-features -- -D warnings
run_check cargo-test cargo test --all-targets --all-features
run_check nextest cargo nextest run --all-features
run_check deny env CARGO_HOME="$policy_cargo_home" cargo deny check advisories licenses bans sources
run_check audit env CARGO_HOME="$policy_cargo_home" cargo audit --json
run_check release-build cargo build --release --bin oscal-cli
if [[ ! -x "$release_bin" ]]; then
    if resolved_bin="$("$client_root/scripts/cargo-binary-path.sh" release)"; then
        release_bin="$resolved_bin"
    fi
fi
run_external_check protocol-audit env OSCALIFY_ROOT="$oscalify_root" \
    "$client_root/scripts/protocol-audit.sh" \
    --output "$evidence_root/protocol-audit.json"
run_external_check protocol-override env OSCALIFY_PROTO_ROOT="$oscalify_root/proto/oscal" \
    cargo check --quiet
run_check protocol-provenance env OSCAL_CLI_BIN="$release_bin" \
    "$client_root/scripts/proto-provenance.sh" \
    --output "$evidence_root/proto-provenance.json"
release_bundle_output="$run_root/oscal-cli-beta.tar.gz"
run_check release-bundle env OSCAL_CLI_BIN="$release_bin" \
    "$client_root/scripts/release-bundle.sh" \
    --output "$release_bundle_output"
if [[ "$(cat "$evidence_root/release-bundle.exit_code")" == 0 ]]; then
    cp "$release_bundle_output" "$evidence_root/oscal-cli-beta.tar.gz"
    cp "$release_bundle_output.sha256" "$evidence_root/oscal-cli-beta.tar.gz.sha256"
    cp "$release_bundle_output.release-provenance.json" \
        "$evidence_root/oscal-cli-beta.tar.gz.release-provenance.json"
fi
run_check e2e env OSCAL_CLI_BIN="$release_bin" OSCALIFY_ROOT="$oscalify_root" \
    OSCALIFY_E2E_EVIDENCE_DIR="$evidence_root/tui" \
    OSCALIFY_E2E_FIXTURES=1 \
    OSCALIFY_REGENERATE_PROTO=1 \
    "$client_root/scripts/e2e-smoke.sh"
run_check tls env OSCAL_CLI_BIN="$release_bin" OSCALIFY_ROOT="$oscalify_root" \
    OSCALIFY_REGENERATE_PROTO=1 \
    "$client_root/scripts/tls-smoke.sh"
run_check coverage env OSCAL_COVERAGE_MIN_REGIONS=80 OSCAL_COVERAGE_MIN_LINES=80 \
    "$client_root/scripts/coverage-smoke.sh"

boundary_after="$evidence_root/upstream-proto-after.txt"
status_after="$evidence_root/upstream-status-after.txt"
if git -C "$oscalify_root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git -C "$oscalify_root" diff --name-only -- proto/oscal | sort >"$boundary_after"
    git -C "$oscalify_root" status --short --untracked-files=all | sort >"$status_after"
else
    printf '%s\n' 'source checkout is not a Git worktree' >"$boundary_after"
    printf '%s\n' 'source checkout is not a Git worktree' >"$status_after"
fi

# Variables intentionally expand in the child shell, not this shell.
# shellcheck disable=SC2016
run_check source-boundary env \
    CLIENT_ROOT="$client_root" \
    OSCALIFY_ROOT="$oscalify_root" \
    BOUNDARY_BEFORE="$boundary_before" \
    BOUNDARY_AFTER="$boundary_after" \
    STATUS_BEFORE="$status_before" \
    STATUS_AFTER="$status_after" \
    bash -c '
        set -euo pipefail
        test -f "$OSCALIFY_ROOT/proto/oscal/services/v1/transparency_exchange_service.proto"
        diff -u \
            "$OSCALIFY_ROOT/proto/oscal/services/v1/transparency_exchange_service.proto" \
            "$CLIENT_ROOT/proto/oscal/services/v1/transparency_exchange_service.proto"
        git -C "$OSCALIFY_ROOT" diff --check -- proto/oscal
        cmp -s "$BOUNDARY_BEFORE" "$BOUNDARY_AFTER"
        cmp -s "$STATUS_BEFORE" "$STATUS_AFTER"
    '

python3 - "$client_root" "$oscalify_root" "$output" "$evidence_root" "$status_before" "${check_ids[@]}" <<'PY'
import datetime
import json
import pathlib
import re
import sys

client_root = pathlib.Path(sys.argv[1]).resolve()
oscalify_root = pathlib.Path(sys.argv[2]).resolve()
output = pathlib.Path(sys.argv[3]).resolve()
evidence_root = pathlib.Path(sys.argv[4]).resolve()
status_before = pathlib.Path(sys.argv[5]).resolve()
check_ids = sys.argv[6:]

def relative(path):
    try:
        return str(path.relative_to(client_root))
    except ValueError:
        return str(path)

checks = []
for check_id in check_ids:
    status_path = evidence_root / f"{check_id}.exit_code"
    stdout_path = evidence_root / f"{check_id}.stdout"
    stderr_path = evidence_root / f"{check_id}.stderr"
    command_path = evidence_root / f"{check_id}.command"
    exit_code = int(status_path.read_text().strip())
    checks.append(
        {
            "id": check_id,
            "scope": "external" if check_id in {"protocol-audit", "protocol-override"} else "local",
            "status": "passed" if exit_code == 0 else "failed",
            "exit_code": exit_code,
            "command": command_path.read_text().strip(),
            "stdout": relative(stdout_path),
            "stderr": relative(stderr_path),
        }
    )

local_checks = [check for check in checks if check["scope"] == "local"]
local_passed = all(check["status"] == "passed" for check in local_checks)
external_gates = [
    {
        "id": "release-signing-identity",
        "status": "pending_human_review",
        "reason": "A release signing identity and verification policy are not held by this checkout.",
    },
    {
        "id": "upstream-authorization",
        "status": "pending_human_review",
        "reason": "A real upstream environment must approve token scope, mTLS identity, and authorization behavior.",
    },
    {
        "id": "capture-retention-policy",
        "status": "pending_human_review",
        "reason": "Organization-specific retention duration, redaction, and evidence handling rules are deployment inputs.",
    },
    {
        "id": "upstream-protocol-acceptance",
        "status": "pending_human_review",
        "reason": "The current source proto and generated bindings match locally, but source ownership, compatibility with older servers, and release acceptance require the OSCALify owner.",
    },
]
protocol_checks = [
    check for check in checks if check["id"] in {"protocol-audit", "protocol-override"}
]
if any(check["status"] != "passed" for check in protocol_checks):
    for gate in external_gates:
        if gate["id"] == "upstream-protocol-acceptance":
            gate["status"] = "pending_external_drift"
            gate["reason"] = "The external proto source does not match this client’s reviewed proto.lock; generated-source alignment belongs to OSCALify."
            break

release_bundle = None
bundle_archive = evidence_root / "oscal-cli-beta.tar.gz"
bundle_checksum = evidence_root / "oscal-cli-beta.tar.gz.sha256"
bundle_provenance = evidence_root / "oscal-cli-beta.tar.gz.release-provenance.json"
bundle_check_passed = any(
    check["id"] == "release-bundle" and check["status"] == "passed"
    for check in checks
)
if bundle_check_passed and bundle_archive.exists() and bundle_checksum.exists() and bundle_provenance.exists():
    release_bundle = {
        "archive": relative(bundle_archive),
        "checksum": relative(bundle_checksum),
        "provenance": relative(bundle_provenance),
        "signing": "unsigned_local_artifact",
    }

boundary_before = evidence_root / "upstream-proto-before.txt"
proto_diff_count = None
if boundary_before.exists():
    lines = [line for line in boundary_before.read_text().splitlines() if line]
    if all(not line.startswith("source checkout") for line in lines):
        proto_diff_count = len(lines)

source_status_count = None
if status_before.exists():
    lines = [line for line in status_before.read_text().splitlines() if line]
    if all(not line.startswith("source checkout") for line in lines):
        source_status_count = len(lines)

manifest = {
    "schema": "urn:oscalify:observer:beta-readiness:v1",
    "generated_at_utc": datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "client": {
        "name": "oscal-cli",
        "root": str(client_root),
        "version": "0.1.0",
        "read_only": True,
    },
    "protocol": {
        "source_root": str(oscalify_root),
        "vendored_root": "proto/oscal",
        "source_proto_diff_files_observed": proto_diff_count,
        "source_status_files_observed": source_status_count,
        "source_mutation_policy": "read_only_input",
    },
    "verdict": {
        "local_gates_passed": local_passed,
        "local_gate_count": len(local_checks),
        "external_check_count": len(checks) - len(local_checks),
        "external_gates_pending": [gate["id"] for gate in external_gates],
        "status": "local_beta_candidate_requires_human_review" if local_passed else "local_gate_failed",
    },
    "checks": checks,
    "release_bundle": release_bundle,
    "external_gates": external_gates,
    "evidence_root": relative(evidence_root),
}
output.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
print(f"beta readiness manifest: {output}")
print(f"local gates passed: {str(local_passed).lower()}")
PY

if python3 - "$output" <<'PY'
import json
import sys
manifest = json.load(open(sys.argv[1]))
raise SystemExit(0 if manifest["verdict"]["local_gates_passed"] else 1)
PY
then
    printf '%s\n' 'local beta readiness passed; human review gates remain pending.'
else
    printf '%s\n' 'local beta readiness failed; inspect the evidence directory.' >&2
    exit 1
fi

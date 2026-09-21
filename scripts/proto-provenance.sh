#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
client_bin="${OSCAL_CLI_BIN:-}"
output=""
signing_key=""
bundle=""
verify_key=""
verify_bundle=""

usage() {
    cat <<'EOF'
usage: proto-provenance.sh --output FILE [options]

Generate deterministic provenance for the vendored OSCALify protocol snapshot.

Options:
  --signing-key FILE       Sign the provenance with cosign and write --bundle.
  --bundle FILE            Sigstore bundle output/input path.
  --verify-key FILE        Verify --bundle with this public key.
  --verify-bundle FILE     Verify an existing bundle without signing.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --output)
            output="${2:?missing value for --output}"
            shift 2
            ;;
        --signing-key)
            signing_key="${2:?missing value for --signing-key}"
            shift 2
            ;;
        --bundle)
            bundle="${2:?missing value for --bundle}"
            shift 2
            ;;
        --verify-key)
            verify_key="${2:?missing value for --verify-key}"
            shift 2
            ;;
        --verify-bundle)
            verify_bundle="${2:?missing value for --verify-bundle}"
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

if [[ -z "$output" ]]; then
    echo "--output is required" >&2
    usage >&2
    exit 2
fi
if [[ -n "$signing_key" && -z "$bundle" ]]; then
    echo "--signing-key requires --bundle" >&2
    exit 2
fi
if [[ -n "$verify_key" && -z "$verify_bundle" ]]; then
    echo "--verify-key requires --verify-bundle" >&2
    exit 2
fi
if [[ -n "$verify_bundle" && -z "$verify_key" ]]; then
    echo "--verify-bundle requires --verify-key" >&2
    exit 2
fi
if [[ ! -x "$client_bin" ]]; then
    (cd "$client_root" && cargo build --quiet)
    if resolved_bin="$("$client_root/scripts/cargo-binary-path.sh" debug)"; then
        client_bin="$resolved_bin"
    fi
fi
if [[ ! -x "$client_bin" ]]; then
    echo "client binary not found: $client_bin" >&2
    exit 1
fi

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/oscal-cli-provenance.XXXXXX")"
trap 'rm -rf "$tmp_root"' EXIT
proto_json="$tmp_root/proto.json"
"$client_bin" --format json proto >"$proto_json"
mkdir -p "$(dirname "$output")"

python3 - "$proto_json" "$output" <<'PY'
import json
import pathlib
import sys

source = json.loads(pathlib.Path(sys.argv[1]).read_text())
record = {
    "descriptor_bytes": source["descriptor_bytes"],
    "descriptor_sha256": source["descriptor_sha256"],
    "proto_file_count": source["proto_file_count"],
    "proto_lock_sha256": source["proto_lock_sha256"],
    "oscal_schema_version": source["oscal_schema_version"],
    "oscal_schema_manifest_sha256": source["oscal_schema_manifest_sha256"],
    "oscal_schema_source_commit": source["oscal_schema_source_commit"],
    "oscal_schema_release_zip_sha256": source["oscal_schema_release_zip_sha256"],
    "read_only": source["read_only"],
    "schema": "urn:oscalify:observer:proto-provenance:v1",
    "services": source["services"],
    "source": "proto/oscal",
}
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(record, indent=2, sort_keys=True) + "\n"
)
PY

if [[ -n "$signing_key" ]]; then
    if ! command -v cosign >/dev/null 2>&1; then
        echo "cosign is required for signing provenance" >&2
        exit 1
    fi
    COSIGN_PASSWORD="${COSIGN_PASSWORD:-}" cosign sign-blob \
        --key "$signing_key" --bundle "$bundle" "$output" >/dev/null
fi

if [[ -n "$verify_bundle" ]]; then
    if ! command -v cosign >/dev/null 2>&1; then
        echo "cosign is required for verifying provenance" >&2
        exit 1
    fi
    cosign verify-blob --key "$verify_key" --bundle "$verify_bundle" \
        "$output" >/dev/null
fi

printf 'protocol provenance written: %s\n' "$output"
if [[ -n "$bundle" && -f "$bundle" ]]; then
    printf 'protocol provenance bundle: %s\n' "$bundle"
fi

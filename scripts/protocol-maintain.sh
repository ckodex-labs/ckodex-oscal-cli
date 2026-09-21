#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_root="${OSCALIFY_ROOT:-"$client_root/../ckodex-oscalify"}"
audit_output="${OSCAL_PROTOCOL_AUDIT_OUTPUT:-$client_root/target/protocol-audit.json}"
run_sync=0
allow_dirty_source=0

usage() {
    cat <<'EOF'
usage: protocol-maintain.sh [options]

Run protocol drift audit and optionally refresh the vendored snapshot.
The source checkout is never modified unless --sync is explicitly requested.

Options:
  --source-root DIR          OSCALify checkout (default: ../ckodex-oscalify)
  --audit-output FILE        Protocol audit output path (default: target/protocol-audit.json)
  --sync                     Refresh vendored proto snapshot when drift is detected
  --allow-dirty-source       Permit sync from a dirty source workspace (recorded in lock)
  -h, --help                Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --source-root)
            source_root="${2:?missing value for --source-root}"
            shift 2
            ;;
        --audit-output)
            audit_output="${2:?missing value for --audit-output}"
            shift 2
            ;;
        --sync)
            run_sync=1
            shift
            ;;
        --allow-dirty-source)
            allow_dirty_source=1
            shift
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

if OSCALIFY_ROOT="$source_root" bash "$client_root/scripts/protocol-audit.sh" --output "$audit_output"; then
    echo "protocol maintenance: audit matched; no snapshot refresh required."
    exit 0
fi

if [[ "$run_sync" -eq 0 ]]; then
    echo "protocol maintenance: drift detected. run with --sync to refresh vendored snapshot." >&2
    exit 1
fi

sync_args=(--source-root "$source_root" --sync)
if [[ "$allow_dirty_source" -eq 1 ]]; then
    sync_args+=(--allow-dirty-source)
fi

echo "protocol maintenance: refreshing vendored snapshot from ${source_root}"
bash "$client_root/scripts/protocol-sync.sh" "${sync_args[@]}"

if OSCALIFY_ROOT="$source_root" bash "$client_root/scripts/protocol-audit.sh" --output "$audit_output"; then
    echo "protocol maintenance: snapshot refreshed and re-audited."
    exit 0
fi

echo "protocol maintenance: refresh completed but re-audit still reports drift." >&2
exit 1

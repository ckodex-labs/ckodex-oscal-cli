#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_root="${OSCALIFY_ROOT:-"$client_root/../ckodex-oscalify"}"
output="${OSCAL_PROTOCOL_AUDIT_OUTPUT:-$client_root/target/protocol-audit.json}"

usage() {
    cat <<'EOF'
usage: protocol-audit.sh [options]

Compare the client protocol lock with an external OSCALify source snapshot.
The audit is read-only and writes only its JSON report.

Options:
  --output FILE  Audit report (default: target/protocol-audit.json)
  -h, --help     Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --output)
            output="${2:?missing value for --output}"
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

mkdir -p "$(dirname "$output")"

if python3 - "$client_root" "$source_root" "$output" <<'PY'
import hashlib
import difflib
import json
import pathlib
import re
import subprocess
import sys

client_root = pathlib.Path(sys.argv[1]).resolve()
source_root = pathlib.Path(sys.argv[2]).resolve()
output = pathlib.Path(sys.argv[3]).resolve()
client_proto_root = client_root / "proto" / "oscal"
source_proto_root = source_root / "proto" / "oscal"
lock_path = client_root / "proto.lock"


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def git_lines(*args):
    try:
        result = subprocess.run(
            ["git", "-C", str(source_root), *args],
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return None
    return [line for line in result.stdout.splitlines() if line]


def git_value(*args):
    try:
        result = subprocess.run(
            ["git", "-C", str(source_root), *args],
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return None
    value = result.stdout.strip()
    return value or None


def upstream_value(block, key):
    match = re.search(
        rf'(?m)^    {re.escape(key)}:\s*"?([^"\s#]+)"?',
        block,
    )
    return match.group(1) if match else None


def read_schema_metadata():
    version_path = (
        source_root / "server" / "internal" / "oscalversion" / "VERSION"
    )
    if not version_path.is_file() or version_path.is_symlink():
        return None, [f"missing version pin: {version_path}"]
    version = version_path.read_text().strip()
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        return None, [f"invalid OSCAL version pin: {version!r}"]
    manifest_path = (
        source_root
        / "server"
        / "internal"
        / "schemavalidate"
        / "schemas"
        / f"v{version}"
        / "MANIFEST.sha256"
    )
    errors = []
    if not manifest_path.is_file() or manifest_path.is_symlink():
        return None, [f"missing schema manifest: {manifest_path}"]
    manifest_names = []
    for line_number, line in enumerate(manifest_path.read_text().splitlines(), start=1):
        if not line:
            continue
        match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9._-]+)", line)
        if not match:
            errors.append(f"invalid manifest line {line_number}: {line!r}")
            continue
        expected_digest, name = match.groups()
        schema_path = manifest_path.parent / name
        manifest_names.append(name)
        if not schema_path.is_file() or schema_path.is_symlink():
            errors.append(f"missing schema: {name}")
        elif sha256(schema_path) != expected_digest:
            errors.append(f"schema digest mismatch: {name}")
    if len(manifest_names) != 9 or "oscal_mapping_schema.json" not in manifest_names:
        errors.append(
            "manifest does not cover the complete schema plus all eight released models"
        )

    upstream_lock_path = source_root / "data" / "oscal" / "upstream.lock.yaml"
    if not upstream_lock_path.is_file() or upstream_lock_path.is_symlink():
        return None, errors + [f"missing upstream lock: {upstream_lock_path}"]
    upstream_text = upstream_lock_path.read_text()
    block_match = re.search(
        r"(?ms)^  oscal:\n(?P<body>.*?)(?=^  [A-Za-z0-9_-]+:\n|\Z)",
        upstream_text,
    )
    if not block_match:
        return None, errors + ["cannot locate sources.oscal in upstream lock"]
    block = block_match.group("body")
    metadata = {
        "version": version,
        "source_commit": upstream_value(block, "source_commit"),
        "release_zip_sha256": upstream_value(block, "release_zip_sha256"),
        "manifest_sha256": f"sha256:{sha256(manifest_path)}",
        "reconciled_at": upstream_value(block, "reconciled_at"),
    }
    if upstream_value(block, "oscal_metaschema_version") != version:
        errors.append("version pin and upstream lock disagree")
    if upstream_value(block, "parity") != "verified":
        errors.append("upstream OSCAL parity is not verified")
    if any(value is None for value in metadata.values()):
        errors.append("upstream OSCAL metadata is incomplete")
    return metadata, errors


lock = json.loads(lock_path.read_text())
locked_files = lock["files"]
source_proto_paths = sorted(
    str(path.relative_to(source_proto_root))
    for path in source_proto_root.rglob("*.proto")
)
locked_proto_paths = sorted(locked_files)
source_added_proto_files = sorted(set(source_proto_paths) - set(locked_proto_paths))
source_missing_proto_files = sorted(set(locked_proto_paths) - set(source_proto_paths))
proto_file_set_status = (
    "match"
    if not source_added_proto_files and not source_missing_proto_files
    else "source_drift"
)
expected_schema_metadata = lock.get("oscal_schema")
source_schema_metadata, schema_manifest_errors = read_schema_metadata()
if not isinstance(expected_schema_metadata, dict):
    schema_status = "client_lock_metadata_missing"
elif schema_manifest_errors:
    schema_status = "source_manifest_invalid"
elif source_schema_metadata != expected_schema_metadata:
    schema_status = "source_drift"
else:
    schema_status = "match"
file_results = []
for relative in sorted(locked_files):
    client_path = client_proto_root / relative
    source_path = source_proto_root / relative
    client_digest = sha256(client_path) if client_path.is_file() else None
    source_digest = sha256(source_path) if source_path.is_file() else None
    expected = locked_files[relative]
    if client_digest != expected:
        status = "client_lock_mismatch"
    elif source_digest is None:
        status = "source_missing"
    elif source_digest != expected:
        status = "source_drift"
    else:
        status = "match"
    unified_diff = []
    if status != "match" and client_path.is_file() and source_path.is_file():
        unified_diff = list(
            difflib.unified_diff(
                client_path.read_text(errors="replace").splitlines(),
                source_path.read_text(errors="replace").splitlines(),
                fromfile=f"client/{relative}",
                tofile=f"source/{relative}",
                lineterm="",
            )
        )
    file_results.append(
        {
            "path": relative,
            "expected_sha256": expected,
            "client_sha256": client_digest,
            "source_sha256": source_digest,
            "status": status,
            "unified_diff": unified_diff,
        }
    )

graph_proto = source_proto_root / "services" / "v1" / "transparency_graph_service.proto"
graph_text = graph_proto.read_text() if graph_proto.is_file() else ""
source_go_files = list(source_root.rglob("*.go"))
go_text = {
    path: path.read_text(errors="replace")
    for path in source_go_files
}
go_mentions_projection_event = any(
    "GraphProjectionEvent" in text for text in go_text.values()
)
proto_declares_projection_event = "message GraphProjectionEvent" in graph_text
generated_paths = {
    "messages": source_root
    / "proto/oscal/services/v1/transparency_graph_service.pb.go",
    "grpc": source_root
    / "proto/oscal/services/v1/transparency_graph_service_grpc.pb.go",
    "gateway": source_root
    / "proto/oscal/services/v1/transparency_graph_service.pb.gw.go",
}
generated_text = {
    name: path.read_text(errors="replace") if path.is_file() else ""
    for name, path in generated_paths.items()
}
generated_binding_probe = {
    "message_type": "type GraphProjectionEvent struct"
    in generated_text["messages"],
    "project_edge_event_field": "ProjectionEvent *GraphProjectionEvent"
    in generated_text["messages"],
    "rpc_method": "ListProjectionEvents(" in generated_text["grpc"],
    "http_route": "/v1/graph/projection-events" in generated_text["gateway"],
}
generated_binding_drift_suspected = (
    proto_declares_projection_event
    and not all(generated_binding_probe.values())
)

proto_diff = git_lines("diff", "--name-only", "--", "proto/oscal")
status_lines = git_lines("status", "--short", "--untracked-files=all")
source_head = git_value("rev-parse", "HEAD")
graph_proto_commit = git_value(
    "log",
    "-1",
    "--format=%H",
    "--",
    "proto/oscal/services/v1/transparency_graph_service.proto",
)
graph_proto_status = git_lines(
    "status",
    "--short",
    "--untracked-files=all",
    "--",
    "proto/oscal/services/v1/transparency_graph_service.proto",
)
drift_count = sum(result["status"] != "match" for result in file_results)
status = (
    "match"
    if drift_count == 0
    and proto_file_set_status == "match"
    and schema_status == "match"
    and not generated_binding_drift_suspected
    else "drift"
)

report = {
    "schema": "urn:oscalify:observer:protocol-audit:v1",
    "read_only": True,
    "client_root": str(client_root),
    "source_root": str(source_root),
    "client_proto_root": "proto/oscal",
    "proto_lock_sha256": sha256(lock_path),
    "status": status,
    "proto_file_count": len(file_results),
    "drift_count": drift_count,
    "proto_file_set": {
        "status": proto_file_set_status,
        "source_added": source_added_proto_files,
        "source_missing": source_missing_proto_files,
    },
    "oscal_schema": {
        "status": schema_status,
        "expected": expected_schema_metadata,
        "source": source_schema_metadata,
        "manifest_errors": schema_manifest_errors,
    },
    "source_proto_diff_files_observed": None if proto_diff is None else len(proto_diff),
    "source_status_files_observed": None if status_lines is None else len(status_lines),
    "source_revision": {
        "head": source_head,
        "worktree_dirty": None if status_lines is None else bool(status_lines),
        "graph_proto_last_committed": graph_proto_commit,
        "graph_proto_worktree_status": graph_proto_status,
    },
    "generated_binding_drift_suspected": generated_binding_drift_suspected,
    "generated_binding_probe": {
        "source_go_files_scanned": len(source_go_files),
        "go_mentions_graph_projection_event": go_mentions_projection_event,
        "source_proto_declares_graph_projection_event": proto_declares_projection_event,
        **generated_binding_probe,
    },
    "files": file_results,
}
output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
print(f"protocol audit report: {output}")
print(f"protocol status: {status}")
sys.exit(0 if status == "match" else 1)
PY
then
    exit 0
else
    status=$?
    exit "$status"
fi

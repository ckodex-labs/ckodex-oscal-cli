#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_root="${OSCALIFY_ROOT:-"$client_root/../ckodex-oscalify"}"
mode="check"
allow_dirty_source=0

usage() {
    cat <<'EOF'
usage: protocol-sync.sh [options]

Check or refresh the vendored OSCALify protobuf snapshot and OSCAL schema pin.
The OSCALify checkout is always read-only.

Options:
  --check                 Report drift without writing (default).
  --sync                  Refresh proto/oscal and proto.lock from the source.
  --source-root DIR       OSCALify checkout (default: ../ckodex-oscalify).
  --allow-dirty-source    Permit --sync from a dirty source checkout.
  -h, --help              Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --check)
            mode="check"
            shift
            ;;
        --sync)
            mode="sync"
            shift
            ;;
        --source-root)
            source_root="${2:?missing value for --source-root}"
            shift 2
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

python3 - "$client_root" "$source_root" "$mode" "$allow_dirty_source" <<'PY'
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

client_root = pathlib.Path(sys.argv[1]).resolve()
source_root = pathlib.Path(sys.argv[2]).resolve()
mode = sys.argv[3]
allow_dirty_source = sys.argv[4] == "1"
client_proto_root = client_root / "proto" / "oscal"
source_proto_root = source_root / "proto" / "oscal"
lock_path = client_root / "proto.lock"


def fail(message):
    print(f"FATAL: {message}", file=sys.stderr)
    raise SystemExit(1)


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def regular_file(path, label):
    if path.is_symlink() or not path.is_file():
        fail(f"{label} is not a regular file: {path}")


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
    return result.stdout.strip() or None


regular_file(lock_path, "protocol lock")
try:
    current_lock = json.loads(lock_path.read_text())
except (OSError, json.JSONDecodeError) as error:
    fail(f"cannot read protocol lock: {error}")
locked_files = current_lock.get("files")
if not isinstance(locked_files, dict) or not locked_files:
    fail("proto.lock must contain a non-empty files object")

expected_paths = sorted(locked_files)
source_paths = sorted(
    str(path.relative_to(source_proto_root))
    for path in source_proto_root.rglob("*.proto")
)
if source_paths != expected_paths:
    missing = sorted(set(expected_paths) - set(source_paths))
    added = sorted(set(source_paths) - set(expected_paths))
    fail(
        "source proto file set changed; update build.rs and the reviewed allowlist "
        f"before syncing (missing={missing}, added={added})"
    )

client_errors = []
source_digests = {}
for relative in expected_paths:
    client_path = client_proto_root / relative
    source_path = source_proto_root / relative
    regular_file(client_path, "vendored proto")
    regular_file(source_path, "source proto")
    client_digest = sha256(client_path)
    expected_digest = locked_files[relative]
    if client_digest != expected_digest:
        client_errors.append(
            f"{relative}: client={client_digest} lock={expected_digest}"
        )
    source_digests[relative] = sha256(source_path)
if client_errors:
    fail("vendored snapshot has local drift:\n  " + "\n  ".join(client_errors))

version_path = source_root / "server" / "internal" / "oscalversion" / "VERSION"
regular_file(version_path, "OSCAL version pin")
oscal_version = version_path.read_text().strip()
if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", oscal_version):
    fail(f"invalid OSCAL version pin: {oscal_version!r}")

schema_root = (
    source_root
    / "server"
    / "internal"
    / "schemavalidate"
    / "schemas"
    / f"v{oscal_version}"
)
manifest_path = schema_root / "MANIFEST.sha256"
regular_file(manifest_path, "OSCAL schema manifest")
manifest_entries = []
for line_number, line in enumerate(manifest_path.read_text().splitlines(), start=1):
    if not line:
        continue
    match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9._-]+)", line)
    if not match:
        fail(f"invalid schema manifest line {line_number}: {line!r}")
    expected_digest, name = match.groups()
    schema_path = schema_root / name
    regular_file(schema_path, "OSCAL schema")
    actual_digest = sha256(schema_path)
    if actual_digest != expected_digest:
        fail(
            f"schema digest mismatch for {name}: "
            f"expected {expected_digest}, got {actual_digest}"
        )
    manifest_entries.append(name)
if "oscal_mapping_schema.json" not in manifest_entries or len(manifest_entries) != 9:
    fail(
        "OSCAL schema manifest must cover the complete schema plus all eight "
        f"released models; observed {manifest_entries}"
    )

upstream_lock_path = source_root / "data" / "oscal" / "upstream.lock.yaml"
regular_file(upstream_lock_path, "OSCAL upstream lock")
upstream_text = upstream_lock_path.read_text()
block_match = re.search(
    r"(?ms)^  oscal:\n(?P<body>.*?)(?=^  [A-Za-z0-9_-]+:\n|\Z)",
    upstream_text,
)
if not block_match:
    fail("cannot locate sources.oscal in the upstream lock")
oscal_block = block_match.group("body")


def upstream_value(key):
    match = re.search(rf'(?m)^    {re.escape(key)}:\s*"?([^"\s#]+)"?', oscal_block)
    if not match:
        fail(f"missing {key} in the OSCAL upstream lock")
    return match.group(1)


upstream_version = upstream_value("oscal_metaschema_version")
source_commit = upstream_value("source_commit")
release_zip_sha256 = upstream_value("release_zip_sha256")
reconciled_at = upstream_value("reconciled_at")
parity = upstream_value("parity")
if upstream_version != oscal_version:
    fail(
        "OSCAL version pin and upstream lock disagree: "
        f"{oscal_version} != {upstream_version}"
    )
if not re.fullmatch(r"[0-9a-f]{40}", source_commit):
    fail(f"invalid OSCAL source commit: {source_commit}")
if not re.fullmatch(r"sha256:[0-9a-f]{64}", release_zip_sha256):
    fail(f"invalid OSCAL release ZIP digest: {release_zip_sha256}")
if parity != "verified":
    fail(f"OSCAL upstream parity is not verified: {parity}")

source_head = git_value("rev-parse", "HEAD")
source_status = git_value("status", "--short", "--untracked-files=all")
source_dirty = source_status is not None and bool(source_status)
if mode == "sync" and source_dirty and not allow_dirty_source:
    fail(
        "source checkout is dirty; review it and pass --allow-dirty-source "
        "to record an intentional workspace snapshot"
    )

schema_metadata = {
    "version": oscal_version,
    "source_commit": source_commit,
    "release_zip_sha256": release_zip_sha256,
    "manifest_sha256": f"sha256:{sha256(manifest_path)}",
    "reconciled_at": reconciled_at,
}
proto_drift = [
    relative
    for relative in expected_paths
    if source_digests[relative] != locked_files[relative]
]
schema_drift = current_lock.get("oscal_schema") != schema_metadata
drift = bool(proto_drift or schema_drift)

if mode == "check":
    print(f"protocol sync status: {'drift' if drift else 'match'}")
    print(f"OSCAL schema version: {oscal_version}")
    print(f"schema manifest: {schema_metadata['manifest_sha256']}")
    print(f"drifted proto files: {len(proto_drift)}")
    for relative in proto_drift:
        print(f"  {relative}")
    raise SystemExit(1 if drift else 0)

new_lock = {
    "schema": "urn:oscalify:observer:proto-lock:v2",
    "source": "../ckodex-oscalify/proto/oscal",
    "mode": "workspace-read-only",
    "note": (
        "Hashes describe an intentionally reviewed source snapshot. "
        "The source checkout is never modified by this client."
    ),
    "source_revision": {
        "head": source_head,
        "worktree_dirty": source_dirty,
    },
    "oscal_schema": schema_metadata,
    "files": source_digests,
}

stage_root = pathlib.Path(tempfile.mkdtemp(prefix=".protocol-sync.", dir=client_root))
stage_proto = stage_root / "oscal"
stage_lock = stage_root / "proto.lock"
backup_proto = stage_root / "previous-oscal"
try:
    for relative in expected_paths:
        destination = stage_proto / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source_proto_root / relative, destination)
        if sha256(destination) != source_digests[relative]:
            fail(f"staged proto digest mismatch: {relative}")
    stage_lock.write_text(json.dumps(new_lock, indent=2) + "\n")

    os.replace(client_proto_root, backup_proto)
    try:
        os.replace(stage_proto, client_proto_root)
        os.replace(stage_lock, lock_path)
    except BaseException:
        if client_proto_root.exists():
            shutil.rmtree(client_proto_root)
        os.replace(backup_proto, client_proto_root)
        raise
    shutil.rmtree(backup_proto)
finally:
    shutil.rmtree(stage_root, ignore_errors=True)

print("protocol sync status: updated")
print(f"OSCAL schema version: {oscal_version}")
print(f"schema manifest: {schema_metadata['manifest_sha256']}")
print(f"refreshed proto files: {len(expected_paths)}")
print(f"changed proto files: {len(proto_drift)}")
PY

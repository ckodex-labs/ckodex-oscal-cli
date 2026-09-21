#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
version="$(awk -F'"' '/^version = / { print $2; exit }' "$client_root/Cargo.toml")"
host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"
client_bin="${OSCAL_CLI_BIN:-}"
output=""
signing_key=""
signing_bundle=""

usage() {
    cat <<'EOF'
usage: release-bundle.sh --output FILE [options]

Build a deterministic, read-only OSCAL CLI beta archive. The archive contains
the release binary, operator documentation, protocol provenance, a bundle
manifest, and SHA-256 checksums. Release signing is optional and signs the
adjacent release-provenance.json, never a private key in this checkout.

Options:
  --output FILE       Archive path (required; existing files are rejected)
  --signing-key FILE  Sign adjacent release-provenance.json with cosign
  --bundle FILE       Sigstore bundle output (required with --signing-key)
  -h, --help          Show this help
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
            signing_bundle="${2:?missing value for --bundle}"
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
if [[ -n "$signing_key" && -z "$signing_bundle" ]]; then
    echo "--signing-key requires --bundle" >&2
    exit 2
fi
if [[ -n "$signing_bundle" && -z "$signing_key" ]]; then
    echo "--bundle requires --signing-key" >&2
    exit 2
fi

mkdir -p "$(dirname "$output")"
output_dir="$(cd "$(dirname "$output")" && pwd)"
output="$output_dir/$(basename "$output")"
if [[ -e "$output" || -e "$output.sha256" || -e "$output.release-provenance.json" ]]; then
    echo "refusing to overwrite an existing release artifact near: $output" >&2
    exit 1
fi

if [[ ! -x "$client_bin" ]]; then
    (cd "$client_root" && cargo build --quiet --release --bin oscal-cli)
    if resolved_bin="$("$client_root/scripts/cargo-binary-path.sh" release)"; then
        client_bin="$resolved_bin"
    fi
fi
if [[ ! -x "$client_bin" ]]; then
    echo "release binary not found: $client_bin" >&2
    exit 1
fi

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/oscal-cli-release-bundle.XXXXXX")"
tmp_archive="$tmp_root/$(basename "$output")"
trap 'rm -rf "$tmp_root"' EXIT

provenance="$tmp_root/protocol-provenance.json"
OSCAL_CLI_BIN="$client_bin" "$client_root/scripts/proto-provenance.sh" \
    --output "$provenance" >/dev/null

package_name="oscal-cli-v${version}-${host_triple}"
package_root="$tmp_root/$package_name"
mkdir -p "$package_root"
install -m 0755 "$client_bin" "$package_root/oscal-cli"
install -m 0644 "$client_root/README.md" "$package_root/README.md"
install -m 0644 "$client_root/ORCHESTRATOR.md" "$package_root/ORCHESTRATOR.md"
install -m 0644 "$client_root/BETA_PLAN.md" "$package_root/BETA_PLAN.md"
install -m 0644 "$client_root/PROTOCOL_ALIGNMENT.md" "$package_root/PROTOCOL_ALIGNMENT.md"
install -m 0644 "$client_root/BETA_ACCEPTANCE.md" "$package_root/BETA_ACCEPTANCE.md"
install -m 0644 "$provenance" "$package_root/protocol-provenance.json"
mkdir -p "$package_root/completions"
for shell in bash elvish fish powershell zsh; do
    "$client_bin" completions "$shell" >"$package_root/completions/oscal-cli.$shell"
done

python3 - "$package_root" "$package_name" "$version" "$host_triple" <<'PY'
import hashlib
import json
import pathlib
import sys

package_root = pathlib.Path(sys.argv[1])
package_name = sys.argv[2]
version = sys.argv[3]
target = sys.argv[4]

def digest(path):
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()

def checksum_digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

protocol = json.loads((package_root / "protocol-provenance.json").read_text())
files = []
for path in sorted(package_root.rglob("*")):
    if not path.is_file() or path.name == "bundle-manifest.json":
        continue
    relative = path.relative_to(package_root).as_posix()
    files.append(
        {
            "path": relative,
            "size_bytes": path.stat().st_size,
            "sha256": digest(path),
        }
    )

manifest = {
    "schema": "urn:oscalify:observer:release-bundle:v1",
    "name": package_name,
    "version": version,
    "target": target,
    "read_only": True,
    "protocol": {
        "descriptor_sha256": protocol["descriptor_sha256"],
        "proto_lock_sha256": protocol["proto_lock_sha256"],
        "proto_file_count": protocol["proto_file_count"],
        "source": protocol["source"],
    },
    "files": files,
}
(package_root / "bundle-manifest.json").write_text(
    json.dumps(manifest, indent=2, sort_keys=True) + "\n"
)

checksum_lines = []
for path in sorted(package_root.rglob("*")):
    if not path.is_file() or path.name == "SHA256SUMS":
        continue
    checksum_lines.append(
        f"{checksum_digest(path)}  {path.relative_to(package_root).as_posix()}"
    )
(package_root / "SHA256SUMS").write_text("\n".join(checksum_lines) + "\n")
PY

python3 - "$package_root" "$tmp_archive" <<'PY'
import gzip
import io
import pathlib
import sys
import tarfile

package_root = pathlib.Path(sys.argv[1])
archive = pathlib.Path(sys.argv[2])
with archive.open("wb") as raw:
    with gzip.GzipFile(fileobj=raw, mode="wb", compresslevel=9, mtime=0, filename="") as gz:
        with tarfile.open(fileobj=gz, mode="w", format=tarfile.USTAR_FORMAT) as tar:
            for path in sorted(package_root.rglob("*")):
                if not path.is_file():
                    continue
                data = path.read_bytes()
                relative = path.relative_to(package_root).as_posix()
                info = tarfile.TarInfo(f"{package_root.name}/{relative}")
                info.size = len(data)
                info.mtime = 0
                info.uid = 0
                info.gid = 0
                info.uname = ""
                info.gname = ""
                info.mode = 0o755 if relative == "oscal-cli" else 0o644
                tar.addfile(info, io.BytesIO(data))
PY

mv "$tmp_archive" "$output"

python3 - "$output" "$output.sha256" "$output.release-provenance.json" "$package_root" "$package_name" "$version" "$host_triple" <<'PY'
import hashlib
import json
import pathlib
import sys

archive = pathlib.Path(sys.argv[1])
checksum_path = pathlib.Path(sys.argv[2])
provenance_path = pathlib.Path(sys.argv[3])
package_root = pathlib.Path(sys.argv[4])
package_name = sys.argv[5]
version = sys.argv[6]
target = sys.argv[7]

def digest(path):
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()

def checksum_digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

manifest = package_root / "bundle-manifest.json"
protocol = json.loads((package_root / "protocol-provenance.json").read_text())
checksum_path.write_text(f"{checksum_digest(archive)}  {archive.name}\n")
provenance = {
    "schema": "urn:oscalify:observer:release-provenance:v1",
    "artifact": {
        "name": archive.name,
        "size_bytes": archive.stat().st_size,
        "sha256": digest(archive),
    },
    "bundle": {
        "name": package_name,
        "version": version,
        "target": target,
        "manifest_sha256": digest(manifest),
        "protocol_descriptor_sha256": protocol["descriptor_sha256"],
        "proto_lock_sha256": protocol["proto_lock_sha256"],
    },
    "read_only": True,
    "signing": "unsigned_local_artifact",
}
provenance_path.write_text(json.dumps(provenance, indent=2, sort_keys=True) + "\n")
PY

python3 - "$output" "$output.sha256" <<'PY'
import hashlib
import pathlib
import sys
import tarfile

archive = pathlib.Path(sys.argv[1])
checksum_path = pathlib.Path(sys.argv[2])

expected_archive, archive_name = checksum_path.read_text().strip().split("  ", 1)
actual_archive = hashlib.sha256(archive.read_bytes()).hexdigest()
if archive_name != archive.name or expected_archive != actual_archive:
    raise SystemExit("release archive checksum verification failed")

with tarfile.open(archive, "r:gz") as handle:
    sum_members = [member for member in handle.getmembers() if member.name.endswith("/SHA256SUMS")]
    if len(sum_members) != 1:
        raise SystemExit("release archive must contain exactly one SHA256SUMS file")
    sum_member = sum_members[0]
    root = sum_member.name.rsplit("/", 1)[0]
    sum_text = handle.extractfile(sum_member).read().decode()
    checked = 0
    for line in sum_text.splitlines():
        expected, relative = line.split("  ", 1)
        member = handle.extractfile(f"{root}/{relative}")
        if member is None:
            raise SystemExit(f"release archive checksum target is missing: {relative}")
        actual = hashlib.sha256(member.read()).hexdigest()
        if actual != expected:
            raise SystemExit(f"release archive member checksum failed: {relative}")
        checked += 1
    if checked == 0:
        raise SystemExit("release archive contains no embedded checksum entries")

print(f"release bundle verified: archive and {checked} embedded files")
PY

if [[ -n "$signing_key" ]]; then
    if ! command -v cosign >/dev/null 2>&1; then
        echo "cosign is required for release provenance signing" >&2
        exit 1
    fi
    COSIGN_PASSWORD="${COSIGN_PASSWORD:-}" cosign sign-blob \
        --key "$signing_key" --bundle "$signing_bundle" \
        "$output.release-provenance.json" >/dev/null
fi

printf 'release bundle written: %s\n' "$output"
printf 'release checksum: %s\n' "$output.sha256"
printf 'release provenance: %s\n' "$output.release-provenance.json"
if [[ -n "$signing_bundle" ]]; then
    printf 'release provenance bundle: %s\n' "$signing_bundle"
fi

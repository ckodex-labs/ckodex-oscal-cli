#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
version="$(awk -F'"' '/^version = / { print $2; exit }' "$client_root/Cargo.toml")"
host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"
readiness="${OSCAL_BETA_READINESS_OUTPUT:-$client_root/target/beta-readiness.json}"
evidence_root="${OSCAL_BETA_READINESS_EVIDENCE_DIR:-$client_root/target/beta-readiness-evidence}"
coverage_root="$client_root/target/coverage-smoke"
output="${OSCAL_BETA_HANDOFF_OUTPUT:-$client_root/target/oscal-cli-beta-handoff.tar.gz}"

usage() {
    cat <<'EOF'
usage: beta-handoff.sh [options]

Build a deterministic, read-only beta review archive from an existing local
readiness run. The archive contains gate evidence, protocol drift evidence,
coverage, source-custody snapshots, release artifacts, and the beta plan.

Options:
  --readiness FILE    Readiness manifest (default: target/beta-readiness.json)
  --evidence-dir DIR  Readiness evidence directory
  --output FILE       Handoff archive (default: target/oscal-cli-beta-handoff.tar.gz)
  -h, --help          Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --readiness)
            readiness="${2:?missing value for --readiness}"
            shift 2
            ;;
        --evidence-dir)
            evidence_root="${2:?missing value for --evidence-dir}"
            shift 2
            ;;
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

if [[ ! -f "$readiness" || ! -d "$evidence_root" || ! -d "$coverage_root" ]]; then
    echo "readiness, evidence, and coverage artifacts are required" >&2
    exit 1
fi

mkdir -p "$(dirname "$output")"
output_dir="$(cd "$(dirname "$output")" && pwd)"
output="$output_dir/$(basename "$output")"
if [[ -e "$output" || -e "$output.sha256" || -e "$output.provenance.json" ]]; then
    echo "refusing to overwrite an existing handoff artifact near: $output" >&2
    exit 1
fi

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/oscal-cli-beta-handoff.XXXXXX")"
tmp_archive="$tmp_root/$(basename "$output")"
trap 'rm -rf "$tmp_root"' EXIT

python3 - "$client_root" "$readiness" "$evidence_root" "$coverage_root" "$tmp_root" "$version" "$host_triple" <<'PY'
import hashlib
import json
import pathlib
import shutil
import sys

client_root = pathlib.Path(sys.argv[1]).resolve()
readiness_path = pathlib.Path(sys.argv[2]).resolve()
evidence_root = pathlib.Path(sys.argv[3]).resolve()
coverage_root = pathlib.Path(sys.argv[4]).resolve()
tmp_root = pathlib.Path(sys.argv[5]).resolve()
version = sys.argv[6]
target = sys.argv[7]

readiness = json.loads(readiness_path.read_text())
if not readiness["verdict"]["local_gates_passed"]:
    raise SystemExit("cannot build a beta handoff from failed local gates")

package_name = f"oscal-cli-beta-handoff-v{version}-{target}"
package_root = tmp_root / package_name
package_root.mkdir(parents=True)

def copy_file(source, relative):
    destination = package_root / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)

copy_file(readiness_path, pathlib.Path("beta-readiness.json"))
for name in ("README.md", "ORCHESTRATOR.md", "BETA_PLAN.md", "PROTOCOL_ALIGNMENT.md", "BETA_ACCEPTANCE.md"):
    copy_file(client_root / name, pathlib.Path(name))

shutil.copytree(evidence_root, package_root / "readiness-evidence")
coverage_destination = package_root / "coverage"
coverage_destination.mkdir()
for name in ("summary.txt", "functions-under-80.txt"):
    source = coverage_root / name
    if source.is_file():
        shutil.copyfile(source, coverage_destination / name)

def digest(path):
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()

files = []
for path in sorted(package_root.rglob("*")):
    if path.is_file() and path.name not in {"handoff-manifest.json", "SHA256SUMS"}:
        files.append(
            {
                "path": path.relative_to(package_root).as_posix(),
                "size_bytes": path.stat().st_size,
                "sha256": digest(path),
            }
        )

manifest = {
    "schema": "urn:oscalify:observer:beta-handoff:v1",
    "name": package_name,
    "version": version,
    "target": target,
    "read_only": True,
    "verdict": readiness["verdict"],
    "files": files,
}
(package_root / "handoff-manifest.json").write_text(
    json.dumps(manifest, indent=2, sort_keys=True) + "\n"
)

checksum_lines = []
for path in sorted(package_root.rglob("*")):
    if path.is_file() and path.name != "SHA256SUMS":
        checksum_lines.append(
            f"{hashlib.sha256(path.read_bytes()).hexdigest()}  "
            f"{path.relative_to(package_root).as_posix()}"
        )
(package_root / "SHA256SUMS").write_text("\n".join(checksum_lines) + "\n")
PY

python3 - "$tmp_root" "$tmp_archive" <<'PY'
import gzip
import io
import pathlib
import sys
import tarfile

package_root = next(path for path in pathlib.Path(sys.argv[1]).iterdir() if path.is_dir())
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
                info.mode = 0o644
                tar.addfile(info, io.BytesIO(data))
PY

mv "$tmp_archive" "$output"

python3 - "$output" "$output.sha256" "$output.provenance.json" <<'PY'
import hashlib
import json
import pathlib
import sys
import tarfile

archive = pathlib.Path(sys.argv[1])
checksum_path = pathlib.Path(sys.argv[2])
provenance_path = pathlib.Path(sys.argv[3])

def digest(path):
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()

checksum_path.write_text(f"{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}\n")
with tarfile.open(archive, "r:gz") as handle:
    sums = [member for member in handle.getmembers() if member.name.endswith("/SHA256SUMS")]
    if len(sums) != 1:
        raise SystemExit("handoff must contain exactly one SHA256SUMS file")
    sum_member = sums[0]
    root = sum_member.name.rsplit("/", 1)[0]
    checked = 0
    for line in handle.extractfile(sum_member).read().decode().splitlines():
        expected, relative = line.split("  ", 1)
        member = handle.extractfile(f"{root}/{relative}")
        if member is None or hashlib.sha256(member.read()).hexdigest() != expected:
            raise SystemExit(f"handoff checksum failed: {relative}")
        checked += 1
provenance_path.write_text(
    json.dumps(
        {
            "schema": "urn:oscalify:observer:beta-handoff-provenance:v1",
            "artifact": {"name": archive.name, "sha256": digest(archive)},
            "embedded_files_verified": checked,
            "read_only": True,
            "signing": "unsigned_local_artifact",
        },
        indent=2,
        sort_keys=True,
    )
    + "\n"
)
print(f"beta handoff verified: archive and {checked} embedded files")
PY

printf 'beta handoff written: %s\n' "$output"
printf 'beta handoff checksum: %s\n' "$output.sha256"
printf 'beta handoff provenance: %s\n' "$output.provenance.json"

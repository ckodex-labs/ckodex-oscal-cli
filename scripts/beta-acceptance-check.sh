#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
record="${OSCAL_BETA_ACCEPTANCE_RECORD:-$client_root/BETA_ACCEPTANCE.md}"
readiness="${OSCAL_BETA_READINESS_OUTPUT:-$client_root/target/beta-readiness.json}"
evidence_root="${OSCAL_BETA_READINESS_EVIDENCE_DIR:-$client_root/target/beta-readiness-evidence}"
output=""

usage() {
    cat <<'EOF'
usage: beta-acceptance-check.sh [options]

Validate the human beta acceptance record without changing it or the OSCALify
checkout. The check passes only when local readiness is green and all four
external dispositions are ACCEPT with owner, date, and evidence reference.

Options:
  --record FILE       Acceptance record (default: BETA_ACCEPTANCE.md)
  --readiness FILE    Readiness manifest (default: target/beta-readiness.json)
  --evidence-dir DIR  Readiness evidence directory
  --output FILE       Write a JSON check report
  -h, --help          Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --record)
            record="${2:?missing value for --record}"
            shift 2
            ;;
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

if [[ ! -f "$record" || ! -f "$readiness" ]]; then
    echo "acceptance record and readiness manifest are required" >&2
    exit 1
fi

python3 - "$client_root" "$record" "$readiness" "$evidence_root" "$output" <<'PY'
import datetime
import json
import pathlib
import re
import sys

client_root = pathlib.Path(sys.argv[1]).resolve()
record_path = pathlib.Path(sys.argv[2]).resolve()
readiness_path = pathlib.Path(sys.argv[3]).resolve()
evidence_root = pathlib.Path(sys.argv[4]).resolve()
output_arg = sys.argv[5]

record_text = record_path.read_text()
readiness = json.loads(readiness_path.read_text())

gate_specs = [
    ("source-ownership-compatibility", "Source ownership and compatibility"),
    ("release-signing-provenance", "Release signing and provenance"),
    ("upstream-authorization", "Upstream authorization"),
    ("capture-retention-redaction", "Capture retention and redaction"),
]
sections = {}
current = None
for line in record_text.splitlines():
    if line.startswith("### "):
        current = line[4:].strip()
        sections[current] = []
    elif current is not None:
        sections[current].append(line)

def field(lines, label):
    prefix = label + ":"
    for line in lines:
        if line.startswith(prefix):
            return line[len(prefix):].strip()
    return ""

def supplied(value):
    return bool(value) and not set(value) <= {"_", " ", "."}

gates = []
for gate_id, heading in gate_specs:
    lines = sections.get(heading, [])
    disposition = field(lines, "Disposition")
    owner_line = field(lines, "Owner")
    evidence_reference = field(lines, "Evidence reference")
    owner = owner_line
    date = ""
    if "Date:" in owner_line:
        owner, date = owner_line.split("Date:", 1)
        owner = owner.strip()
        date = date.strip()
    accepted = bool(re.search(r"\[\s*[xX]\s*\]\s*ACCEPT\b", disposition))
    rejected = bool(re.search(r"\[\s*[xX]\s*\]\s*REJECT\b", disposition))
    deferred = bool(re.search(r"\[\s*[xX]\s*\]\s*DEFER\b", disposition))
    issues = []
    if accepted:
        if not supplied(owner):
            issues.append("owner missing")
        if not supplied(date):
            issues.append("date missing")
        if not supplied(evidence_reference):
            issues.append("evidence reference missing")
        status = "accepted" if not issues else "invalid_acceptance"
    elif rejected:
        status = "rejected"
    elif deferred:
        status = "deferred"
    else:
        status = "pending"
    gates.append(
        {
            "id": gate_id,
            "heading": heading,
            "status": status,
            "owner_supplied": supplied(owner),
            "date_supplied": supplied(date),
            "evidence_reference_supplied": supplied(evidence_reference),
            "issues": issues,
        }
    )

local_ready = bool(readiness.get("verdict", {}).get("local_gates_passed"))
all_accepted = local_ready and all(gate["status"] == "accepted" for gate in gates)
if not local_ready:
    status = "local_gate_failed"
elif all_accepted:
    status = "accepted"
else:
    status = "pending_human_review"

report = {
    "schema": "urn:oscalify:observer:beta-acceptance-check:v1",
    "generated_at_utc": datetime.datetime.now(datetime.timezone.utc)
    .replace(microsecond=0)
    .isoformat()
    .replace("+00:00", "Z"),
    "record": str(record_path),
    "readiness": str(readiness_path),
    "evidence_root": str(evidence_root),
    "status": status,
    "promotion_allowed": status == "accepted",
    "gates": gates,
}

serialized = json.dumps(report, indent=2, sort_keys=True) + "\n"
if output_arg:
    output_path = pathlib.Path(output_arg).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(serialized)
print(serialized, end="")
raise SystemExit(0 if status == "accepted" else 1)
PY

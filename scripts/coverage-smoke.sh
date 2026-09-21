#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
coverage_root="$client_root/target/coverage-smoke"
run_dir=""
min_region_coverage="${OSCAL_COVERAGE_MIN_REGIONS:-80.0}"
min_line_coverage="${OSCAL_COVERAGE_MIN_LINES:-80.0}"

cleanup() {
    if [[ "${OSCAL_COVERAGE_KEEP:-0}" != "1" && -n "$run_dir" && -d "$run_dir" ]]; then
        rm -rf "$run_dir"
    fi
}
trap cleanup EXIT

mkdir -p "$coverage_root"
run_dir="$(mktemp -d "$coverage_root/run.XXXXXX")"

eval "$(cd "$client_root" && cargo llvm-cov show-env --export-prefix 2>/dev/null)"
export CARGO_TARGET_DIR="$run_dir/target"
export CARGO_LLVM_COV_TARGET_DIR="$run_dir/target"
export LLVM_PROFILE_FILE="$run_dir/oscal-cli-%p-%m.profraw"

(cd "$client_root" && cargo build --quiet --bin oscal-cli)
host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"
client_bin="$run_dir/target/$host_triple/debug/oscal-cli"
[[ -x "$client_bin" ]]

OSCALIFY_E2E_FIXTURES=1 OSCALIFY_REGENERATE_PROTO=1 \
    OSCAL_CLI_BIN="$client_bin" "$client_root/scripts/e2e-smoke.sh"

if command -v llvm-profdata >/dev/null 2>&1 && command -v llvm-cov >/dev/null 2>&1; then
    llvm_profdata="$(command -v llvm-profdata)"
    llvm_cov="$(command -v llvm-cov)"
elif command -v xcrun >/dev/null 2>&1; then
    llvm_profdata="$(xcrun --find llvm-profdata)"
    llvm_cov="$(xcrun --find llvm-cov)"
else
    echo "llvm-profdata and llvm-cov are required for coverage-smoke" >&2
    exit 1
fi

profiles=()
while IFS= read -r profile; do
    profiles+=("$profile")
done < <(find "$run_dir" -type f -name '*.profraw' -print)
if (( ${#profiles[@]} == 0 )); then
    echo "coverage-smoke produced no profile data" >&2
    exit 1
fi

profile_data="$run_dir/oscal-cli.profdata"
"$llvm_profdata" merge -sparse "${profiles[@]}" -o "$profile_data"
summary_path="$coverage_root/summary.txt"
"$llvm_cov" report "$client_bin" \
    -instr-profile="$profile_data" \
    -ignore-filename-regex='(/rustc/|/\.cargo/|/target/.*/build/oscal-cli-.*\.rs)' \
    | tee "$summary_path"

python3 - "$summary_path" "$min_region_coverage" "$min_line_coverage" <<'PY'
import pathlib
import re
import sys

summary_path = pathlib.Path(sys.argv[1])
min_regions = float(sys.argv[2])
min_lines = float(sys.argv[3])
total = next(
    (line for line in summary_path.read_text().splitlines() if line.startswith("TOTAL ")),
    None,
)
if total is None:
    raise SystemExit("coverage report has no TOTAL row")

percentages = re.findall(r"(\d+(?:\.\d+)?)%", total)
if len(percentages) < 3:
    raise SystemExit("coverage TOTAL row has no region, function, and line percentages")

regions = float(percentages[0])
lines = float(percentages[2])
if regions < min_regions or lines < min_lines:
    raise SystemExit(
        f"coverage threshold failed: regions={regions:.2f}% (min {min_regions:.2f}%), "
        f"lines={lines:.2f}% (min {min_lines:.2f}%)"
    )
print(
    f"coverage threshold passed: regions={regions:.2f}% >= {min_regions:.2f}%, "
    f"lines={lines:.2f}% >= {min_lines:.2f}%"
)
PY

function_summary_path="$coverage_root/functions-under-80.txt"
source_files=()
while IFS= read -r source_file; do
    source_files+=("$source_file")
done < <(find "$client_root/src" -type f -name '*.rs' -print)
# llvm-cov returns non-zero when this informational filter finds functions below
# 80%. The aggregate region/line threshold above is the release gate; retain
# the diagnostic report without allowing its findings to masquerade as a gate
# failure.
if ! "$llvm_cov" report "$client_bin" \
    -instr-profile="$profile_data" \
    -ignore-filename-regex='(/rustc/|/\.cargo/|/target/.*/build/oscal-cli-.*\.rs)' \
    -show-functions \
    -region-coverage-lt=80 \
    "${source_files[@]}" \
    >"$function_summary_path"; then
    printf '%s\n' 'function detail contains below-threshold functions; aggregate threshold remains authoritative' >&2
fi
printf '%s\n' "coverage summary: $summary_path"
printf '%s\n' "function detail: $function_summary_path"
if [[ "${OSCAL_COVERAGE_KEEP:-0}" == "1" ]]; then
    printf '%s\n' "coverage run retained: $run_dir"
fi

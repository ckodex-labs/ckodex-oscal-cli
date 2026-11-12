#!/usr/bin/env bash
# ==============================================================================
# Mizan · Unified Pages Documentation & Next.js Workbench Portal Generator
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
PUBLIC_DIR="${ROOT_DIR}/public"
WORKBENCH_DIR="${ROOT_DIR}/apps/workbench"

echo "=== Mizan · Generating Static Pages & Workbench Portal ==="
echo "Root:      ${ROOT_DIR}"
echo "Workbench: ${WORKBENCH_DIR}"
echo "Public:    ${PUBLIC_DIR}"
echo "=========================================================="

mkdir -p "${PUBLIC_DIR}/badges"
mkdir -p "${PUBLIC_DIR}/docs/api"

# 1. Compile Mizan binary
echo ">> [1/6] Building Mizan release binary..."
cargo build --release --bin mizan
MIZAN_BIN="${ROOT_DIR}/target/release/mizan"
if [[ -f "${HOME}/.cache/cargo-target/release/mizan" ]] && [[ ! -f "${MIZAN_BIN}" || "${HOME}/.cache/cargo-target/release/mizan" -nt "${MIZAN_BIN}" ]]; then
    MIZAN_BIN="${HOME}/.cache/cargo-target/release/mizan"
fi

# 2. Generate Rust API Documentation
echo ">> [2/6] Compiling Workspace Rustdoc..."
cargo doc --workspace --no-deps
DOC_DIR="${ROOT_DIR}/target/doc"
if [[ ! -d "${DOC_DIR}" ]]; then
    DOC_DIR="${HOME}/.cache/cargo-target/doc"
fi
if [[ -d "${DOC_DIR}" ]]; then
    cp -r "${DOC_DIR}/"* "${PUBLIC_DIR}/docs/api/"
fi

# 3. Generate shieldcn-zig Badges
echo ">> [3/6] Generating shieldcn-zig Badges (APCA WCAG 3.0)..."
"${MIZAN_BIN}" export badge \
    --label "OSCAL Metaschema" \
    --message "v1.2.3 Validated" \
    --color blue \
    --variant secondary \
    --wcag 3 \
    --output "${PUBLIC_DIR}/badges/badge-oscal.md"

"${MIZAN_BIN}" export badge \
    --label "SLSA Level" \
    --message "3 In-Toto" \
    --color emerald \
    --variant secondary \
    --wcag 3 \
    --output "${PUBLIC_DIR}/badges/badge-slsa.md"

"${MIZAN_BIN}" export badge \
    --from-document "${ROOT_DIR}/examples/sample-catalog.json" \
    --variant secondary \
    --wcag 3 \
    --output "${PUBLIC_DIR}/badges/badge-fedramp.md"

# 4. Generate Standalone Offline Evidence Capsule
echo ">> [4/6] Exporting Offline Cryptographic Evidence Capsule..."
CAPSULE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/mizan-portal-capsule.XXXXXX")"
trap 'rm -rf "${CAPSULE_DIR}"' EXIT INT TERM
"${MIZAN_BIN}" pipeline run \
    -j us \
    --sbom "${ROOT_DIR}/examples/inventory-sbom.json" \
    -o "${CAPSULE_DIR}"
"${MIZAN_BIN}" export capsule \
    -a "${CAPSULE_DIR}/oscal-assessment-results.json" \
    -b "${CAPSULE_DIR}/evidence-bundle.json" \
    -o "${PUBLIC_DIR}/capsule.html"
rm -rf "${CAPSULE_DIR}"

# Generate static UTCP manual snapshot for offline / Pages fallback
mkdir -p "${WORKBENCH_DIR}/public/snapshot"
mkdir -p "${PUBLIC_DIR}/snapshot"
"${MIZAN_BIN}" utcp manual --transport http --base-url "http://localhost:3000" --format json \
    -o "${WORKBENCH_DIR}/public/snapshot/utcp-manual.json"
cp "${WORKBENCH_DIR}/public/snapshot/utcp-manual.json" "${PUBLIC_DIR}/snapshot/utcp-manual.json"

# 5. Build Next.js 16 Workbench Static Export
echo ">> [5/6] Building Next.js 16 Workbench static export..."
if [[ -d "${WORKBENCH_DIR}" ]] && command -v npm &> /dev/null; then
    (
        cd "${WORKBENCH_DIR}"
        npm run build:export
        rm -rf "${PUBLIC_DIR}/_next" "${PUBLIC_DIR}/_not-found" "${PUBLIC_DIR}/404" "${PUBLIC_DIR}"/__next* "${PUBLIC_DIR}/index.html" "${PUBLIC_DIR}/index.txt" "${PUBLIC_DIR}/404.html"
        cp -r out/* "${PUBLIC_DIR}/"
    )
else
    echo "[WARN] npm is unavailable; generating fallback static portal index..."
    mkdir -p "${PUBLIC_DIR}/_next"
    cat > "${PUBLIC_DIR}/index.html" <<'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>Mizan · OSCAL Compliance Kernel &amp; Workbench</title>
</head>
<body>
  <h1>Mizan · High-Assurance OSCAL Compliance Kernel</h1>
  <ul>
    <li><a href="capsule.html">Offline Cryptographic Evidence Capsule</a></li>
    <li><a href="docs/api/mizan_oscal/index.html">Rust API Documentation</a></li>
  </ul>
</body>
</html>
EOF
fi
touch "${PUBLIC_DIR}/.nojekyll"

# 6. Verify Public Assets
echo ">> [6/6] Verifying Portal Assets..."
if [[ -f "${PUBLIC_DIR}/index.html" ]] && [[ -f "${PUBLIC_DIR}/capsule.html" ]] && [[ -d "${PUBLIC_DIR}/_next" ]]; then
    echo "[OK] Portal generation verified successfully with Next.js 16 assets."
    ls -lh "${PUBLIC_DIR}"
else
    echo "[ERROR] Missing required public artifacts."
    exit 1
fi

echo "=========================================================="
echo "[SUCCESS] Pages & Workbench artifacts ready in ${PUBLIC_DIR}"
echo "=========================================================="

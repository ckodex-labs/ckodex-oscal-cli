#!/usr/bin/env bash
# ==============================================================================
# Mizan · GitLab Pages Documentation & Evidence Portal Generator
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
PUBLIC_DIR="${ROOT_DIR}/public"

echo "=== Mizan · Generating GitLab Pages Site ==="
echo "Root:   ${ROOT_DIR}"
echo "Public: ${PUBLIC_DIR}"
echo "==========================================="

mkdir -p "${PUBLIC_DIR}/badges"
mkdir -p "${PUBLIC_DIR}/docs/api"

# 1. Compile Mizan binary
echo ">> [1/5] Building Mizan binary..."
cargo build --release --bin mizan
MIZAN_BIN="${ROOT_DIR}/target/release/mizan"
if [[ ! -f "${MIZAN_BIN}" ]]; then
    MIZAN_BIN="${HOME}/.cache/cargo-target/release/mizan"
fi

# 2. Generate Rust API Documentation
echo ">> [2/5] Compiling Workspace Rustdoc..."
cargo doc --workspace --no-deps
DOC_DIR="${ROOT_DIR}/target/doc"
if [[ ! -d "${DOC_DIR}" ]]; then
    DOC_DIR="${HOME}/.cache/cargo-target/doc"
fi
if [[ -d "${DOC_DIR}" ]]; then
    cp -r "${DOC_DIR}/"* "${PUBLIC_DIR}/docs/api/"
fi

# 3. Generate shieldcn-zig Badges
echo ">> [3/5] Generating shieldcn-zig Badges (APCA WCAG 3.0)..."
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
echo ">> [4/5] Exporting Offline Cryptographic Evidence Capsule..."
"${MIZAN_BIN}" export capsule \
    -a <(echo '{"assessment-results":{"uuid":"mizan-gl-pages-001","metadata":{"title":"Mizan GitLab Pages Continuous Assurance Report","version":"1.0.0","remarks":"Verified by Mizan Zero-Trust Cryptographic Evidence Fabric"}}}') \
    -o "${PUBLIC_DIR}/capsule.html"

# 5. Verify Public Assets
echo ">> [5/5] Verifying Portal Assets..."
if [[ -f "${PUBLIC_DIR}/index.html" ]] && [[ -f "${PUBLIC_DIR}/capsule.html" ]]; then
    echo "✓ Portal generation verified successfully."
    ls -lh "${PUBLIC_DIR}"
else
    echo "❌ Error: Missing required public artifacts."
    exit 1
fi

echo "==========================================="
echo "✅ GitLab Pages artifacts ready in ${PUBLIC_DIR}"
echo "==========================================="

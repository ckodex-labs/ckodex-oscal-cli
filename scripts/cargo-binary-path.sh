#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
profile="${1:-debug}"

if [[ "$profile" != "debug" && "$profile" != "release" ]]; then
    echo "usage: cargo-binary-path.sh [debug|release]" >&2
    exit 2
fi

target_root="$(
    cd "$client_root"
    cargo metadata --format-version 1 --no-deps |
        python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])'
)"
host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"

for candidate in \
    "$target_root/$host_triple/$profile/oscal-cli" \
    "$target_root/$profile/oscal-cli" \
    "$client_root/target/$host_triple/$profile/oscal-cli" \
    "$client_root/target/$profile/oscal-cli"; do
    if [[ -x "$candidate" ]]; then
        printf '%s\n' "$candidate"
        exit 0
    fi
done

echo "oscal-cli $profile binary was not found under Cargo's target directory" >&2
exit 1

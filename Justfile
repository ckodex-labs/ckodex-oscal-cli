# Mizan · High-Assurance OSCAL Compliance Kernel & Workbench
# Task Automation Recipes

default:
    @just --list

# Run all unit and integration tests
test:
    cargo test --all-targets

# Run formatters and linter checks with zero warnings tolerance
lint:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cd apps/workbench && npm run lint

# Format all code across Rust and TypeScript
fmt:
    cargo fmt
    cd apps/workbench && npm run format

# Build all release binaries and artifacts
build:
    cargo build --release --bins
    cd apps/workbench && npm run build

# Start the Mizan Web Workbench in development mode
workbench:
    cd apps/workbench && npm run dev

# Start the native Mizan Desktop App via Tauri
tauri:
    cd apps/workbench && npm run tauri dev

# Run the native AI Model Context Protocol (MCP) server
mcp:
    cargo run --bin mizan -- mcp

# Run the continuous compliance background daemon
daemon dir="./workspace":
    cargo run --bin mizan -- daemon --dir {{dir}}

# Watch workspace directory for real-time compliance changes
watch dir="./workspace":
    cargo run --bin mizan -- watch {{dir}}

# Execute the full end-to-end zero-trust compliance pipeline
pipeline jurisdiction="us" out="mizan-pipeline-output":
    cargo run --bin mizan -- pipeline run -j {{jurisdiction}} -o {{out}}

# Build the minimal hardened multi-stage Docker container
docker-build:
    docker build -t mizan:latest .

# Audit protocol snapshots against upstream Squarify
audit:
    ./scripts/protocol-audit.sh --output target/protocol-audit.json

# Changelog

All notable changes to Mizan (`ckodex-oscal-cli`) are documented in this file.

## [1.0.0] - 2026-11-10

### Added
- **NIST OSCAL v1.2.3 Kernel Parity**: Full validation, profile resolution, semantic diffing, format conversion (`json`, `yaml`, `csv`, `proto`), linting, deduplication, and Markdown workspace `split` / `assemble` across all 8 OSCAL models (`catalog`, `profile`, `ssp`, `component-definition`, `assessment-plan`, `assessment-results`, `poam`, `mapping-collection`).
- **Universal Tool Calling Protocol (`UTCP v1.0.1`)**: Native `mizan utcp` CLI (`manual`, `config`, `validate`, `call`, `sdk`, `search`), SHA-256 execution receipts, Workbench TypeScript UTCP client (`utcp-client.ts`), `/api/utcp` HTTP route, and Tauri IPC commands compatible with `rs-utcp`, `typescript-utcp`, `go-utcp`, `python-utcp`, and `utcp-mcp`.
- **Model Context Protocol (`MCP`) Server**: Bounded `16 MiB` JSON-RPC 2.0 `stdio` server (`mizan mcp`) exposing 24 compliance authoring, policy evaluation, SBOM, POA&M, Root Fabric, and UTCP tools.
- **Multi-Tenant Root Fabric & gRPC Server**: Partitioned `(tenant_id, resource_id)` isolation across all 8 OSCAL families backed by `FabricDataStore` persistence, startup warm-loading, `/grpc.health.v1.Health`, SPIFFE/SPIRE SVID validation, OIDC verification, and RBAC.
- **Zero-Trust Supply Chain & Continuous Assurance**: CycloneDX/SPDX SBOM ingestion, in-process Regorus policy evaluation (`cis-k8s`, `fedramp`, `itsg33`), Kyverno/OPA policy compilation, time-bounded derogation waivers, SLSA v1.2 & v1.0 in-toto provenance, OASIS SARIF v2.1.0 & GitLab Security reports, and standalone WebCrypto Merkle HTML capsules (`mizan export capsule`).
- **Interactive Surfaces**: Terminal UI (`mizan tui`), Next.js 16 Turbopack Workbench (`apps/workbench`) with static GitHub Pages export (`build:export`), and Tauri 2.0 Desktop application (`build:tauri`).
- **Multi-Platform & Container Distribution**: Pre-built release archives for Linux (`gnu`, `musl`, `aarch64`), macOS (`x86_64`, `aarch64`), and Windows (`x86_64`), SLSA build provenance attestations, composite GitHub Action (`action.yml`), and multi-arch (`linux/amd64`, `linux/arm64`) distroless container images on GHCR.

### Security & Resilience
- Enforced crash-safe `atomic_write_bytes`, `64 MiB` regular-file bounds (`io_safe.rs`), ACID WAL rollback (`tx/`), self-healing CAS blobs (`cas/`), corruption quarantine (`.corrupt.<ts>.bak`), canonicalized workspace path confinement, and zero production `.unwrap()` calls across `src/`.
- Resolved all Rust (`cargo deny check`) and Workbench (`npm audit`) dependency advisories (`0` vulnerabilities).

## [0.1.0] - 2026-09-23

### Added
- Initial public preview of the Mizan OSCAL CLI, TUI observer, protobuf bindings, and Workbench prototype.

# Contributing to Mizan

Thank you for contributing to Mizan (`ckodex-oscal-cli`).

## Prerequisites

- **Rust**: `1.88+` (`rustup toolchain install stable`)
- **Protocol Buffers**: `protoc` (`brew install protobuf` or `apt-get install protobuf-compiler`)
- **Node.js**: `22+` (for `apps/workbench`)
- **Optional**: [`just`](https://github.com/casey/just) and [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny)

## Local Verification Workflow

Before opening a pull request, run the full verification suite:

```bash
# 1. Format check and strict Clippy (-D warnings)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# 2. Rust unit & integration test suite
cargo test --workspace --locked

# 3. Supply-chain and license audit
cargo deny check

# 4. Next.js 16 Workbench type-check and production build
cd apps/workbench
npm ci
npm run lint
npm run build
npm run build:export
```

If your change alters CLI JSON output consumed by the static Workbench snapshot, regenerate the snapshot on a clean working tree after building the release binary:

```bash
cargo build --release --bin mizan
MIZAN_BIN=target/release/mizan node apps/workbench/scripts/gen-snapshot.mjs
```

## Engineering Guidelines

1. **Zero Unsafe & Zero Production Panics**: `#![deny(unsafe_code)]` is enforced. Do not introduce `.unwrap()` or `.expect()` in production paths under `src/`.
2. **Bounded & Crash-Safe File I/O**: Route file reads and writes through `src/document/io_safe.rs` (`read_bounded_regular_file`, `read_bounded_utf8_file`, `atomic_write_bytes`).
3. **Protocol Integrity**: `proto/oscal` files are verified at compile time against `proto.lock` in `build.rs`. Keep LF line endings (`.gitattributes`).
4. **Commit Messages**: Use concise, imperative commit subjects (<= 72 characters) explaining what changed and why.

## License

By contributing to this repository, you agree that your contributions will be licensed under the [Apache License, Version 2.0](LICENSE).

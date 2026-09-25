# syntax=docker/dockerfile:1.4
# ------------------------------------------------------------------------------
# Mizan · Multi-Stage Zero-Trust Hardened Container Build
# ------------------------------------------------------------------------------

# Stage 1: Build & Static Linkage
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev protobuf-dev protoc git

WORKDIR /usr/src/mizan

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY ci ./ci
COPY proto ./proto
COPY schemas ./schemas
COPY build.rs ./
COPY src ./src

# Build statically linked release binaries
RUN cargo build --release --bin mizan --bin oscal-cli

# Stage 2: Minimal Distroless Runtime Surface
FROM gcr.io/distroless/static-debian12:nonroot

LABEL org.opencontainers.image.title="Mizan Compliance Kernel" \
      org.opencontainers.image.description="High-Assurance OSCAL Compliance Engine, SLSA v1.2 Attestation & Policy Kernel" \
      org.opencontainers.image.vendor="Runbase / CKODEX" \
      org.opencontainers.image.licenses="Apache-2.0"

WORKDIR /workspace

# Copy compiled binaries from builder
COPY --from=builder /usr/src/mizan/target/release/mizan /usr/local/bin/mizan
COPY --from=builder /usr/src/mizan/target/release/oscal-cli /usr/local/bin/oscal-cli

USER nonroot:nonroot

ENTRYPOINT ["/usr/local/bin/mizan"]
CMD ["--help"]

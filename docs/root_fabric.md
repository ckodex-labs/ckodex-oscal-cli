# Mizan Root Fabric · Architecture & Security Guide

The **Mizan Root Fabric** provides zero-trust identity, multi-tenant isolation, role-based access governance, and a cryptographically partitioned DataStore for high-assurance compliance systems.

---

## Architecture Overview

```
                      ┌──────────────────────────────────────────────────────────┐
                      │                 Mizan Root Fabric Engine                 │
                      └────────────────────────────┬─────────────────────────────┘
                                                   │
              ┌──────────────────────────┬─────────┴─────────┬──────────────────────────┐
              │                          │                   │                          │
              ▼                          ▼                   ▼                          ▼
    ┌───────────────────┐      ┌───────────────────┐ ┌───────────────┐        ┌───────────────────┐
    │   SPIFFE/SPIRE    │      │   OIDC Provider   │ │ TenantManager │        │    RBAC / ABAC    │
    │  Workload SVIDs   │      │  JWT Claim Engine │ │ Quota Guards  │        │ Permission Matrix │
    └─────────┬─────────┘      └─────────┬─────────┘ └───────┬───────┘        └─────────┬─────────┘
              │                          │                   │                          │
              └──────────────────────────┼───────────────────┴──────────────────────────┘
                                         ▼
                      ┌───────────────────────────────────────┐
                      │      Isolated FabricDataStore         │
                      │  - Key: (TenantId, Namespace, DocId)  │
                      │  - Append-Only Cryptographic Receipts │
                      │  - SHA-256 Digest Verification        │
                      └───────────────────────────────────────┘
```

---

## Core Capabilities

### 1. SPIFFE / SPIRE Workload Identity
Workloads in Kubernetes clusters, pipelines, or cloud hosts attest their identities via SPIFFE IDs conforming to RFC standard URIs:
```
spiffe://meridian.runbase.io/ns/<namespace>/sa/<service-account>
```
- **Trust Domain Verification**: Workloads outside the trusted domain (e.g. `meridian.runbase.io`) fail closed.
- **SVID Types**: Full support for X.509 SVIDs and JWT SVIDs with cryptographic expiration enforcement.

### 2. OpenID Connect (OIDC) User Federation
Federated tokens carrying claims are validated against configured identity providers:
- Validated Claims: `iss` (Issuer), `sub` (Subject), `aud` (Audience), `exp` (Expiry), `tenant_id`, `roles`, `groups`.
- Strict matching prevents cross-issuer token reuse or replay.

### 3. Multi-Tenant Isolation
- **Tenant Context (`TenantContext`)**: Binds every incoming operation to an authenticated `TenantId` and `UserId`.
- **Boundary Assertion**: Calls to `assert_same_tenant` guarantee that no cross-tenant read, write, or query can bleed across partitions.
- **Quotas & Baselines**: Tenant quotas and default compliance jurisdictions (US NIST 800-53, Canadian CCCS ITSG-33, European EUCS / ISO 27001) are strictly governed per tenant.

### 4. Role-Based & Attribute-Based Access Control (RBAC / ABAC)
Roles and permissions are governed across key compliance actions:
- **`TenantAdmin`**: Unrestricted operations within the tenant boundary.
- **`ComplianceArchitect`**: Author catalogs, tailor profiles, manage SSPs, publish baselines, and promote environments.
- **`Auditor`**: Inspect documents, sign attestations, and emit POA&M and Assessment Results (mutation of catalogs/profiles blocked).
- **`SecOpsOperator`**: Execute automated policy scans and record assessment outcomes.
- **`Viewer`**: Read-only access to governed baselines.

### 5. Partitioned Cryptographic DataStore
- Documents are indexed by `(TenantId, Namespace, DocumentId)`.
- Monotonic versioning on all updates.
- Append-only audit log with SHA-256 cryptographic digests, actor IDs, and RFC 3339 timestamps.

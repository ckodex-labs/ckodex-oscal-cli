# Mizan CLI Reference Manual

The `mizan` (and `oscal-cli`) binary is a high-assurance, multi-jurisdiction OSCAL compliance and zero-trust orchestration engine.

---

## Commands Summary

### 1. Root Fabric & Multi-Tenancy (`mizan fabric`)
```bash
# Check status of Root Fabric, active tenants, trust domain, and DataStore
mizan fabric status

# List all registered tenant partitions
mizan fabric tenant list

# Create a new tenant partition with dedicated storage quota
mizan fabric tenant create corp-fintech --name "FinTech Global" --tier Enterprise --quota-gb 250 --jurisdiction us

# Inspect specific tenant metadata
mizan fabric tenant inspect corp-fintech

# Validate a SPIFFE workload identity URI
mizan fabric spiffe spiffe://meridian.runbase.io/ns/prod/sa/auditor

# Decode and validate an OIDC JWT token
mizan fabric oidc <jwt-token> --issuer https://auth.runbase.io --audience mizan-workbench

# Query DataStore partition stats
mizan fabric datastore stats
```

---

### 2. End-to-End Compliance Pipeline (`mizan pipeline`)
```bash
# Execute single-step automated zero-trust pipeline:
# SBOM Ingest -> Rego Policy Eval -> OSCAL Assessment -> SLSA v1.2 Provenance -> SARIF/GitLab Export
mizan pipeline run \
  --jurisdiction us \
  --sbom bom.json \
  --workload deployment.yaml \
  --output-dir pipeline-output \
  --subject production-service:v1.2.0
```

---

### 3. Supply Chain & CI/CD (`mizan sbom`, `mizan export`, `mizan attest`)
```bash
# Convert CycloneDX or SPDX SBOM into OSCAL Component Definition
mizan sbom import -i bom.json -o oscal-components.json

# Export OSCAL Assessment or Catalog to OASIS SARIF v2.1.0 (GitHub Code Scanning)
mizan export sarif -i assessment.json -o code-scanning.sarif

# Export OSCAL Assessment or Catalog to GitLab Security Scanner Report format
mizan export gitlab -i assessment.json -o gl-security.json

# Generate SLSA v1.2 / in-toto supply-chain attestation
mizan attest slsa --subject my-app:1.0 --digest <sha256> --version 1.2 -o provenance.json
```

---

### 4. Policy Evaluation & In-Process Rego (`mizan policy`, `mizan audit`)
```bash
# List built-in Rego compliance rulepacks (CIS Kubernetes, FedRAMP, ITSG-33)
mizan policy rulepack list

# Evaluate built-in Rego compliance rulepack against workload manifest
mizan policy rulepack eval -r cis-k8s-5.2.1 -i pod.yaml

# Audit live Kubernetes cluster or YAML manifests
mizan audit -n default -o assessment-results.json
```

---

### 5. Multi-Jurisdiction Catalogs & Profiles (`mizan catalog`, `mizan profile`, `mizan ssp`)
```bash
# List built-in jurisdictions (US NIST 800-53, CA CCCS ITSG-33, EU EUCS / ISO 27001)
mizan catalog list

# Export built-in jurisdiction catalog to JSON or YAML
mizan catalog export -j us -o nist-800-53-r5.json

# Extend catalog with custom enterprise controls
mizan catalog extend --base us --title "Fintech Controls" -o extended.json

# Authoring template scaffolding
mizan template ssp --standard fedramp-moderate -o scaffolded-ssp.json
```

---

### 6. Agile GitOps Markdown Split & Sync (`mizan split`, `mizan assemble`, `mizan sync`)
```bash
# Split monolithic OSCAL JSON into decomposed markdown workspace
mizan split ssp.json -o workspace/

# Reassemble markdown directory back into valid OSCAL JSON
mizan assemble workspace/ -o ssp-assembled.json

# Perform 3-way AST merge across Git branches
mizan sync --base base.json --upstream upstream.json --local local.json -o merged.json
```

---

### 7. Strict NIST OSCAL Validation & Linting (`mizan validate`, `oscal-cli validate`, `mizan lint`)
```bash
# Validate against official NIST OSCAL v1.2.3 schemas with strict constraints
mizan validate path/to/doc.json --strict
oscal-cli validate --strict path/to/doc.json

# Model-specific validation commands
mizan catalog validate path/to/catalog.json
mizan profile validate path/to/profile.json
mizan ssp validate path/to/ssp.json

# Lint OSCAL document for UUID hygiene, timestamps, and schema deprecations
mizan lint path/to/doc.json
mizan lint path/to/doc.json --fix
oscal-cli lint path/to/doc.json
```

---

### 8. Blast Radius Impact Analysis, Quick-Fix & Waivers (`mizan blast-radius`, `mizan fix`, `mizan waive`)
```bash
# Calculate downstream compliance blast radius of modifying a control or component
mizan blast-radius examples/sample-ssp.json --target ac-1

# Automated quick-fix engine for standard compliance findings
mizan fix path/to/doc.json --rule require-remarks --apply

# Create time-bounded cryptographic pipeline derogation waiver
mizan waive create --rule cis-k8s-5.2.1 --resource deployment.yaml --duration-days 30 --justification "Planned migration"
mizan waive list
mizan waive revoke <lease-id>
```

---

### 9. Cryptographic Evidence Capsules & Contrast Badges (`mizan export capsule`, `mizan export badge`)
```bash
# Export standalone offline cryptographic evidence capsule (WebCrypto Merkle proof)
mizan export capsule -a assessment-results.json -o capsule.html

# Generate APCA WCAG 3.0 contrast-compliant shieldcn-zig badge (?wcag=3, |Lc| >= 60)
mizan export badge --label "FedRAMP" --message "Moderate COMPLIANT" --color green --variant secondary --wcag 3
mizan export badge --from-document examples/sample-catalog.json --variant secondary --wcag 3
```


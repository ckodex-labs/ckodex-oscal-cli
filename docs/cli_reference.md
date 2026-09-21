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
# List built-in jurisdictions (US NIST 800-53, CA CCCS ITSG-33, EU BSI C5)
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

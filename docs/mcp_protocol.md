# Mizan Model Context Protocol (MCP) Server Reference

Mizan includes a native **Model Context Protocol (MCP)** server operating over `stdio` conforming to JSON-RPC 2.0 (protocol version `2024-11-05`). It equips autonomous AI coding agents with direct tools for compliance governance.

---

## Starting the MCP Server

```bash
mizan mcp
```

Or configure in your agent IDE's MCP config:
```json
{
  "mcpServers": {
    "mizan": {
      "command": "mizan",
      "args": ["mcp"]
    }
  }
}
```

---

## Tool Catalog

| Tool Name | Purpose | Key Arguments |
|:---|:---|:---|
| `query_control` | Query an OSCAL catalog for control text, parameters, and guidance | `catalog_file`, `control_id` |
| `inspect_document` | Inspect an OSCAL document summary, kinds, control counts, and metadata | `file` (path) |
| `compute_blast_radius` | Calculate downstream compliance blast radius for a control or component | `file` (path), `target` (string) |
| `validate_fedramp` | Run high-assurance FedRAMP PMO baseline validation on an SSP | `ssp_file` (path), `baseline` (string) |
| `audit_kubernetes_cluster` | Audit a live Kubernetes cluster or workload against NIST 800-53 controls | `namespace` (string), `output_file` (path) |
| `evaluate_rego_policy` | Evaluate a Rego policy file against input JSON with a query package | `policy_file`, `input_data`, `query_package` |
| `sync_3way_merge` | 3-way GitOps AST merge across base, upstream, and local documents | `base_file`, `upstream_file`, `local_file`, `strategy` |
| `split_oscal_markdown` | Split a monolithic OSCAL document into a Markdown authoring workspace | `file` (path), `output_dir` (path) |
| `assemble_oscal_markdown` | Assemble a Markdown workspace back into a validated OSCAL document | `input_dir` (path), `output_file` (path) |
| `export_baseline_catalog` | Retrieve an embedded tri-jurisdictional compliance catalog | `jurisdiction` (string) |
| `extend_enterprise_catalog` | Build an enterprise custom catalog inheriting base standards | `base_jurisdiction`, `title`, `custom_control_id`, `custom_control_title`, `custom_control_description` |
| `generate_slsa_provenance` | Generate a SLSA v1.2 / v1.0 supply-chain attestation statement | `subject_name`, `subject_digest`, `slsa_version`, `evidence_file` |
| `verify_slsa_provenance` | Cryptographically verify a SLSA supply-chain provenance statement | `statement_json` (string) |
| `query_cas_store` | Query local Content-Addressable Storage for object contents or stats | `digest` (string) |
| `export_sarif` | Convert OSCAL compliance document into OASIS SARIF v2.1.0 report | `file` (path) |
| `export_gitlab` | Convert OSCAL compliance document into GitLab Security Scanner report | `file` (path) |
| `import_sbom_cyclonedx` | Convert CycloneDX/SPDX SBOM JSON into OSCAL component-definition | `file` (path) |
| `evaluate_policy_rulepack` | Evaluate built-in Rego compliance rulepacks against a workload | `rule_id`, `input_data` |
| `execute_compliance_pipeline` | Run end-to-end zero-trust pipeline (SBOM -> Rego -> OSCAL -> SLSA -> SARIF/GitLab) | `jurisdiction`, `sbom_file`, `workload_file`, `output_dir`, `subject_name` |
| `fabric_status` | Inspect Root Fabric health, active tenants, trust domain, and DataStore partitions | None |
| `fabric_validate_spiffe` | Validate a SPIFFE Workload ID URI | `spiffe_id` (string) |

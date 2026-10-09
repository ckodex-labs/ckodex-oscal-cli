/**
 * Shapes of the snapshot files consumed by the Pipeline, Jurisdictions & SLSA,
 * and Policy Gates & SBOM surfaces. These mirror the JSON that `mizan
 * --format json <cmd>` emitted when the snapshot was generated
 * (public/snapshot/manifest.json lists the exact argv for each file).
 *
 * Fields are typed as optional where the engine may omit them; consumers must
 * treat a missing field as UNKNOWN, never as zero or false.
 */

export interface PipelineWaiver {
  id: string;
  rule_id: string;
  reason?: string;
  scope?: string;
  author?: string;
  created_at?: string;
  expires_at?: string;
  status?: string;
  fingerprint?: string;
}

/** `mizan pipeline run` */
export interface PipelineRunOutput {
  timestamp?: string;
  jurisdiction?: string;
  oscal_catalog_uuid?: string;
  sbom_components_count?: number;
  evaluated_rules_count?: number;
  passed_rules_count?: number;
  waived_rules_count?: number;
  violations_count?: number;
  active_waivers?: PipelineWaiver[];
  violation_details?: unknown[];
  merkle_root?: string;
  cas_objects_written?: number;
  slsa_provenance_path?: string;
  sarif_report_path?: string;
  gitlab_report_path?: string;
  oscal_assessment_path?: string;
  all_passed?: boolean;
}

/** `mizan export sarif` */
export interface SarifExportOutput {
  format?: string;
  output_file?: string;
  results_count?: number;
  rules_count?: number;
}

/** `mizan sbom import` */
export interface SbomImportOutput {
  format?: string;
  spec_version?: string;
  component_count?: number;
  direct_dependencies?: number;
  oscal_component_uuid?: string;
}

/** `mizan policy rulepack list` (array of these) */
export interface Rulepack {
  id: string;
  name?: string;
  benchmark?: string;
  target_controls?: string[];
  severity?: string;
  description?: string;
  rego_source?: string;
}

/** `mizan catalog list` (array of these) */
export interface BuiltinCatalog {
  code: string;
  controls_count?: number;
  jurisdiction?: string;
  standard?: string;
}

export interface FedrampFinding {
  rule_id: string;
  title?: string;
  severity?: string;
  detail?: string;
}

/** `mizan fedramp validate` */
export interface FedrampValidateOutput {
  file?: string;
  kind?: string;
  baseline?: string;
  is_compliant?: boolean;
  total_rules_checked?: number;
  passed_rules?: number;
  failed_rules?: number;
  /** Number of findings (several findings may share one rule). */
  findings_count?: number;
  findings?: FedrampFinding[];
}

export interface ValidateDiagnostic {
  level?: string;
  code?: string;
  path?: string;
  message?: string;
}

/** `mizan validate` */
export interface ValidateOutput {
  file?: string;
  kind?: string;
  is_valid?: boolean;
  schema_valid?: boolean;
  constraints_valid?: boolean;
  diagnostics?: ValidateDiagnostic[];
}

/** `mizan inspect` (catalog subset used here) */
export interface CatalogInspectOutput {
  file?: string;
  kind?: string;
  title?: string;
  version?: string;
  oscal_version?: string;
  uuid?: string;
  stats?: {
    total_controls?: number;
    total_groups?: number;
    total_params?: number;
    controls_by_family?: Record<string, number>;
  };
}

/** `mizan policy rulepack eval` (via /api/eval) */
export interface RulepackEvalOutput {
  policy_file?: string;
  query?: string;
  passed?: boolean;
  findings?: string[];
  raw_output?: unknown;
}

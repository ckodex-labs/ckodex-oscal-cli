export interface BlastRadiusReport {
  target_id: string;
  target_kind: string;
  risk_exposure_score: number;
  is_critical_path: boolean;
  documents_analyzed: string[];
  direct_dependents: string[];
  transitive_dependents: string[];
  downstream_impact_paths: string[][];
}

export interface FedrampFinding {
  rule_id: string;
  severity: "high" | "medium" | "low";
  title: string;
  detail: string;
  target: string;
}

export interface FedrampReport {
  document_kind: string;
  baseline: string;
  passed: boolean;
  rule_count_evaluated: number;
  violation_count: number;
  findings: FedrampFinding[];
}

export interface MergeConflict {
  control_id: string;
  field: string;
  local_summary: string;
  upstream_summary: string;
  resolution: string;
}

export interface MergeReport {
  strategy: string;
  controls_merged: number;
  added_from_upstream: string[];
  preserved_local_additions: string[];
  updated_from_upstream: string[];
  retained_local_modifications: string[];
  conflicts: MergeConflict[];
  is_clean: boolean;
  output_file?: string;
}

export interface ControlParameter {
  id: string;
  label?: string;
  values?: string[];
}

export interface ControlDetail {
  id: string;
  title: string;
  class?: string;
  statement?: string;
  guidance?: string;
  params?: ControlParameter[];
}

export type LensMode =
  "author" | "architect" | "engineer" | "assessor" | "risk-owner" | "ciso";
export type ThemeMode = "ledger" | "vault" | "hc";

export interface ImpactedNode {
  id: string;
  kind?: string;
  relation?: string;
  title?: string;
  document?: string;
}

export interface BlastRadiusReport {
  target_id: string;
  target_kind: string;
  risk_exposure_score: number;
  is_critical_path: boolean;
  documents_analyzed: string[];
  direct_dependents: (string | ImpactedNode)[];
  transitive_dependents: (string | ImpactedNode)[];
  affected_components?: string[];
  affected_findings?: string[];
  affected_poam_items?: string[];
  downstream_impact_paths?: string[][];
}

export interface FedrampFinding {
  rule_id: string;
  severity: "high" | "medium" | "low" | string;
  title: string;
  detail: string;
  target?: string;
}

export interface FedrampReport {
  document_kind?: string;
  kind?: string;
  file?: string;
  baseline: string;
  passed?: boolean;
  is_compliant?: boolean;
  rule_count_evaluated?: number;
  total_rules_checked?: number;
  violation_count?: number;
  failed_rules?: number;
  passed_rules?: number;
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

export type Presence = "empty" | "present" | "unknown" | "redacted";
export type DirectionalValence = "positive" | "negative" | "neutral" | "mixed" | "unresolved";
export type AntiRelation = "none" | "contradicts" | "attacks" | "invalidates";
export type Coherence = "coherent" | "partially_coherent" | "decoherent" | "reconciling";
export type EvidenceStatus = "claimed" | "inferred" | "observed" | "verified" | "attested" | "quarantined";

export interface VectorState {
  presence: Presence;
  valence: DirectionalValence;
  anti: AntiRelation;
  coherence: Coherence;
  evidence: EvidenceStatus;
  lifecycle: string;
  epoch: number;
}

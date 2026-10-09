/**
 * Shapes of `mizan --format json` outputs consumed by the chat answer cards
 * and the Inspector. These mirror the captured snapshot files under
 * public/snapshot/; fields the engine may omit are optional.
 */

export interface InspectReport {
  file: string;
  kind: string;
  title?: string | null;
  version?: string | null;
  oscal_version?: string | null;
  uuid?: string | null;
  last_modified?: string | null;
  published?: string | null;
  stats?: {
    total_controls?: number;
    controls_by_family?: Record<string, number>;
    total_groups?: number;
    total_params?: number;
    total_roles?: number;
    total_parties?: number;
    total_components?: number;
    total_findings?: number;
    total_observations?: number;
    total_poam_items?: number;
    total_mappings?: number;
  };
}

export interface ValidateDiagnostic {
  level: string;
  code?: string;
  path?: string;
  message: string;
}

export interface ValidateReport {
  file: string;
  kind?: string;
  is_valid: boolean;
  schema_valid?: boolean;
  constraints_valid?: boolean;
  diagnostics?: ValidateDiagnostic[];
}

/** Derived projection written by the snapshot script (atlas.json). */
export interface AtlasControl {
  id: string;
  family: string;
  title: string;
  parent: string | null;
  in_baseline: boolean;
  withdrawn: boolean;
  params?: { id: string; label?: string }[];
  statement?: string;
}

export interface AtlasFile {
  schema: string;
  statementFormat?: string;
  counts?: Record<string, number>;
  families?: { id: string; title: string; total_in_catalog: number; in_baseline: number }[];
  controls: AtlasControl[];
}

/** `mizan fabric spiffe <id> --format json` (offline parse). */
export interface SpiffeParseReport {
  spiffe_id: string;
  trust_domain: string;
  path: string;
  namespace?: string | null;
  service_account?: string | null;
  valid_for_trust_domain: boolean;
}

export function countByLevel(diags: ValidateDiagnostic[] | undefined) {
  const out = { error: 0, warning: 0, other: 0 };
  for (const d of diags ?? []) {
    const l = d.level.toLowerCase();
    if (l === "error") out.error += 1;
    else if (l === "warning") out.warning += 1;
    else out.other += 1;
  }
  return out;
}

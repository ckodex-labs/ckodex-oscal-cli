/**
 * Types for the build-time snapshot projections consumed by Atlas and
 * Composer. Shapes mirror the actual files under public/snapshot/ as written
 * by scripts/gen-snapshot.mjs and by `mizan --format json`.
 *
 * These are read-only views of engine output. Nothing here is computed in the
 * browser except the derived indexes in components/surfaces/atlas/.
 */

/* ------------------------------------------------------------------ */
/* atlas.json  (schema mizan.workbench.atlas/v1)                       */
/* ------------------------------------------------------------------ */

export interface AtlasParam {
  id: string;
  label: string;
}

/** Fields present on every control in the catalog projection. */
interface AtlasControlBase {
  /** Lowercase OSCAL id, e.g. "ac-2" or "ac-2.1". */
  id: string;
  /** Lowercase family id, e.g. "ac". */
  family: string;
  title: string;
  /** Parent control id for enhancements, null for base controls. */
  parent: string | null;
  in_baseline: boolean;
  withdrawn: boolean;
}

/**
 * Baseline controls additionally carry the statement and params. Catalog
 * controls outside the baseline do not; their statement is not projected
 * (absent, not empty).
 */
export interface AtlasControl extends AtlasControlBase {
  params?: AtlasParam[];
  /**
   * Statement prose, whitespace-collapsed, parameter insertions rendered as
   * `[param-id]`, truncated to 600 characters with a trailing "...".
   */
  statement?: string;
}

export interface AtlasFamily {
  id: string;
  title: string;
  total_in_catalog: number;
  in_baseline: number;
}

export interface AtlasSource {
  path: string;
  sha256: string;
  source?: string;
}

export interface Atlas {
  schema: "mizan.workbench.atlas/v1";
  derivedFrom: {
    catalog: AtlasSource;
    profile: AtlasSource;
    resolvedCatalog: { command: string; sha256: string; note?: string };
  };
  statementFormat: string;
  counts: {
    catalogControls: number;
    baselineControls: number;
    baselineFamilies: number;
    unmatchedBaselineControls: number;
  };
  unmatchedBaselineControls: string[];
  families: AtlasFamily[];
  controls: AtlasControl[];
}

/* ------------------------------------------------------------------ */
/* ssp-status.json  (schema mizan.workbench.ssp-status/v1)             */
/* ------------------------------------------------------------------ */

export type Presence = "EMPTY" | "PRESENT" | "UNKNOWN" | "REDACTED";

export interface SspComponentStatus {
  component_uuid: string;
  implementation_status: string | null;
  status_presence: Presence;
}

export interface SspImplementedRequirement {
  control_id: string;
  uuid: string;
  implementation_status: string | null;
  status_presence: Presence;
  by_components: SspComponentStatus[];
}

export interface SspStatus {
  schema: "mizan.workbench.ssp-status/v1";
  derivedFrom: { path: string; sha256: string };
  systemTitle: string;
  semantics: string;
  implemented_requirements: SspImplementedRequirement[];
}

/* ------------------------------------------------------------------ */
/* mizan --format json validate / inspect                              */
/* ------------------------------------------------------------------ */

export interface ValidateDiagnostic {
  level: string;
  code: string;
  path: string;
  message: string;
}

export interface ValidateResult {
  file: string;
  kind: string;
  is_valid: boolean;
  schema_valid: boolean;
  constraints_valid: boolean;
  diagnostics: ValidateDiagnostic[];
}

export interface InspectResult {
  file: string;
  kind: string;
  title: string;
  version: string;
  oscal_version: string;
  uuid: string;
  last_modified: string | null;
  published: string | null;
  stats: {
    total_controls: number;
    controls_by_family: Record<string, number>;
    total_groups: number;
    total_params: number;
    [k: string]: unknown;
  };
}

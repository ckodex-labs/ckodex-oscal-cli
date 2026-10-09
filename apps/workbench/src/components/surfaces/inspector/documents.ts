/** Documents the Inspector can show from the build-time snapshot. */

export type DocId = "catalog" | "baseline" | "ssp";

export interface InspectorDoc {
  id: DocId;
  label: string;
  inspectId: string;
  validateId: string;
  /** Repo-relative source input recorded in manifest.inputs. */
  sourceInput: string;
  note?: string;
}

export const DOCS: InspectorDoc[] = [
  {
    id: "catalog",
    label: "NIST catalog",
    inspectId: "nist-catalog-inspect",
    validateId: "nist-catalog-validate",
    sourceInput: "examples/nist-800-53-r5/NIST_SP-800-53_rev5_catalog.json",
  },
  {
    id: "baseline",
    label: "Moderate baseline",
    inspectId: "nist-moderate-inspect",
    validateId: "nist-moderate-validate",
    sourceInput: "examples/nist-800-53-r5/NIST_SP-800-53_rev5_MODERATE-baseline_profile.json",
    note: "This is the catalog produced by `mizan resolve` on the Moderate baseline profile at snapshot time, not the profile itself.",
  },
  {
    id: "ssp",
    label: "Sample SSP",
    inspectId: "ssp-inspect",
    validateId: "ssp-validate",
    sourceInput: "examples/sample-ssp.json",
    note: "Repository example. It is schema-invalid; the validation errors below are genuine engine output.",
  },
];

/** Suggested repo-relative paths for live runs. */
export const LIVE_PATHS = [
  "examples/nist-800-53-r5/NIST_SP-800-53_rev5_catalog.json",
  "examples/nist-800-53-r5/NIST_SP-800-53_rev5_MODERATE-baseline_profile.json",
  "examples/sample-ssp.json",
];

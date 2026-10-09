/**
 * Deterministic command language for the Copilot drawer.
 *
 * There is no language model behind the drawer. Input is matched against a
 * small fixed grammar; anything else is reported as not understood.
 */

import type { Provenance } from "@/lib/provenance";
import type { BlastRadiusReport } from "@/lib/oscal-types";

export type ValidateDoc = "catalog" | "baseline" | "ssp";

export type Answer =
  | { kind: "help" }
  | { kind: "control"; id: string }
  | { kind: "blast-snapshot" }
  | { kind: "blast-live"; report: BlastRadiusReport; provenance: Provenance }
  | { kind: "fedramp" }
  | { kind: "validate"; doc: ValidateDoc }
  | { kind: "pending"; text: string }
  | { kind: "note"; tone: "info" | "warn" | "neg"; text: string };

export interface Entry {
  id: number;
  input: string;
  answer: Answer;
}

/** The snapshot blast-radius capture targets this control only. */
export const SNAPSHOT_BLAST_TARGET = "ac-1";
export const SAMPLE_SSP = "examples/sample-ssp.json";

const CONTROL_ID = /^[a-z]{2}-\d+(\.\d+)?$/;

export type Parsed =
  | { type: "help" }
  | { type: "control"; id: string }
  | { type: "blast"; target: string }
  | { type: "fedramp" }
  | { type: "validate"; doc: ValidateDoc }
  | { type: "unknown" };

export function parseCommand(raw: string): Parsed {
  const s = raw.trim().toLowerCase().replace(/\s+/g, " ");
  if (s === "help" || s === "?" || s === "commands") return { type: "help" };

  const show = s.match(/^(?:show|control|lookup) ([a-z]{2}-\d+(?:\.\d+)?)$/);
  if (show) return { type: "control", id: show[1] };
  if (CONTROL_ID.test(s)) return { type: "control", id: s };

  const blast = s.match(/^blast[ -]?radius(?: (?:of |for )?([a-z]{2}-\d+(?:\.\d+)?))?$/);
  if (blast) return { type: "blast", target: blast[1] ?? SNAPSHOT_BLAST_TARGET };

  if (/^fedramp( validate| check)?( ssp)?$/.test(s)) return { type: "fedramp" };

  const val = s.match(/^validate (catalog|nist|baseline|moderate|ssp)$/);
  if (val) {
    const v = val[1];
    const doc: ValidateDoc = v === "ssp" ? "ssp" : v === "baseline" || v === "moderate" ? "baseline" : "catalog";
    return { type: "validate", doc };
  }
  return { type: "unknown" };
}

export const COMMAND_HELP: { cmd: string; what: string }[] = [
  { cmd: "show ac-2", what: "Control text and baseline membership from the NIST SP 800-53 Rev 5 catalog projection." },
  { cmd: "blast radius", what: "Blast radius of ac-1 in the sample SSP (other targets need the local engine)." },
  { cmd: "fedramp", what: "FedRAMP Moderate check of the sample SSP." },
  { cmd: "validate ssp | catalog | baseline", what: "Schema and constraint validation result." },
];

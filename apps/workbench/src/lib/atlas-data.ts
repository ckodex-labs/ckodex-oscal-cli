/**
 * FIXTURE data for the Bridge, Ledger and Docket surfaces.
 *
 * Nothing in this file was produced by the mizan engine and none of it
 * describes a real system. It exists so those surfaces can be designed and
 * reviewed before the engine inputs exist:
 *
 *   Bridge  -> an OSCAL mapping-collection processed by `mizan mapping`
 *   Ledger  -> content-addressed evidence via `mizan cas` / `mizan evidence`
 *   Docket  -> an OSCAL POA&M processed by `mizan poam`
 *
 * Rules for this file:
 * - No signatures, digests, or "sealed"/"verified" flags. Fixture evidence has
 *   no stored object, so its integrity is UNKNOWN, not positive.
 * - Actors are role names, never people or organizations.
 * - Control ids are real NIST SP 800-53 rev5 ids present in the Moderate
 *   baseline, so Docket can navigate to them in the Atlas.
 */

/* ------------------------------------------------------------------ */
/* Types still consumed by the Atlas surface                           */
/* ------------------------------------------------------------------ */

export interface RegionFamily {
  id: string;
  short: string;
  n: number;
  x: number;
  y: number;
  cols: number;
  w?: number;
  h?: number;
  cx?: number;
  cy?: number;
  cov?: number;
}

export interface AtlasControl {
  id: string;
  fam: string;
  title: string;
  st: "implemented" | "partial" | "planned" | "unassessed" | "na";
  days: number;
  deps: number;
  x: number;
  y: number;
  poam: boolean;
  drift: boolean;
  att: boolean;
}

/* ------------------------------------------------------------------ */
/* Bridge                                                              */
/* ------------------------------------------------------------------ */

export interface MappingRow {
  id: string;
  t: string;
}

export type MappingRelationship =
  | "equal-to"
  | "equivalent-to"
  | "subset-of"
  | "superset-of"
  | "intersects-with"
  | "no-relationship";

export interface BridgeEdge {
  id: string;
  /** Index into the source rows. */
  l: number;
  /** Index into the target rows. */
  r: number;
  rel: MappingRelationship;
  rat: "functional" | "semantic" | "syntactic";
  /** Confidence 0..1 assigned by whoever proposed the edge. */
  conf: number;
  /** Fraction of the target requirement covered, 0..1. */
  cov: number;
  st: "complete" | "draft";
  m: "human" | "automation" | "hybrid";
  /** Role or tool that proposed the edge. */
  by: string;
  /** Optional qualifier carried from the mapping, e.g. "addressable". */
  qual?: string;
  /** Edge was inferred transitively through another framework. */
  der?: boolean;
}

/* ------------------------------------------------------------------ */
/* Ledger                                                              */
/* ------------------------------------------------------------------ */

export type EvidenceKind =
  | "scan"
  | "query"
  | "test"
  | "export"
  | "report"
  | "minutes"
  | "screenshot";

export interface EvidenceItem {
  id: string;
  t: string;
  kind: EvidenceKind;
  /** Role or pipeline job that collected it. */
  by: string;
  m: "automation" | "human" | "hybrid";
  /** Age relative to the fixture reference date, as display text. */
  age: string;
  /** Age in days, for filtering and sorting. */
  days: number;
  /** NIST SP 800-53 control ids (uppercase display form). */
  ids: string[];
}

/* ------------------------------------------------------------------ */
/* Docket                                                              */
/* ------------------------------------------------------------------ */

export type PoamStatus = "open" | "in-progress" | "pending-review";

export interface PoamItem {
  id: string;
  t: string;
  /** Owner role. */
  own: string;
  /** Due date relative to the fixture reference date, in days (negative = overdue). */
  dueN: number;
  /** Days the item has been open. */
  age: number;
  /** NIST SP 800-53 control ids (uppercase display form). */
  ids: string[];
  status: PoamStatus;
  milestones: { done: number; total: number };
  risk: "low" | "moderate" | "high";
}

/** All fixture dates are relative to this date. It is not "today". */
export const FIXTURE_REFERENCE_DATE = "2026-07-01";

export function fixtureDateFromOffset(days: number): string {
  const d = new Date(`${FIXTURE_REFERENCE_DATE}T00:00:00Z`);
  d.setUTCDate(d.getUTCDate() + days);
  return d.toISOString().slice(0, 10);
}

/* ------------------------------------------------------------------ */
/* Fixture factory                                                     */
/* ------------------------------------------------------------------ */

export function createAtlasInitialData() {
  const mer: MappingRow[] = [
    { id: "PRAC-AC-01", t: "Access lifecycle (joiner, mover, leaver)" },
    { id: "PRAC-AC-02", t: "Least-privilege administrative access" },
    { id: "PRAC-LOG-01", t: "Central security logging" },
    { id: "PRAC-LOG-02", t: "Log review and alerting" },
    { id: "PRAC-CRY-01", t: "Encryption in transit" },
    { id: "PRAC-VUL-01", t: "Vulnerability remediation targets" },
    { id: "PRAC-BCP-01", t: "Backup and restore drills" },
    { id: "PRAC-DEV-01", t: "Secure build pipeline" },
    { id: "PRAC-PHY-01", t: "Physical access reviews" },
  ];

  const iso: MappingRow[] = [
    { id: "A.5.15", t: "Access control" },
    { id: "A.5.18", t: "Access rights" },
    { id: "A.8.2", t: "Privileged access rights" },
    { id: "A.8.15", t: "Logging" },
    { id: "A.8.16", t: "Monitoring activities" },
    { id: "A.8.24", t: "Use of cryptography" },
    { id: "A.8.8", t: "Management of technical vulnerabilities" },
    { id: "A.8.13", t: "Information backup" },
    { id: "A.8.25", t: "Secure development life cycle" },
    { id: "A.8.28", t: "Secure coding" },
    { id: "A.5.30", t: "ICT readiness for business continuity" },
    { id: "A.8.12", t: "Data leakage prevention" },
  ];

  const csf: MappingRow[] = [
    { id: "PR.AA-01", t: "Identities and credentials are managed" },
    { id: "PR.AA-05", t: "Access permissions follow least privilege" },
    { id: "DE.CM-01", t: "Networks are monitored" },
    { id: "DE.AE-02", t: "Adverse events are analyzed" },
    { id: "PR.DS-02", t: "Data in transit is protected" },
    { id: "PR.DS-11", t: "Backups are maintained and tested" },
    { id: "ID.RA-01", t: "Vulnerabilities are identified" },
    { id: "PR.PS-06", t: "Secure development practices are used" },
    { id: "GV.OC-03", t: "Legal and regulatory requirements are understood" },
    { id: "DE.CM-06", t: "External provider activity is monitored" },
  ];

  const edgesIso: BridgeEdge[] = [
    { id: "e1", l: 0, r: 0, rel: "intersects-with", rat: "semantic", conf: 0.61, cov: 0.4, st: "complete", m: "human", by: "Compliance Analyst" },
    { id: "e2", l: 0, r: 1, rel: "equivalent-to", rat: "functional", conf: 0.86, cov: 0.9, st: "complete", m: "human", by: "Compliance Analyst" },
    { id: "e3", l: 1, r: 2, rel: "equal-to", rat: "functional", conf: 0.93, cov: 1.0, st: "complete", m: "human", by: "Platform Engineer" },
    { id: "e4", l: 2, r: 3, rel: "superset-of", rat: "functional", conf: 0.8, cov: 0.75, st: "complete", m: "hybrid", by: "Security Engineer" },
    { id: "e5", l: 3, r: 4, rel: "equivalent-to", rat: "functional", conf: 0.7, cov: 0.6, st: "draft", m: "human", by: "Compliance Analyst" },
    { id: "e6", l: 3, r: 4, rel: "intersects-with", rat: "semantic", conf: 0.55, cov: 0.35, st: "draft", m: "human", by: "Security Engineer" },
    { id: "e7", l: 4, r: 5, rel: "subset-of", rat: "syntactic", conf: 0.45, cov: 0.3, st: "draft", m: "automation", by: "Suggestion tool (illustrative)" },
    { id: "e8", l: 5, r: 6, rel: "equivalent-to", rat: "functional", conf: 0.78, cov: 0.8, st: "complete", m: "human", by: "Vulnerability Manager" },
    { id: "e9", l: 6, r: 7, rel: "equal-to", rat: "functional", conf: 0.9, cov: 0.95, st: "complete", m: "human", by: "Platform Engineer" },
    { id: "e10", l: 7, r: 8, rel: "intersects-with", rat: "semantic", conf: 0.58, cov: 0.5, st: "draft", m: "automation", by: "Suggestion tool (illustrative)", qual: "addressable" },
    { id: "e11", l: 7, r: 9, rel: "intersects-with", rat: "functional", conf: 0.8, cov: 0.4, st: "complete", m: "human", by: "Application Security Lead" },
  ];

  const edgesCsf: BridgeEdge[] = [
    { id: "c1", l: 0, r: 0, rel: "intersects-with", rat: "functional", conf: 0.7, cov: 0.6, st: "complete", m: "human", by: "Compliance Analyst" },
    { id: "c2", l: 1, r: 1, rel: "subset-of", rat: "functional", conf: 0.8, cov: 0.7, st: "complete", m: "human", by: "Platform Engineer" },
    { id: "c3", l: 2, r: 2, rel: "superset-of", rat: "functional", conf: 0.75, cov: 0.7, st: "complete", m: "hybrid", by: "Security Engineer" },
    { id: "c4", l: 3, r: 3, rel: "equivalent-to", rat: "semantic", conf: 0.55, cov: 0.4, st: "draft", m: "automation", by: "Inferred through ISO A.8.16", der: true },
    { id: "c5", l: 4, r: 4, rel: "equivalent-to", rat: "functional", conf: 0.8, cov: 0.8, st: "complete", m: "human", by: "Vulnerability Manager" },
    { id: "c6", l: 6, r: 5, rel: "equal-to", rat: "functional", conf: 0.85, cov: 0.9, st: "complete", m: "human", by: "Platform Engineer" },
    { id: "c7", l: 5, r: 6, rel: "intersects-with", rat: "semantic", conf: 0.6, cov: 0.5, st: "draft", m: "human", by: "Vulnerability Manager" },
    { id: "c8", l: 7, r: 7, rel: "intersects-with", rat: "functional", conf: 0.72, cov: 0.6, st: "complete", m: "human", by: "Application Security Lead" },
  ];

  const evidence: EvidenceItem[] = [
    { id: "EV-01", t: "Configuration scan, production landing zone", kind: "scan", by: "Pipeline job: config-scan", m: "automation", age: "6 minutes", days: 0.004, ids: ["SC-7", "CM-6"] },
    { id: "EV-02", t: "MFA enrollment coverage query", kind: "query", by: "Pipeline job: idp-export", m: "automation", age: "55 minutes", days: 0.04, ids: ["IA-2"] },
    { id: "EV-03", t: "Vulnerability scan delta", kind: "scan", by: "Pipeline job: vuln-scan", m: "automation", age: "3 hours", days: 0.12, ids: ["RA-5", "SI-2"] },
    { id: "EV-04", t: "Alert-routing test for security monitoring", kind: "test", by: "Pipeline job: alert-probe", m: "automation", age: "26 hours", days: 1.1, ids: ["AU-6", "IR-4"] },
    { id: "EV-05", t: "Quarterly access-review export", kind: "export", by: "Identity Administrator", m: "hybrid", age: "12 days", days: 12, ids: ["AC-2", "AC-6"] },
    { id: "EV-06", t: "Penetration test report", kind: "report", by: "External Assessor (role)", m: "human", age: "74 days", days: 74, ids: ["SC-7", "SI-4"] },
    { id: "EV-07", t: "Backup restore-drill minutes", kind: "minutes", by: "Continuity Coordinator", m: "human", age: "160 days", days: 160, ids: ["CP-9", "CP-4"] },
    { id: "EV-08", t: "Screenshot of firewall console", kind: "screenshot", by: "Collector not recorded", m: "human", age: "335 days", days: 335, ids: ["SC-7"] },
  ];

  const poams: PoamItem[] = [
    { id: "POAM-01", t: "Storage bucket public-access review incomplete", own: "Cloud Platform Owner", dueN: -6, age: 130, ids: ["AC-3", "SC-7"], status: "in-progress", milestones: { done: 2, total: 4 }, risk: "high" },
    { id: "POAM-02", t: "Legacy remote-access gateway lacks MFA", own: "Network Owner", dueN: 9, age: 84, ids: ["IA-2", "AC-17"], status: "in-progress", milestones: { done: 1, total: 3 }, risk: "high" },
    { id: "POAM-03", t: "Endpoint monitoring gap on build agents", own: "Security Operations Lead", dueN: 14, age: 33, ids: ["SI-4"], status: "open", milestones: { done: 0, total: 2 }, risk: "moderate" },
    { id: "POAM-04", t: "Disaster-recovery runbook untested for payment service", own: "Continuity Coordinator", dueN: 30, age: 200, ids: ["CP-4"], status: "open", milestones: { done: 0, total: 3 }, risk: "moderate" },
    { id: "POAM-05", t: "Audit log retention below policy in one region", own: "Logging Platform Owner", dueN: 21, age: 40, ids: ["AU-11"], status: "pending-review", milestones: { done: 3, total: 3 }, risk: "moderate" },
    { id: "POAM-06", t: "Container base images not pinned to digests", own: "Platform Engineer", dueN: 45, age: 26, ids: ["CM-2", "CM-6"], status: "open", milestones: { done: 0, total: 2 }, risk: "low" },
    { id: "POAM-07", t: "Vendor SSO offboarding is a manual step", own: "Identity Administrator", dueN: 90, age: 12, ids: ["AC-2"], status: "open", milestones: { done: 0, total: 2 }, risk: "low" },
  ];

  return { mer, iso, csf, edgesIso, edgesCsf, evidence, poams };
}

/* ------------------------------------------------------------------ */
/* Relationship glyph geometry (two-circle set diagrams)               */
/* ------------------------------------------------------------------ */

export function getRelationshipGlyph(rel: string) {
  const G: Record<
    string,
    { c1x: number; c1r: number; c1f: string; c2x: number; c2r: number; c2f: string }
  > = {
    "equal-to": { c1x: 0, c1r: 6.5, c1f: "none", c2x: 0, c2r: 4, c2f: "none" },
    "equivalent-to": { c1x: -1.5, c1r: 6, c1f: "none", c2x: 1.5, c2r: 6, c2f: "none" },
    "subset-of": { c1x: 1.5, c1r: 7.5, c1f: "none", c2x: -1.5, c2r: 3.2, c2f: "currentColor" },
    "superset-of": { c1x: -1.5, c1r: 7.5, c1f: "none", c2x: 1.5, c2r: 3.2, c2f: "currentColor" },
    "intersects-with": { c1x: -3.5, c1r: 5.5, c1f: "none", c2x: 3.5, c2r: 5.5, c2f: "none" },
    "no-relationship": { c1x: -7, c1r: 4.5, c1f: "none", c2x: 7, c2r: 4.5, c2f: "none" },
  };
  return G[rel] || G["intersects-with"];
}

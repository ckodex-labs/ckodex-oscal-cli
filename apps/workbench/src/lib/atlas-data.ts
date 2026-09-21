// Mizan / Atlas Domain Data & Geometric State

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

export interface MappingRow {
  id: string;
  t: string;
}

export interface BridgeEdge {
  id: string;
  l: number;
  r: number;
  rel:
    | "equal-to"
    | "equivalent-to"
    | "subset-of"
    | "superset-of"
    | "intersects-with"
    | "no-relationship";
  rat: "functional" | "semantic" | "syntactic";
  conf: number;
  cov: number;
  st: "complete" | "draft";
  m: "human" | "automation" | "hybrid";
  by: string;
  sealed?: boolean;
  hash?: string;
  disp?: number;
  qual?: string;
  der?: boolean;
}

export interface EvidenceItem {
  t: string;
  by: string;
  m: "automation" | "human" | "hybrid";
  age: string;
  days: number;
  ids: string[];
  signed: boolean;
  hash?: string;
}

export interface PoamItem {
  id: string;
  t: string;
  own: string;
  due: string;
  dueN: number;
  age: number;
  ids: string[];
  impl: number;
  blast?: number;
  pressure?: number;
}

export function createAtlasInitialData() {
  // Deterministic PRNG
  let seed = 42;
  const R = () => {
    seed = (seed * 16807) % 2147483647;
    return (seed - 1) / 2147483646;
  };

  const fams: RegionFamily[] = [
    { id: "AC", short: "ACCESS CONTROL", n: 14, x: 42, y: 72, cols: 4 },
    { id: "AU", short: "AUDIT", n: 9, x: 426, y: 72, cols: 3 },
    { id: "IA", short: "IDENTIFICATION", n: 8, x: 72, y: 340, cols: 4 },
    { id: "SC", short: "COMMUNICATIONS", n: 13, x: 428, y: 260, cols: 4 },
    { id: "SI", short: "INTEGRITY", n: 9, x: 762, y: 60, cols: 3 },
    { id: "CM", short: "CONFIGURATION", n: 9, x: 398, y: 462, cols: 3 },
    { id: "CP", short: "CONTINGENCY", n: 6, x: 146, y: 496, cols: 3 },
    { id: "IR", short: "INCIDENT RESPONSE", n: 7, x: 880, y: 494, cols: 4 },
    { id: "RA", short: "RISK", n: 5, x: 1002, y: 96, cols: 3 },
    { id: "CA", short: "ASSESSMENT", n: 6, x: 660, y: 530, cols: 3 },
  ];

  const titles: Record<string, string> = {
    "AC-2": "Account Management",
    "AC-3": "Access Enforcement",
    "AC-6": "Least Privilege",
    "AC-17": "Remote Access",
    "AU-2": "Event Logging",
    "AU-6": "Audit Record Review",
    "AU-11": "Audit Record Retention",
    "IA-2": "Multi-factor Authentication",
    "IA-5": "Authenticator Management",
    "SC-7": "Boundary Protection",
    "SC-8": "Transmission Confidentiality",
    "SC-12": "Key Management",
    "SI-2": "Flaw Remediation",
    "SI-4": "System Monitoring",
    "CM-2": "Baseline Configuration",
    "CM-6": "Configuration Settings",
    "CP-4": "Contingency Plan Testing",
    "CP-9": "System Backup",
    "IR-4": "Incident Handling",
    "RA-5": "Vulnerability Monitoring",
    "CA-7": "Continuous Monitoring",
  };

  const ctrls: AtlasControl[] = [];
  fams.forEach((f) => {
    const rows = Math.ceil(f.n / f.cols);
    f.w = f.cols * 34 + 22;
    f.h = rows * 34 + 38;
    f.cx = f.x + (f.w || 0) / 2;
    f.cy = f.y + (f.h || 0) / 2;

    for (let j = 0; j < f.n; j++) {
      const r1 = R(),
        r2 = R(),
        r3 = R(),
        r4 = R();
      const st =
        r1 < 0.45
          ? "implemented"
          : r1 < 0.7
            ? "partial"
            : r1 < 0.82
              ? "planned"
              : r1 < 0.94
                ? "unassessed"
                : "na";
      const id = `${f.id}-${j + 1}`;
      ctrls.push({
        id,
        fam: f.id,
        title: titles[id] || "",
        st,
        days: Math.floor(r2 * 470) + 6,
        deps:
          st === "na" || st === "unassessed" ? 0 : Math.floor(r3 * r3 * 13) + 1,
        x: f.x + 11 + (j % f.cols) * 34 + 16,
        y: f.y + 28 + Math.floor(j / f.cols) * 34 + 16,
        poam: false,
        drift: r4 > 0.82,
        att: st === "implemented" && r4 > 0.55,
      });
    }
  });

  const by: Record<string, AtlasControl> = {};
  ctrls.forEach((c) => (by[c.id] = c));

  const rename = (from: string, to: string) => {
    const c = by[from];
    if (c && !by[to]) {
      delete by[from];
      c.id = to;
      by[to] = c;
    }
  };
  rename("AC-14", "AC-17");
  rename("AU-9", "AU-11");

  const set = (id: string, o: Partial<AtlasControl>) => {
    if (by[id]) Object.assign(by[id], o);
  };
  set("AC-2", {
    st: "implemented",
    days: 9,
    deps: 12,
    att: true,
    drift: false,
  });
  set("AC-3", { st: "implemented", days: 31, deps: 10, poam: true });
  set("AC-6", { st: "partial", days: 88, deps: 8 });
  set("AC-17", { st: "partial", days: 140, deps: 5, poam: true });
  set("IA-2", { st: "implemented", days: 2, deps: 11, att: true, poam: true });
  set("SC-7", { st: "partial", days: 210, deps: 9, poam: true, att: false });
  set("SI-4", { st: "partial", days: 96, deps: 7, poam: true, drift: true });
  set("IR-4", { st: "partial", days: 388, deps: 4 });
  set("AU-11", { st: "partial", days: 64, deps: 3, poam: true });
  set("CM-2", { st: "implemented", days: 44, deps: 6, poam: true });
  set("CP-4", { st: "planned", days: 430, deps: 2, poam: true });

  ctrls.forEach((c) => {
    if (c.days > 180) c.att = false;
  });

  fams.forEach((f) => {
    const fc = ctrls.filter((c) => c.fam === f.id);
    f.cov =
      fc.reduce(
        (a, c) =>
          a + (c.st === "implemented" ? 1 : c.st === "partial" ? 0.5 : 0),
        0,
      ) / f.n;
  });

  const corridors: [string, string][] = [
    ["AC", "AU"],
    ["AC", "IA"],
    ["AC", "SC"],
    ["SC", "SI"],
    ["SC", "IR"],
    ["AC", "CM"],
    ["CM", "CP"],
    ["CM", "CA"],
    ["SI", "RA"],
    ["AU", "SI"],
  ];

  const mer: MappingRow[] = [
    { id: "MER-AC-01", t: "Access lifecycle (joiner–mover–leaver)" },
    { id: "MER-AC-02", t: "Least-privilege administrative access" },
    { id: "MER-LOG-01", t: "Central security logging" },
    { id: "MER-LOG-02", t: "Log review & alerting" },
    { id: "MER-CRY-01", t: "Encryption in transit" },
    { id: "MER-VUL-01", t: "Vulnerability remediation SLAs" },
    { id: "MER-BCP-01", t: "Backup & restore drills" },
    { id: "MER-DEV-01", t: "Secure build pipeline" },
    { id: "MER-PHY-01", t: "Physical access reviews" },
  ];

  const iso: MappingRow[] = [
    { id: "A.5.15", t: "Access control" },
    { id: "A.5.18", t: "Access rights" },
    { id: "A.8.2", t: "Privileged access rights" },
    { id: "A.8.15", t: "Logging" },
    { id: "A.8.16", t: "Monitoring activities" },
    { id: "A.8.24", t: "Use of cryptography" },
    { id: "A.8.8", t: "Technical vulnerabilities" },
    { id: "A.8.13", t: "Information backup" },
    { id: "A.8.25", t: "Secure development" },
    { id: "A.8.28", t: "Secure coding" },
    { id: "A.5.30", t: "ICT readiness for BC" },
    { id: "A.8.12", t: "Data leakage prevention" },
  ];

  const csf: MappingRow[] = [
    { id: "PR.AA-01", t: "Identities & credentials managed" },
    { id: "PR.AA-05", t: "Access permissions & least privilege" },
    { id: "DE.CM-01", t: "Networks monitored" },
    { id: "DE.AE-02", t: "Events analyzed" },
    { id: "PR.DS-02", t: "Data-in-transit protected" },
    { id: "PR.DS-11", t: "Backups maintained" },
    { id: "ID.RA-01", t: "Vulnerabilities identified" },
    { id: "PR.PS-06", t: "Secure development practices" },
    { id: "GV.OC-03", t: "Legal & regulatory requirements" },
    { id: "DE.CM-06", t: "External provider activity" },
  ];

  const edgesIso: BridgeEdge[] = [
    {
      id: "e2",
      l: 0,
      r: 0,
      rel: "intersects-with",
      rat: "semantic",
      conf: 0.61,
      cov: 0.4,
      st: "complete",
      m: "human",
      by: "R. Vance · GRC",
    },
    {
      id: "e1",
      l: 0,
      r: 1,
      rel: "equivalent-to",
      rat: "functional",
      conf: 0.86,
      cov: 0.9,
      st: "complete",
      m: "human",
      by: "R. Vance · GRC",
    },
    {
      id: "e3",
      l: 1,
      r: 2,
      rel: "equal-to",
      rat: "functional",
      conf: 0.93,
      cov: 1.0,
      st: "complete",
      m: "human",
      by: "T. Mori · Platform",
      sealed: true,
      hash: "sha256:71ae…c402",
    },
    {
      id: "e4",
      l: 2,
      r: 3,
      rel: "superset-of",
      rat: "functional",
      conf: 0.8,
      cov: 0.75,
      st: "complete",
      m: "hybrid",
      by: "D. Okafor · SecEng",
    },
    {
      id: "e5",
      l: 3,
      r: 4,
      rel: "equivalent-to",
      rat: "functional",
      conf: 0.7,
      cov: 0.6,
      st: "draft",
      m: "human",
      by: "R. Vance · GRC",
      disp: 0,
    },
    {
      id: "e6",
      l: 3,
      r: 4,
      rel: "intersects-with",
      rat: "semantic",
      conf: 0.55,
      cov: 0.35,
      st: "draft",
      m: "human",
      by: "D. Okafor · SecEng",
      disp: 1,
    },
    {
      id: "e7",
      l: 4,
      r: 5,
      rel: "subset-of",
      rat: "syntactic",
      conf: 0.45,
      cov: 0.3,
      st: "draft",
      m: "automation",
      by: "atlas-suggest v0.9",
    },
    {
      id: "e8",
      l: 5,
      r: 6,
      rel: "equivalent-to",
      rat: "functional",
      conf: 0.78,
      cov: 0.8,
      st: "complete",
      m: "human",
      by: "K. Ilori · VulnMgmt",
    },
    {
      id: "e9",
      l: 6,
      r: 7,
      rel: "equal-to",
      rat: "functional",
      conf: 0.9,
      cov: 0.95,
      st: "complete",
      m: "human",
      by: "T. Mori · Platform",
    },
    {
      id: "e10",
      l: 7,
      r: 8,
      rel: "intersects-with",
      rat: "semantic",
      conf: 0.58,
      cov: 0.5,
      st: "draft",
      m: "human",
      by: "S. Weil · AppSec",
      qual: "addressable",
    },
    {
      id: "e11",
      l: 7,
      r: 9,
      rel: "intersects-with",
      rat: "functional",
      conf: 0.8,
      cov: 0.4,
      st: "complete",
      m: "human",
      by: "S. Weil · AppSec",
      qual: "blocked",
    },
  ];

  const edgesCsf: BridgeEdge[] = [
    {
      id: "c1",
      l: 0,
      r: 0,
      rel: "intersects-with",
      rat: "functional",
      conf: 0.7,
      cov: 0.6,
      st: "complete",
      m: "human",
      by: "R. Vance · GRC",
    },
    {
      id: "c2",
      l: 1,
      r: 1,
      rel: "subset-of",
      rat: "functional",
      conf: 0.8,
      cov: 0.7,
      st: "complete",
      m: "human",
      by: "T. Mori · Platform",
    },
    {
      id: "c3",
      l: 2,
      r: 2,
      rel: "superset-of",
      rat: "functional",
      conf: 0.75,
      cov: 0.7,
      st: "complete",
      m: "hybrid",
      by: "D. Okafor · SecEng",
    },
    {
      id: "c4",
      l: 3,
      r: 3,
      rel: "equivalent-to",
      rat: "semantic",
      conf: 0.55,
      cov: 0.4,
      st: "draft",
      m: "automation",
      by: "derived via ISO A.8.16",
      der: true,
    },
    {
      id: "c5",
      l: 4,
      r: 4,
      rel: "equivalent-to",
      rat: "functional",
      conf: 0.8,
      cov: 0.8,
      st: "complete",
      m: "human",
      by: "K. Ilori · VulnMgmt",
    },
    {
      id: "c6",
      l: 6,
      r: 5,
      rel: "equal-to",
      rat: "functional",
      conf: 0.85,
      cov: 0.9,
      st: "complete",
      m: "human",
      by: "T. Mori · Platform",
    },
    {
      id: "c7",
      l: 5,
      r: 6,
      rel: "intersects-with",
      rat: "semantic",
      conf: 0.6,
      cov: 0.5,
      st: "draft",
      m: "human",
      by: "K. Ilori · VulnMgmt",
    },
    {
      id: "c8",
      l: 7,
      r: 7,
      rel: "intersects-with",
      rat: "functional",
      conf: 0.72,
      cov: 0.6,
      st: "complete",
      m: "human",
      by: "S. Weil · AppSec",
    },
  ];

  const evidence: EvidenceItem[] = [
    {
      t: "Config scan · prod landing zone",
      by: "evidence-collect@pipeline",
      m: "automation",
      age: "6 min",
      days: 0.004,
      ids: ["SC-7", "CM-6"],
      signed: true,
      hash: "sha256:9f3c…a217",
    },
    {
      t: "MFA coverage query",
      by: "idp-export@pipeline",
      m: "automation",
      age: "55 min",
      days: 0.04,
      ids: ["IA-2"],
      signed: true,
      hash: "sha256:4b18…c390",
    },
    {
      t: "Vulnerability scan delta",
      by: "scanner@pipeline",
      m: "automation",
      age: "3 h",
      days: 0.12,
      ids: ["RA-5", "SI-2"],
      signed: true,
      hash: "sha256:77b1…0ea4",
    },
    {
      t: "SIEM alert-routing test",
      by: "siem-probe@pipeline",
      m: "automation",
      age: "26 h",
      days: 1.1,
      ids: ["AU-6", "IR-4"],
      signed: true,
      hash: "sha256:d81f…2e0b",
    },
    {
      t: "Quarterly access-review export",
      by: "M. Ferreira",
      m: "human",
      age: "12 d",
      days: 12,
      ids: ["AC-2", "AC-6"],
      signed: false,
    },
    {
      t: "Pen-test report 2026-Q2",
      by: "Corvid Labs (external)",
      m: "human",
      age: "74 d",
      days: 74,
      ids: ["SC-7", "SI-4"],
      signed: false,
    },
    {
      t: "Backup restore-drill minutes",
      by: "J. Adeyemi",
      m: "human",
      age: "160 d",
      days: 160,
      ids: ["CP-9"],
      signed: false,
    },
    {
      t: "Screenshot · firewall console",
      by: "unknown uploader",
      m: "human",
      age: "335 d",
      days: 335,
      ids: ["SC-7"],
      signed: false,
    },
  ];

  const poams: PoamItem[] = [
    {
      id: "P-108",
      t: "S3 public-access audit incomplete",
      own: "M. Ferreira",
      due: "overdue 6 d",
      dueN: -6,
      age: 130,
      ids: ["AC-3", "SC-7"],
      impl: 14,
    },
    {
      id: "P-114",
      t: "Legacy VPN lacks MFA",
      own: "K. Ilori",
      due: "due in 9 d",
      dueN: 9,
      age: 84,
      ids: ["IA-2", "AC-17"],
      impl: 13,
    },
    {
      id: "P-127",
      t: "EDR coverage gap on build agents",
      own: "D. Okafor",
      due: "due in 14 d",
      dueN: 14,
      age: 33,
      ids: ["SI-4"],
      impl: 6,
    },
    {
      id: "P-102",
      t: "DR runbook untested for payments",
      own: "J. Adeyemi",
      due: "due in 30 d",
      dueN: 30,
      age: 200,
      ids: ["CP-4"],
      impl: 5,
    },
    {
      id: "P-121",
      t: "Log retention below 12 mo in eu-west",
      own: "T. Mori",
      due: "due in 21 d",
      dueN: 21,
      age: 40,
      ids: ["AU-11"],
      impl: 4,
    },
    {
      id: "P-131",
      t: "Container base images unpinned",
      own: "D. Okafor",
      due: "due in 45 d",
      dueN: 45,
      age: 26,
      ids: ["CM-2"],
      impl: 3,
    },
    {
      id: "P-135",
      t: "Vendor SSO offboarding is manual",
      own: "M. Ferreira",
      due: "due in 90 d",
      dueN: 90,
      age: 12,
      ids: ["AC-2"],
      impl: 2,
    },
  ];

  poams.forEach((p) => {
    p.blast = p.ids.reduce((a, i) => a + (by[i] ? by[i].deps : 0), 0);
    const dn = p.dueN < 0 ? 1 : Math.max(0, 1 - p.dueN / 90);
    p.pressure = Math.min(
      1,
      0.45 * dn +
        0.3 * Math.min(1, p.age / 180) +
        0.25 * Math.min(1, (p.blast || 0) / 20),
    );
  });

  return {
    fams,
    ctrls,
    by,
    corridors,
    mer,
    iso,
    csf,
    edgesIso,
    edgesCsf,
    evidence,
    poams,
  };
}

export function getRelationshipGlyph(rel: string) {
  const G: Record<
    string,
    {
      c1x: number;
      c1r: number;
      c1f: string;
      c2x: number;
      c2r: number;
      c2f: string;
    }
  > = {
    "equal-to": { c1x: 0, c1r: 6.5, c1f: "none", c2x: 0, c2r: 4, c2f: "none" },
    "equivalent-to": {
      c1x: -1.5,
      c1r: 6,
      c1f: "none",
      c2x: 1.5,
      c2r: 6,
      c2f: "none",
    },
    "subset-of": {
      c1x: 1.5,
      c1r: 7.5,
      c1f: "none",
      c2x: -1.5,
      c2r: 3.2,
      c2f: "var(--ck-fg-1)",
    },
    "superset-of": {
      c1x: -1.5,
      c1r: 7.5,
      c1f: "none",
      c2x: 1.5,
      c2r: 3.2,
      c2f: "var(--ck-fg-1)",
    },
    "intersects-with": {
      c1x: -3.5,
      c1r: 5.5,
      c1f: "none",
      c2x: 3.5,
      c2r: 5.5,
      c2f: "none",
    },
    "no-relationship": {
      c1x: -7,
      c1r: 4.5,
      c1f: "none",
      c2x: 7,
      c2r: 4.5,
      c2f: "none",
    },
  };
  return G[rel] || G["intersects-with"];
}

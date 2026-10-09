"use client";

/**
 * Jurisdictions & SLSA.
 *
 * - Built-in jurisdiction catalogs (`catalog list`): excerpt catalogs embedded
 *   in the binary, not complete baselines.
 * - The official NIST SP 800-53 r5 catalog vendored in examples/ (`inspect`).
 * - SLSA / attestation: only what the snapshot actually contains.
 * - FedRAMP validation and schema validation of the sample SSP, shown as-is,
 *   including the known counting defect and the schema-validation failure.
 */

import * as React from "react";
import type {
  BuiltinCatalog,
  CatalogInspectOutput,
  FedrampValidateOutput,
  PipelineRunOutput,
  ValidateOutput,
} from "@/lib/snapshot-types-b";
import {
  DataTable,
  EmptyState,
  PageHeader,
  Panel,
  ReadOnlyNotice,
  StatGrid,
  StatTile,
  StateBadge,
  Terminal,
  type Tone,
} from "@/components/kit";
import {
  Code,
  Fields,
  Note,
  SnapFallback,
  num,
  pretty,
  useSnap,
  type SnapResult,
} from "@/components/generative-ui/b/shared";

import type { LensMode } from "@/lib/oscal-types";

function severityTone(s: string | undefined): Tone {
  const v = (s ?? "").toLowerCase();
  if (v === "high" || v === "critical") return "neg";
  if (v === "medium" || v === "moderate") return "warn";
  if (v === "low") return "info";
  return "unk";
}

export interface JurisdictionSlsaPanelProps {
  lens?: LensMode;
}

export function JurisdictionSlsaPanel({ lens = "ciso" }: JurisdictionSlsaPanelProps) {
  const catalogs = useSnap<BuiltinCatalog[]>("catalog-list");
  const nist = useSnap<CatalogInspectOutput>("nist-catalog-inspect", 1);
  const pipeline = useSnap<PipelineRunOutput>("pipeline-run");
  const fedramp = useSnap<FedrampValidateOutput>("ssp-fedramp-validate");
  const validate = useSnap<ValidateOutput>("ssp-validate", 1);

  const JURISDICTION_LENS_INFO: Record<
    LensMode,
    { title: string; desc: string; tone: "info" | "pos" | "warn" | "neutral" }
  > = {
    ciso: {
      title: "CISO Lens Active",
      desc: "Sovereign jurisdiction catalog baselines, FedRAMP package compliance readiness, and enterprise SLSA attestation.",
      tone: "info",
    },
    assessor: {
      title: "Assessor Lens Active",
      desc: "Auditing SLSA v1.2 build provenance, FedRAMP rule validation errors, and schema conformance diagnostics.",
      tone: "warn",
    },
    architect: {
      title: "Architect Lens Active",
      desc: "Analyzing multi-jurisdiction catalog federation (US NIST vs FedRAMP vs sovereign overlays) and SLSA levels.",
      tone: "info",
    },
    author: {
      title: "Author Lens Active",
      desc: "Reviewing catalog version discrepancies, SSP schema compliance, and jurisdiction inheritance mappings.",
      tone: "neutral",
    },
    "risk-owner": {
      title: "Risk Owner Lens Active",
      desc: "Tracking FedRAMP validation violations, supply chain attestation gaps, and authorization submission blockers.",
      tone: "warn",
    },
    engineer: {
      title: "Engineer Lens Active",
      desc: "Inspecting raw SLSA provenance JSON payloads, Merkle roots, and machine schema validation outputs.",
      tone: "pos",
    },
  };

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="Jurisdictions"
        title="Jurisdictions & SLSA"
        description="Which control catalogs the engine ships, what the official NIST catalog contains, what supply-chain evidence this snapshot holds, and how the sample SSP fares against FedRAMP and OSCAL schema checks."
      />
      <ReadOnlyNotice />

      {/* Active Lens Status Banner */}
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-ck-hairline-strong bg-ck-bg-1 px-3 py-2 text-xs">
        <div className="flex items-center gap-2 min-w-0">
          <StateBadge tone={JURISDICTION_LENS_INFO[lens].tone}>
            {JURISDICTION_LENS_INFO[lens].title}
          </StateBadge>
          <span className="text-ck-fg-2 truncate font-medium">
            {JURISDICTION_LENS_INFO[lens].desc}
          </span>
        </div>
        <div className="flex items-center gap-1.5 shrink-0 text-2xs text-ck-fg-mute font-mono">
          <span>Lens: {lens} (L key cycles)</span>
        </div>
      </div>

      <div className="grid gap-4 2xl:grid-cols-2">
        <CatalogsPanel catalogs={catalogs} />
        <NistPanel nist={nist} />
      </div>

      <SlsaPanel pipeline={pipeline} />

      <div className="grid gap-4 2xl:grid-cols-2">
        <FedrampPanel fedramp={fedramp} />
        <SchemaPanel validate={validate} />
      </div>
    </div>
  );
}

function CatalogsPanel({ catalogs }: { catalogs: SnapResult<BuiltinCatalog[]> }) {
  if (!catalogs.data || !catalogs.provenance) {
    return <SnapFallback title="built-in catalogs" state={catalogs} />;
  }
  return (
    <Panel
      title="Built-in jurisdiction catalogs"
      subtitle="catalog list"
      provenance={catalogs.provenance}
    >
      <div className="space-y-3">
        <DataTable
          caption="Built-in jurisdiction catalogs"
          columns={[
            { key: "code", label: "Code" },
            { key: "jurisdiction", label: "Jurisdiction" },
            { key: "standard", label: "Standard" },
            { key: "count", label: "Controls", numeric: true },
          ]}
          rows={catalogs.data.map((c) => ({
            code: <Code nowrap>{c.code}</Code>,
            jurisdiction: <span className="text-ck-fg-1">{c.jurisdiction ?? "UNKNOWN"}</span>,
            standard: c.standard ?? "UNKNOWN",
            count: num(c.controls_count),
          }))}
        />
        <Note title="Excerpts, not baselines">
          These catalogs are small excerpts embedded in the binary for pipeline gating. They
          are not complete baselines; for example, the full NIST SP 800-53 r5 catalog (next
          panel) contains far more controls than the US excerpt.
        </Note>
      </div>
    </Panel>
  );
}

function NistPanel({ nist }: { nist: SnapResult<CatalogInspectOutput> }) {
  if (!nist.data || !nist.provenance) {
    return <SnapFallback title="NIST SP 800-53 r5 catalog" state={nist} />;
  }
  const n = nist.data;
  const families = Object.keys(n.stats?.controls_by_family ?? {}).length;
  return (
    <Panel
      title="Official NIST SP 800-53 r5 catalog"
      subtitle="Vendored in examples/nist-800-53-r5/ and used by the Atlas"
      provenance={nist.provenance}
    >
      <div className="space-y-3">
        <StatGrid>
          <StatTile label="Controls" value={num(n.stats?.total_controls)} hint="incl. enhancements" />
          <StatTile label="Groups" value={num(n.stats?.total_groups)} />
          <StatTile label="Parameters" value={num(n.stats?.total_params)} />
          <StatTile label="Families listed" value={n.stats?.controls_by_family ? String(families) : "UNKNOWN"} />
        </StatGrid>
        <Fields
          items={[
            { label: "Title", value: n.title ?? "UNKNOWN" },
            { label: "Version", value: n.version ?? "UNKNOWN", mono: true },
            { label: "OSCAL version", value: n.oscal_version ?? "UNKNOWN", mono: true },
            { label: "File", value: n.file ?? "UNKNOWN", mono: true },
          ]}
        />
        <p className="text-xs text-ck-fg-mute">
          Source and digest of this file are recorded in the snapshot manifest inputs. Open the
          Atlas surface to browse the full catalog.
        </p>
      </div>
    </Panel>
  );
}

function SlsaPanel({ pipeline }: { pipeline: SnapResult<PipelineRunOutput> }) {
  const p = pipeline.data;
  if (!p || !pipeline.provenance) {
    return <SnapFallback title="supply-chain evidence" state={pipeline} />;
  }
  const hasAny = p.slsa_provenance_path || p.merkle_root || typeof p.cas_objects_written === "number";
  return (
    <Panel
      title="SLSA and attestation evidence"
      subtitle="Only fields present in the captured pipeline run output"
      provenance={pipeline.provenance}
    >
      <div className="grid gap-4 lg:grid-cols-2">
        <div className="space-y-3">
          {hasAny ? (
            <Fields
              items={[
                { label: "SLSA provenance written to", value: p.slsa_provenance_path ?? "UNKNOWN", mono: true },
                { label: "Merkle root", value: p.merkle_root ?? "UNKNOWN", mono: true },
                { label: "CAS objects written", value: num(p.cas_objects_written), mono: true },
              ]}
            />
          ) : (
            <p className="text-sm text-ck-fg-3">The pipeline output contains no supply-chain fields.</p>
          )}
          <p className="text-xs text-ck-fg-mute">
            These values come from <Code>pipeline run</Code>. A path shows that the run reported
            writing a file; it is not evidence of the file&apos;s contents or of a signature.
          </p>
        </div>
        <EmptyState kind="unknown" title="Attestation contents not captured">
          The SLSA provenance document the pipeline wrote was not copied into this snapshot,
          and <Code>mizan attest slsa</Code> was not captured. Subject digests, builder identity,
          and signature status are therefore not established here.
        </EmptyState>
      </div>
    </Panel>
  );
}

function FedrampPanel({ fedramp }: { fedramp: SnapResult<FedrampValidateOutput> }) {
  if (!fedramp.data || !fedramp.provenance) {
    return <SnapFallback title="FedRAMP validation" state={fedramp} />;
  }
  const f = fedramp.data;
  const findings = f.findings ?? [];
  const findingsCount = typeof f.findings_count === "number" ? f.findings_count : findings.length;
  // Defensive: only shown if a future snapshot reports impossible counts.
  const countDefect =
    typeof f.failed_rules === "number" &&
    typeof f.total_rules_checked === "number" &&
    f.failed_rules > f.total_rules_checked;
  return (
    <Panel
      title="FedRAMP validation: sample SSP"
      subtitle={`Baseline ${f.baseline ?? "UNKNOWN"}, ${f.file ?? ""}`}
      provenance={fedramp.provenance}
      actions={
        typeof f.is_compliant === "boolean" ? (
          <StateBadge tone={f.is_compliant ? "pos" : "neg"}>
            {f.is_compliant ? "compliant" : "not compliant"}
          </StateBadge>
        ) : (
          <StateBadge tone="unk">unknown</StateBadge>
        )
      }
    >
      <div className="space-y-3">
        <p className="text-sm text-ck-fg-2">
          <span className="font-mono ck-num">{num(f.failed_rules)}</span> of{" "}
          <span className="font-mono ck-num">{num(f.total_rules_checked)}</span> rules failed (
          <span className="font-mono ck-num">{findingsCount}</span> findings).
        </p>
        <StatGrid>
          <StatTile label="Rules checked" value={num(f.total_rules_checked)} />
          <StatTile label="Passed" value={num(f.passed_rules)} tone={f.passed_rules ? "pos" : "neutral"} />
          <StatTile label="Failed" value={num(f.failed_rules)} tone={f.failed_rules ? "neg" : "neutral"} />
          <StatTile label="Findings" value={String(findingsCount)} />
        </StatGrid>
        {countDefect && (
          <Note tone="warn" title="Inconsistent counts">
            <Code>failed_rules</Code> ({num(f.failed_rules)}) exceeds{" "}
            <Code>total_rules_checked</Code> ({num(f.total_rules_checked)}). Shown as reported.
          </Note>
        )}
        <DataTable
          caption="FedRAMP findings"
          empty={<p className="text-sm text-ck-fg-3">No findings reported.</p>}
          columns={[
            { key: "rule", label: "Rule" },
            { key: "title", label: "Finding" },
            { key: "severity", label: "Severity" },
          ]}
          rows={findings.map((x) => ({
            rule: <Code nowrap>{x.rule_id}</Code>,
            title: (
              <div className="min-w-0">
                <p className="text-ck-fg-1">{x.title ?? "UNKNOWN"}</p>
                {x.detail && <p className="text-xs text-ck-fg-mute">{x.detail}</p>}
              </div>
            ),
            severity: <StateBadge tone={severityTone(x.severity)}>{x.severity ?? "unknown"}</StateBadge>,
          }))}
        />
      </div>
    </Panel>
  );
}

function SchemaPanel({ validate }: { validate: SnapResult<ValidateOutput> }) {
  if (!validate.data || !validate.provenance) {
    return <SnapFallback title="schema validation" state={validate} />;
  }
  const v = validate.data;
  const diags = v.diagnostics ?? [];
  const errors = diags.filter((x) => (x.level ?? "").toLowerCase() === "error").length;
  return (
    <Panel
      title="OSCAL schema validation: sample SSP"
      subtitle={v.file}
      provenance={validate.provenance}
      actions={
        typeof v.is_valid === "boolean" ? (
          <StateBadge tone={v.is_valid ? "pos" : "neg"}>{v.is_valid ? "valid" : "invalid"}</StateBadge>
        ) : (
          <StateBadge tone="unk">unknown</StateBadge>
        )
      }
    >
      <div className="space-y-3">
        <StatGrid>
          <StatTile
            label="Schema valid"
            value={typeof v.schema_valid === "boolean" ? String(v.schema_valid) : "UNKNOWN"}
            tone={v.schema_valid === false ? "neg" : v.schema_valid ? "pos" : "unk"}
          />
          <StatTile
            label="Constraints valid"
            value={typeof v.constraints_valid === "boolean" ? String(v.constraints_valid) : "UNKNOWN"}
            tone={v.constraints_valid === false ? "neg" : v.constraints_valid ? "pos" : "unk"}
          />
          <StatTile label="Errors" value={String(errors)} tone={errors ? "neg" : "neutral"} />
          <StatTile label="Exit code" value={num(validate.exitCode)} tone={validate.exitCode ? "neg" : "neutral"} />
        </StatGrid>
        <p className="text-sm text-ck-fg-3">
          The repository&apos;s example SSP does not conform to the OSCAL schema. The
          FedRAMP check still ran against it; its result should be read with that in mind.
        </p>
        <ul className="space-y-2">
          {diags.map((x, i) => (
            <li key={i} className="rounded-md border border-ck-hairline bg-ck-bg-0 px-3 py-2">
              <div className="flex flex-wrap items-center gap-2">
                <StateBadge tone={(x.level ?? "").toLowerCase() === "error" ? "neg" : "warn"}>
                  {x.level ?? "unknown"}
                </StateBadge>
                <Code>{x.code ?? "UNKNOWN"}</Code>
              </div>
              <p className="mt-1 text-sm text-ck-fg-1 break-words">{x.message ?? "UNKNOWN"}</p>
              {x.path && <p className="mt-0.5 font-mono text-2xs text-ck-fg-mute break-all">{x.path}</p>}
            </li>
          ))}
        </ul>
        {validate.record?.stderr && (
          <Terminal
            command={validate.command}
            output={`stderr:\n${validate.record.stderr.trim()}`}
            exitCode={validate.exitCode}
            maxHeight={200}
          />
        )}
        <details className="text-sm">
          <summary className="cursor-pointer text-ck-fg-3 hover:text-ck-fg-1">Raw JSON output</summary>
          <div className="mt-2">
            <Terminal command={validate.command} output={pretty(v)} exitCode={validate.exitCode} maxHeight={4000} />
          </div>
        </details>
      </div>
    </Panel>
  );
}

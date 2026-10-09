"use client";

/**
 * Pipeline surface.
 *
 * Renders the real `mizan pipeline run` output captured in the build-time
 * snapshot, plus the SARIF export of its assessment results. Nothing here is
 * re-executed: `pipeline run` writes files and is not on the read-only
 * /api/cli allowlist. When the local engine is LIVE, a separate panel offers
 * allowlisted read-only re-checks.
 */

import * as React from "react";
import { useEngine } from "@/lib/engine";
import type { Provenance } from "@/lib/provenance";
import type {
  PipelineRunOutput,
  Rulepack,
  SarifExportOutput,
} from "@/lib/snapshot-types-b";
import {
  DataTable,
  PageHeader,
  Panel,
  ReadOnlyNotice,
  StatGrid,
  StatTile,
  StateBadge,
  Terminal,
} from "@/components/kit";
import {
  Code,
  Fields,
  Note,
  SnapFallback,
  bool,
  num,
  pretty,
  useSnap,
  type SnapResult,
} from "@/components/generative-ui/b/shared";
import { LiveChecks } from "@/components/surfaces/pipeline/live-checks";

export function PipelineSurface({
  onRecordLocal,
}: {
  onRecordLocal: (event: string) => Promise<void>;
}) {
  const { status } = useEngine();
  const run = useSnap<PipelineRunOutput>("pipeline-run");
  const sarif = useSnap<SarifExportOutput>("pipeline-export-sarif", 2);
  const rules = useSnap<Rulepack[]>("policy-rulepack-list", 3);

  const d = run.data;

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="CI gate"
        title="Pipeline"
        description="Result of one real pipeline run captured when this site was built: built-in policy rules evaluated against the example SBOM, waivers applied, and artifacts written."
        meta={
          d ? (
            <>
              {typeof d.all_passed === "boolean" ? (
                <StateBadge tone={d.all_passed ? "pos" : "neg"}>
                  all_passed {String(d.all_passed)}
                </StateBadge>
              ) : (
                <StateBadge tone="unk">all_passed unknown</StateBadge>
              )}
              {run.exitCode !== null && (
                <StateBadge tone={run.exitCode === 0 ? "neutral" : "neg"} glyph={false}>
                  exit {run.exitCode}
                </StateBadge>
              )}
              {d.jurisdiction && (
                <span className="text-xs text-ck-fg-3">{d.jurisdiction}</span>
              )}
            </>
          ) : undefined
        }
      />
      <ReadOnlyNotice />

      {!d || !run.provenance ? (
        <SnapFallback title="pipeline run output" state={run} />
      ) : (
        <>
          <Panel
            title="Run summary"
            subtitle={d.timestamp ? `Engine timestamp ${d.timestamp}` : undefined}
            provenance={run.provenance}
          >
            <div className="space-y-4">
              <StatGrid>
                <StatTile label="Rules evaluated" value={num(d.evaluated_rules_count)} />
                <StatTile label="Passed" value={num(d.passed_rules_count)} tone="pos" />
                <StatTile label="Waived" value={num(d.waived_rules_count)} tone="warn" />
                <StatTile
                  label="Violations"
                  value={num(d.violations_count)}
                  tone={d.violations_count ? "neg" : "neutral"}
                />
                <StatTile label="SBOM components" value={num(d.sbom_components_count)} />
                <StatTile label="CAS objects written" value={num(d.cas_objects_written)} />
              </StatGrid>
              <Fields
                items={[
                  { label: "Jurisdiction", value: d.jurisdiction ?? "UNKNOWN" },
                  { label: "OSCAL catalog UUID", value: d.oscal_catalog_uuid ?? "UNKNOWN", mono: true },
                  { label: "Merkle root", value: d.merkle_root ?? "UNKNOWN", mono: true },
                  { label: "all_passed", value: bool(d.all_passed), mono: true },
                ]}
              />
            </div>
          </Panel>

          <div className="grid gap-4 2xl:grid-cols-2">
            <GatesPanel run={d} runProv={run.provenance} rules={rules.data} />
            <WaiversPanel run={d} runProv={run.provenance} />
          </div>

          <div className="grid gap-4 2xl:grid-cols-2">
            <Panel
              title="Artifacts produced"
              subtitle="Paths the run reported writing. The files themselves are not part of this snapshot."
              provenance={run.provenance}
            >
              <Fields
                items={[
                  { label: "SLSA provenance", value: d.slsa_provenance_path ?? "UNKNOWN", mono: true },
                  { label: "SARIF report", value: d.sarif_report_path ?? "UNKNOWN", mono: true },
                  { label: "GitLab security report", value: d.gitlab_report_path ?? "UNKNOWN", mono: true },
                  { label: "OSCAL assessment results", value: d.oscal_assessment_path ?? "UNKNOWN", mono: true },
                ]}
              />
            </Panel>
            <SarifPanel
              sarif={sarif}
              violations={d.violations_count}
              waivedIds={(d.active_waivers ?? []).map((w) => w.rule_id)}
            />
          </div>

          <Panel
            title="Command and raw output"
            subtitle="Exactly what the engine printed, pretty-printed."
            provenance={run.provenance}
          >
            <Terminal
              command={run.command}
              output={pretty(d)}
              exitCode={run.exitCode}
              maxHeight={4000}
            />
          </Panel>
        </>
      )}

      {status.kind === "live" && <LiveChecks onRecordLocal={onRecordLocal} />}
    </div>
  );
}

function GatesPanel({
  run,
  runProv,
  rules,
}: {
  run: PipelineRunOutput;
  runProv: Provenance;
  rules: Rulepack[] | null;
}) {
  const waived = new Set((run.active_waivers ?? []).map((w) => w.rule_id));
  const evaluated = run.evaluated_rules_count;
  const passed = run.passed_rules_count;
  const violations = run.violations_count;
  // Per-rule outcomes are only inferable when the counts reconcile exactly
  // with the built-in rulepack list and every violation count is zero.
  const canInfer =
    !!rules &&
    typeof evaluated === "number" &&
    typeof passed === "number" &&
    violations === 0 &&
    rules.length === evaluated &&
    rules.filter((r) => waived.has(r.id)).length + passed === evaluated;

  const rows = (rules ?? []).map((r) => ({
    rule: <Code nowrap>{r.id}</Code>,
    name: <span className="text-ck-fg-1">{r.name ?? "UNKNOWN"}</span>,
    severity: (
      <StateBadge tone="neutral" glyph={false}>
        {r.severity ?? "unknown"}
      </StateBadge>
    ),
    controls: (
      <span className="font-mono text-xs">{(r.target_controls ?? []).join(", ") || "none"}</span>
    ),
    result: waived.has(r.id) ? (
      <StateBadge tone="warn">waived</StateBadge>
    ) : canInfer ? (
      <StateBadge tone="pos">pass (inferred)</StateBadge>
    ) : (
      <StateBadge tone="unk">not itemized</StateBadge>
    ),
  }));

  return (
    <Panel
      title="Gates"
      subtitle="Built-in rulepack rules (policy rulepack list) with the outcome reported by the run."
      provenance={runProv}
    >
      <div className="space-y-3">
        <DataTable
          caption="Policy gates"
          columns={[
            { key: "rule", label: "Rule" },
            { key: "name", label: "Name" },
            { key: "severity", label: "Severity" },
            { key: "controls", label: "Controls" },
            { key: "result", label: "Result" },
          ]}
          rows={rows}
        />
        <Note title="How the result column is derived">
          The run output reports counts and active waivers, not per-rule results. Waived comes
          directly from <Code>active_waivers</Code>.{" "}
          {canInfer
            ? `Pass is inferred: ${num(evaluated)} rules evaluated, ${num(passed)} passed, ${num(violations)} violations, and the rulepack list contains exactly ${rules?.length} rules.`
            : "The counts do not reconcile with the rulepack list, so per-rule outcomes are not shown."}
        </Note>
      </div>
    </Panel>
  );
}

function WaiversPanel({
  run,
  runProv,
}: {
  run: PipelineRunOutput;
  runProv: Provenance;
}) {
  const waivers = run.active_waivers ?? [];
  return (
    <Panel title="Active waivers" subtitle="Waivers the run applied." provenance={runProv}>
      {waivers.length === 0 ? (
        <p className="text-sm text-ck-fg-3">The run reported no active waivers.</p>
      ) : (
        <div className="space-y-4">
          {waivers.map((w) => (
            <div key={w.id} className="space-y-2 rounded-md border border-ck-hairline bg-ck-bg-0 p-3">
              <div className="flex flex-wrap items-center gap-2">
                <Code>{w.id}</Code>
                <StateBadge tone="warn">{w.status ?? "unknown"}</StateBadge>
              </div>
              <Fields
                items={[
                  { label: "Rule", value: w.rule_id, mono: true },
                  { label: "Reason", value: w.reason ?? "UNKNOWN" },
                  { label: "Scope", value: w.scope ?? "UNKNOWN", mono: true },
                  { label: "Author", value: w.author ?? "UNKNOWN" },
                  { label: "Created", value: w.created_at ?? "UNKNOWN", mono: true },
                  { label: "Expires", value: w.expires_at ?? "UNKNOWN", mono: true },
                  { label: "Fingerprint", value: w.fingerprint ?? "UNKNOWN", mono: true },
                ]}
              />
            </div>
          ))}
          <p className="text-xs text-ck-fg-mute">
            The fingerprint is a digest reported by the engine. It is not a signature.
          </p>
        </div>
      )}
    </Panel>
  );
}

function SarifPanel({
  sarif,
  violations,
  waivedIds,
}: {
  sarif: SnapResult<SarifExportOutput>;
  violations: number | undefined;
  waivedIds: string[];
}) {
  if (!sarif.data || !sarif.provenance) {
    return <SnapFallback title="SARIF export" state={sarif} />;
  }
  const s = sarif.data;
  return (
    <Panel
      title="SARIF export"
      subtitle="export sarif run over the pipeline's OSCAL assessment results."
      provenance={sarif.provenance}
    >
      <div className="space-y-3">
        <StatGrid>
          <StatTile label="Results" value={num(s.results_count)} />
          <StatTile label="Rules" value={num(s.rules_count)} />
          <StatTile label="Format" value={<span className="text-sm">{s.format ?? "UNKNOWN"}</span>} />
        </StatGrid>
        <Note title="What this export contains">
          The export reports counts only; the SARIF file is not part of this snapshot, so
          individual results are not shown here. SARIF results are generated from assessment
          findings. This run reported {num(violations)} violations and{" "}
          {waivedIds.length} waived rule{waivedIds.length === 1 ? "" : "s"}
          {waivedIds.length > 0 && (
            <>
              {" "}(<Code>{waivedIds.join(", ")}</Code>)
            </>
          )}
          {s.results_count === waivedIds.length && (violations ?? 0) === 0
            ? ", so the result count matches the waived finding (inferred: the result is the waived rule, exported as suppressed)."
            : "."}
        </Note>
        <Terminal command={sarif.command} output={pretty(s)} exitCode={sarif.exitCode} />
      </div>
    </Panel>
  );
}

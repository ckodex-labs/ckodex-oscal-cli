"use client";

/**
 * Policy Gates & SBOM.
 *
 * - Built-in policy rulepacks (`policy rulepack list`) with their Rego source.
 * - CycloneDX import into an OSCAL component definition (`sbom import`).
 * - Interactive Rego evaluation through /api/eval, only when the local engine
 *   is LIVE. Without it, the panel says so; there is no client-side imitation.
 */

import * as React from "react";
import { useEngine, BASE_PATH } from "@/lib/engine";
import type { Provenance } from "@/lib/provenance";
import type { Rulepack, RulepackEvalOutput, SbomImportOutput } from "@/lib/snapshot-types-b";
import {
  DesktopOnly,
  EmptyState,
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
  btnPrimaryClass,
  num,
  pretty,
  useSnap,
  type SnapResult,
} from "@/components/generative-ui/b/shared";

export function CicdSbomPanel() {
  const { status } = useEngine();
  const rules = useSnap<Rulepack[]>("policy-rulepack-list", 3);
  const sbom = useSnap<SbomImportOutput>("sbom-import");

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="Policy"
        title="Policy Gates & SBOM"
        description="The Rego rules the engine ships for CI gating, and the OSCAL component definition produced from the example CycloneDX SBOM."
      />
      <ReadOnlyNotice />

      <RulepacksPanel rules={rules} />

      <div className="grid gap-4 2xl:grid-cols-2">
        <SbomPanel sbom={sbom} />
        {status.kind === "live" ? (
          <EvalPanel rules={rules.data ?? []} />
        ) : (
          <EvalUnavailable probing={status.kind === "probing"} />
        )}
      </div>
    </div>
  );
}

function RulepacksPanel({ rules }: { rules: SnapResult<Rulepack[]> }) {
  if (!rules.data || !rules.provenance) {
    return <SnapFallback title="policy rulepacks" state={rules} />;
  }
  return (
    <Panel
      title="Built-in policy rulepacks"
      subtitle={`${rules.data.length} rules returned by policy rulepack list`}
      provenance={rules.provenance}
    >
      <ul className="grid gap-3 lg:grid-cols-2">
        {rules.data.map((r) => (
          <li key={r.id} className="min-w-0 space-y-2 rounded-md border border-ck-hairline bg-ck-bg-0 p-3">
            <div className="flex flex-wrap items-center gap-2">
              <Code>{r.id}</Code>
              <StateBadge tone="neutral" glyph={false}>
                {r.severity ?? "unknown"}
              </StateBadge>
            </div>
            <p className="text-sm font-medium text-ck-fg-1">{r.name ?? "UNKNOWN"}</p>
            {r.description && <p className="text-sm text-ck-fg-3">{r.description}</p>}
            <Fields
              items={[
                { label: "Benchmark", value: r.benchmark ?? "UNKNOWN" },
                {
                  label: "Target controls",
                  value: (r.target_controls ?? []).join(", ") || "none",
                  mono: true,
                },
              ]}
            />
            {r.rego_source && (
              <details className="text-sm">
                <summary className="cursor-pointer text-ck-fg-3 hover:text-ck-fg-1">Rego source</summary>
                <pre className="mt-2 overflow-auto whitespace-pre-wrap break-words rounded-md border border-ck-hairline-strong bg-[#111214] p-3 font-mono text-xs leading-5 text-[#e6e4df]">
                  {r.rego_source.trim()}
                </pre>
              </details>
            )}
          </li>
        ))}
      </ul>
    </Panel>
  );
}

function SbomPanel({ sbom }: { sbom: SnapResult<SbomImportOutput> }) {
  if (!sbom.data || !sbom.provenance) {
    return <SnapFallback title="SBOM import" state={sbom} />;
  }
  const s = sbom.data;
  return (
    <Panel
      title="SBOM import"
      subtitle="examples/inventory-sbom.json to an OSCAL component definition"
      provenance={sbom.provenance}
    >
      <div className="space-y-3">
        <StatGrid>
          <StatTile label="Components" value={num(s.component_count)} />
          <StatTile label="Direct dependencies" value={num(s.direct_dependencies)} />
          <StatTile
            label="Input format"
            value={<span className="text-sm">{`${s.format ?? "UNKNOWN"} ${s.spec_version ?? ""}`.trim()}</span>}
          />
        </StatGrid>
        <Fields
          items={[
            { label: "Component definition UUID", value: s.oscal_component_uuid ?? "UNKNOWN", mono: true },
          ]}
        />
        <p className="text-xs text-ck-fg-mute">
          The component definition file was written to the build work directory and is not part
          of this snapshot. The engine reported no vulnerability data for this import.
        </p>
        <Terminal command={sbom.command} output={pretty(s)} exitCode={sbom.exitCode} />
      </div>
    </Panel>
  );
}

function EvalUnavailable({ probing }: { probing: boolean }) {
  return (
    <Panel
      title="Evaluate a rule"
      subtitle="policy rulepack eval through the local engine"
      provenance={{
        kind: "local",
        label: "no engine",
        detail: "Rule evaluation requires the local mizan binary behind next dev.",
      }}
    >
      <EmptyState kind="unknown" title={probing ? "Checking for the local engine" : "Evaluation needs the local engine"}>
        Running a Rego rule against your own input requires <Code>mizan</Code> behind the
        Workbench dev server (<Code>npm run dev</Code> in apps/workbench). This build has only
        the captured snapshot, so no evaluation is performed and no result is shown.
      </EmptyState>
    </Panel>
  );
}

const DEFAULT_INPUT = JSON.stringify(
  {
    apiVersion: "v1",
    kind: "Pod",
    spec: {
      containers: [{ name: "app", securityContext: { runAsNonRoot: true } }],
    },
  },
  null,
  2,
);

type EvalState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "done"; rule: string; data: RulepackEvalOutput; provenance: Provenance }
  | { kind: "error"; message: string };

function EvalPanel({ rules }: { rules: Rulepack[] }) {
  const [rule, setRule] = React.useState<string>("");
  const [input, setInput] = React.useState(DEFAULT_INPUT);
  const [state, setState] = React.useState<EvalState>({ kind: "idle" });
  const selected = rule || rules[0]?.id || "";

  let parseError: string | null = null;
  try {
    JSON.parse(input);
  } catch (e) {
    parseError = e instanceof Error ? e.message : String(e);
  }

  const evaluate = async () => {
    if (!selected || parseError) return;
    setState({ kind: "running" });
    try {
      const res = await fetch(`${BASE_PATH}/api/eval`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ rule: selected, payload: input }),
      });
      const ct = res.headers.get("content-type") ?? "";
      if (!ct.includes("application/json")) {
        throw new Error(`${res.status} ${res.statusText}: non-JSON response from /api/eval`);
      }
      const body = (await res.json()) as { success: boolean; data?: RulepackEvalOutput; error?: string };
      if (!res.ok || !body.success || !body.data) {
        throw new Error(body.error ?? `${res.status} ${res.statusText}`);
      }
      setState({
        kind: "done",
        rule: selected,
        data: body.data,
        provenance: {
          kind: "live",
          label: "mizan policy rulepack eval",
          command: `mizan policy rulepack eval -r ${selected} -i <temp file> --format json`,
          generatedAt: new Date().toISOString(),
        },
      });
    } catch (e) {
      setState({ kind: "error", message: e instanceof Error ? e.message : String(e) });
    }
  };

  const prov: Provenance =
    state.kind === "done"
      ? state.provenance
      : { kind: "local", label: "your input", detail: "Input typed in this browser session; nothing evaluated yet." };

  return (
    <Panel
      title="Evaluate a rule"
      subtitle="policy rulepack eval through the local engine"
      provenance={prov}
    >
      <div className="space-y-3">
        <DesktopOnly>
          <div className="space-y-3">
            <label className="block space-y-1">
              <span className="text-xs text-ck-fg-mute">Rule</span>
              <select
                className="block w-full rounded-md border border-ck-hairline-strong bg-ck-bg-0 px-2 py-1.5 font-mono text-xs text-ck-fg-1"
                value={selected}
                onChange={(e) => setRule(e.target.value)}
              >
                {rules.map((r) => (
                  <option key={r.id} value={r.id}>
                    {r.id}
                  </option>
                ))}
              </select>
            </label>
            <label className="block space-y-1">
              <span className="text-xs text-ck-fg-mute">Input JSON (Kubernetes-style manifest, edit freely)</span>
              <textarea
                className="block h-48 w-full resize-y rounded-md border border-ck-hairline-strong bg-ck-bg-0 p-2 font-mono text-xs text-ck-fg-1"
                spellCheck={false}
                value={input}
                onChange={(e) => setInput(e.target.value)}
              />
            </label>
            {parseError && <p className="text-xs text-ck-neg">Input is not valid JSON: {parseError}</p>}
            <button
              type="button"
              className={btnPrimaryClass}
              onClick={evaluate}
              disabled={!selected || !!parseError || state.kind === "running"}
            >
              {state.kind === "running" ? "Evaluating..." : "Evaluate"}
            </button>
          </div>
        </DesktopOnly>

        {state.kind === "error" && (
          <EmptyState kind="error" title="Evaluation failed">
            {state.message}
          </EmptyState>
        )}
        {state.kind === "done" && (
          <div className="space-y-2">
            <div className="flex flex-wrap items-center gap-2">
              <Code>{state.rule}</Code>
              {typeof state.data.passed === "boolean" ? (
                <StateBadge tone={state.data.passed ? "pos" : "neg"}>
                  {state.data.passed ? "passed" : "denied"}
                </StateBadge>
              ) : (
                <StateBadge tone="unk">unknown</StateBadge>
              )}
            </div>
            {(state.data.findings ?? []).length > 0 && (
              <ul className="list-disc space-y-1 pl-5 text-sm text-ck-fg-2">
                {(state.data.findings ?? []).map((f, i) => (
                  <li key={i} className="break-words">{f}</li>
                ))}
              </ul>
            )}
            <Terminal command={state.provenance.command ?? ""} output={pretty(state.data)} maxHeight={320} />
          </div>
        )}
        <Note title="Scope">
          Evaluates one built-in rule against the input above. It does not change pipeline
          results or waivers.
        </Note>
      </div>
    </Panel>
  );
}

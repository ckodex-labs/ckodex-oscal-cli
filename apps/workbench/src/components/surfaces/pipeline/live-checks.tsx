"use client";

/**
 * LIVE read-only re-checks for the Pipeline surface.
 *
 * `pipeline run` writes files and is NOT on the /api/cli allowlist, so it is
 * never re-run here. Only allowlisted, read-only commands are offered.
 */

import * as React from "react";
import { runLive } from "@/lib/engine";
import type { Provenance } from "@/lib/provenance";
import { local } from "@/lib/provenance";
import { DesktopOnly, EmptyState, Panel, StateBadge, Terminal } from "@/components/kit";
import { btnClass, pretty } from "@/components/generative-ui/b/shared";

const CHECKS: { id: string; title: string; argv: string[] }[] = [
  {
    id: "fedramp",
    title: "FedRAMP validation of the sample SSP",
    argv: ["fedramp", "validate", "examples/sample-ssp.json", "--baseline", "moderate"],
  },
  {
    id: "validate",
    title: "Schema and constraint validation of the sample SSP",
    argv: ["validate", "examples/sample-ssp.json"],
  },
];

type CheckResult =
  | { id: string; ok: true; data: unknown; provenance: Provenance; durationMs: number }
  | { id: string; ok: false; error: string };

export function LiveChecks({
  onRecordLocal,
}: {
  onRecordLocal: (event: string) => Promise<void>;
}) {
  const [running, setRunning] = React.useState(false);
  const [results, setResults] = React.useState<CheckResult[] | null>(null);

  const run = async () => {
    setRunning(true);
    const out: CheckResult[] = [];
    for (const c of CHECKS) {
      try {
        const r = await runLive(c.argv);
        out.push({ id: c.id, ok: true, data: r.data, provenance: r.provenance, durationMs: r.durationMs });
      } catch (e) {
        out.push({ id: c.id, ok: false, error: e instanceof Error ? e.message : String(e) });
      }
    }
    setResults(out);
    setRunning(false);
    const summary = out
      .map((r) => {
        const c = CHECKS.find((x) => x.id === r.id)!;
        return `mizan ${c.argv.join(" ")}: ${r.ok ? "returned output" : `error (${r.error})`}`;
      })
      .join("; ");
    await onRecordLocal(`Re-ran read-only checks against the local engine. ${summary}`);
  };

  return (
    <Panel
      title="Read-only re-checks (local engine)"
      subtitle="Runs allowlisted read-only commands through /api/cli. pipeline run writes files and is not re-run."
      provenance={
        results?.find((r) => r.ok)
          ? (results.find((r) => r.ok) as Extract<CheckResult, { ok: true }>).provenance
          : local("not run yet", "No live command has been executed in this session.")
      }
      actions={
        <DesktopOnly>
          <button type="button" className={btnClass} onClick={run} disabled={running}>
            {running ? "Running..." : "Re-run read-only checks"}
          </button>
        </DesktopOnly>
      }
    >
      {!results ? (
        <EmptyState kind="empty" title="No live checks run in this session">
          The local engine is available. Use the button to run{" "}
          {CHECKS.map((c, i) => (
            <React.Fragment key={c.id}>
              {i > 0 && " and "}
              <code className="font-mono text-xs">mizan {c.argv.join(" ")}</code>
            </React.Fragment>
          ))}
          .
        </EmptyState>
      ) : (
        <div className="space-y-4">
          {results.map((r) => {
            const c = CHECKS.find((x) => x.id === r.id)!;
            return (
              <div key={r.id} className="space-y-2">
                <div className="flex flex-wrap items-center gap-2">
                  <p className="text-sm font-medium text-ck-fg-1">{c.title}</p>
                  {r.ok ? (
                    <StateBadge tone="info">LIVE output</StateBadge>
                  ) : (
                    <StateBadge tone="neg">bridge error</StateBadge>
                  )}
                  {r.ok && (
                    <span className="font-mono text-2xs text-ck-fg-mute">{r.durationMs} ms</span>
                  )}
                </div>
                <Terminal
                  command={`mizan ${c.argv.join(" ")} --format json`}
                  output={r.ok ? pretty(r.data) : r.error}
                  maxHeight={360}
                />
              </div>
            );
          })}
        </div>
      )}
    </Panel>
  );
}

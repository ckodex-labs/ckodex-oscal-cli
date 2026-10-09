"use client";

import * as React from "react";
import { runLive } from "@/lib/engine";
import type { Provenance } from "@/lib/provenance";
import { EmptyState, Panel, Segmented, Terminal, Toolbar } from "@/components/kit";
import type { InspectReport, ValidateReport } from "@/components/generative-ui/engine-types";
import { countByLevel } from "@/components/generative-ui/engine-types";
import { InspectBody } from "./inspect-body";
import { ValidateBody } from "./validate-body";
import { LIVE_PATHS } from "./documents";

type Verb = "inspect" | "validate";

type LiveResult =
  | { verb: "inspect"; path: string; data: InspectReport; provenance: Provenance; exitCode: number }
  | { verb: "validate"; path: string; data: ValidateReport; provenance: Provenance; exitCode: number }
  | { verb: Verb; path: string; error: string };

/**
 * The bridge rejects non-zero exits with "mizan CLI failed (<code>): <stdout|stderr>".
 * `validate` exits 1 for invalid documents but still prints its JSON report,
 * so recover that report when it is present.
 */
function recoverNonZero(msg: string): { exitCode: number; data: unknown } | null {
  const m = msg.match(/^mizan CLI failed \((\d+)\):\s*([\s\S]*)$/);
  if (!m) return null;
  try {
    return { exitCode: Number(m[1]), data: JSON.parse(m[2]) };
  } catch {
    return null;
  }
}

const PATH_RE = /^[A-Za-z0-9._\-/]+$/;

export function LiveRunner({ onRecordLocal }: { onRecordLocal: (event: string) => Promise<void> }) {
  const [verb, setVerb] = React.useState<Verb>("validate");
  const [path, setPath] = React.useState(LIVE_PATHS[2]);
  const [busy, setBusy] = React.useState(false);
  const [result, setResult] = React.useState<LiveResult | null>(null);
  const inputId = React.useId();

  const pathOk = PATH_RE.test(path) && !path.startsWith("/") && !path.split("/").includes("..");

  const run = async () => {
    if (!pathOk || busy) return;
    setBusy(true);
    const argv = [verb, path];
    try {
      const r = await runLive<unknown>(argv);
      if (verb === "inspect") {
        const data = r.data as InspectReport;
        setResult({ verb, path, data, provenance: r.provenance, exitCode: 0 });
        await onRecordLocal(
          `Ran mizan inspect ${path} on the local engine: ${data.kind}, ${data.stats?.total_controls ?? 0} controls (exit 0)`,
        );
      } else {
        const data = r.data as ValidateReport;
        const c = countByLevel(data.diagnostics);
        setResult({ verb, path, data, provenance: r.provenance, exitCode: 0 });
        await onRecordLocal(
          `Ran mizan validate ${path} on the local engine: ${data.is_valid ? "valid" : "invalid"}, ${c.error} errors, ${c.warning} warnings (exit 0)`,
        );
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      const rec = recoverNonZero(msg);
      if (verb === "validate" && rec && typeof rec.data === "object" && rec.data && "is_valid" in rec.data) {
        const data = rec.data as ValidateReport;
        const c = countByLevel(data.diagnostics);
        setResult({
          verb,
          path,
          data,
          exitCode: rec.exitCode,
          provenance: {
            kind: "live",
            label: "mizan validate",
            command: `mizan ${argv.join(" ")} --format json`,
            detail: `Engine exited ${rec.exitCode}; the JSON report was recovered from the bridge error.`,
            generatedAt: new Date().toISOString(),
          },
        });
        await onRecordLocal(
          `Ran mizan validate ${path} on the local engine: ${data.is_valid ? "valid" : "invalid"}, ${c.error} errors, ${c.warning} warnings (exit ${rec.exitCode})`,
        );
      } else {
        setResult({ verb, path, error: msg });
        await onRecordLocal(`mizan ${verb} ${path} failed on the local engine: ${msg.slice(0, 160)}`);
      }
    } finally {
      setBusy(false);
    }
  };

  return (
    <Panel
      title="Run on the local engine"
      subtitle="Read-only inspect or validate of a repository-relative file through the /api/cli bridge."
      provenance={
        result && "provenance" in result
          ? result.provenance
          : { kind: "live", label: "local engine", detail: "The local mizan binary is reachable. No command has run yet." }
      }
      bodyClassName="p-0"
    >
      <form
        className="px-4 py-3"
        onSubmit={(e) => {
          e.preventDefault();
          void run();
        }}
      >
        <Toolbar>
          <Segmented<Verb>
            label="Command"
            value={verb}
            onChange={setVerb}
            options={[
              { value: "inspect", label: "inspect" },
              { value: "validate", label: "validate" },
            ]}
          />
          <label htmlFor={inputId} className="sr-only">
            Repository-relative path
          </label>
          <input
            id={inputId}
            list={`${inputId}-paths`}
            value={path}
            onChange={(e) => setPath(e.target.value)}
            spellCheck={false}
            aria-invalid={!pathOk}
            className="h-8 min-w-0 flex-1 basis-64 rounded-md border border-ck-hairline-strong bg-ck-bg-0 px-2 font-mono text-xs text-ck-fg-1 focus:outline-none focus:ring-1 focus:ring-ck-accent"
          />
          <datalist id={`${inputId}-paths`}>
            {LIVE_PATHS.map((p) => (
              <option key={p} value={p} />
            ))}
          </datalist>
          <button
            type="submit"
            disabled={busy || !pathOk}
            className="h-8 shrink-0 rounded-md border border-ck-fg-1 bg-ck-fg-1 px-3 text-xs font-medium text-ck-bg-0 disabled:opacity-50"
          >
            {busy ? "Running" : "Run"}
          </button>
        </Toolbar>
        {!pathOk && (
          <p className="mt-2 text-xs text-ck-neg">Use a repository-relative path without spaces or "..".</p>
        )}
      </form>
      <div className="border-t border-ck-hairline">
        {!result ? (
          <div className="p-4">
            <EmptyState kind="empty" title="No live result yet">
              Results returned by the local engine appear here with LIVE provenance.
            </EmptyState>
          </div>
        ) : "error" in result ? (
          <div className="p-4">
            <Terminal command={`mizan ${result.verb} ${result.path} --format json`} output={result.error} maxHeight={220} />
          </div>
        ) : result.verb === "inspect" ? (
          <div className="p-4">
            <InspectBody report={result.data} />
          </div>
        ) : (
          <ValidateBody report={result.data} exitCode={result.exitCode} />
        )}
      </div>
    </Panel>
  );
}

"use client";

/**
 * Inspector: document inspection over the build-time snapshot.
 *
 * Shows the real `mizan inspect` and `mizan validate` outputs for the NIST
 * catalog, the resolved Moderate baseline and the sample SSP. When the local
 * engine is reachable, repository files can be inspected or validated live.
 */

import * as React from "react";
import { useEngine, useSnapshot } from "@/lib/engine";
import {
  DesktopOnly,
  EmptyState,
  PageHeader,
  Panel,
  ReadOnlyNotice,
  Segmented,
  StateBadge,
} from "@/components/kit";
import type { InspectReport, ValidateReport } from "@/components/generative-ui/engine-types";
import type { LensMode } from "@/lib/oscal-types";
import { DOCS, type DocId } from "./inspector/documents";
import { InspectBody } from "./inspector/inspect-body";
import { ValidateBody } from "./inspector/validate-body";
import { LiveRunner } from "./inspector/live-runner";

export interface InspectorSurfaceProps {
  onRecordLocal: (event: string) => Promise<void>;
  lens?: LensMode;
}

function missing(id: string, error: string | null) {
  return {
    kind: "fixture" as const,
    label: "missing snapshot",
    detail: `Snapshot '${id}' could not be loaded${error ? `: ${error}` : ""}.`,
  };
}

export function InspectorSurface({
  onRecordLocal,
  lens = "engineer",
}: InspectorSurfaceProps) {
  const { status, manifest } = useEngine();
  const [docId, setDocId] = React.useState<DocId>("catalog");
  const doc = DOCS.find((d) => d.id === docId)!;

  const inspect = useSnapshot<InspectReport>(doc.inspectId);
  const validate = useSnapshot<ValidateReport>(doc.validateId);
  const input = manifest?.inputs.find((i) => i.path === doc.sourceInput);

  const INSPECTOR_LENS_INFO: Record<
    LensMode,
    { title: string; desc: string; tone: "info" | "pos" | "warn" | "neutral" }
  > = {
    engineer: {
      title: "Engineer Lens Active",
      desc: "Inspecting raw document schemas, AST node counts, and CLI command execution diagnostics.",
      tone: "neutral",
    },
    assessor: {
      title: "Assessor Lens Active",
      desc: "Evaluating syntax conformance, constraint validation diagnostics, and schema errors.",
      tone: "warn",
    },
    architect: {
      title: "Architect Lens Active",
      desc: "Verifying catalog and profile composition, inclusion rules, and parameter definitions.",
      tone: "info",
    },
    author: {
      title: "Author Lens Active",
      desc: "Reviewing document prose validity, parameter definition structures, and metadata integrity.",
      tone: "pos",
    },
    "risk-owner": {
      title: "Risk Owner Lens Active",
      desc: "Checking schema compliance status and document readiness for formal authorization submission.",
      tone: "warn",
    },
    ciso: {
      title: "CISO Lens Active",
      desc: "Executive view of core artifact validity, schema health, and engine diagnostic status.",
      tone: "info",
    },
  };

  return (
    <>
      <PageHeader
        eyebrow="Tools"
        title="Inspector"
        description="Document statistics and validation results produced by the engine for the documents in the snapshot."
        meta={
          status.kind === "live" ? (
            <StateBadge tone="pos">Local engine connected</StateBadge>
          ) : status.kind === "snapshot" ? (
            <StateBadge tone="info">Snapshot only</StateBadge>
          ) : status.kind === "offline" ? (
            <StateBadge tone="unk">No engine, no snapshot</StateBadge>
          ) : (
            <StateBadge tone="unk">Checking engine</StateBadge>
          )
        }
      />
      <ReadOnlyNotice />

      {/* Active Lens Status Banner */}
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-ck-hairline-strong bg-ck-bg-1 px-3 py-2 text-xs">
        <div className="flex items-center gap-2 min-w-0">
          <StateBadge tone={INSPECTOR_LENS_INFO[lens].tone}>
            {INSPECTOR_LENS_INFO[lens].title}
          </StateBadge>
          <span className="text-ck-fg-2 truncate font-medium">
            {INSPECTOR_LENS_INFO[lens].desc}
          </span>
        </div>
        <div className="flex items-center gap-1.5 shrink-0 text-2xs text-ck-fg-mute font-mono">
          <span>Lens: {lens} (L key cycles)</span>
        </div>
      </div>

      <div className="flex min-w-0 flex-wrap items-center gap-3">
        <Segmented<DocId>
          label="Document"
          value={docId}
          onChange={setDocId}
          options={DOCS.map((d) => ({ value: d.id, label: d.label }))}
        />
        {input && (
          <p className="min-w-0 text-xs text-ck-fg-3">
            Source <span className="break-all font-mono">{input.path}</span>{" "}
            <span className="font-mono text-ck-fg-mute">sha256:{input.sha256.slice(0, 12)}</span>
            {input.source && <span> from {input.source}</span>}
          </p>
        )}
      </div>
      {doc.note && <p className="text-sm text-ck-fg-3">{doc.note}</p>}

      <div className="grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
        <Panel title="Inspect" subtitle="mizan inspect" provenance={inspect.provenance ?? missing(doc.inspectId, inspect.error)}>
          {inspect.loading ? (
            <p className="text-sm text-ck-fg-3">Loading snapshot.</p>
          ) : inspect.data ? (
            <InspectBody report={inspect.data} />
          ) : (
            <EmptyState kind="unknown" title="Inspect output unavailable">
              {inspect.error ?? "The snapshot has no inspect output for this document."}
            </EmptyState>
          )}
        </Panel>

        <Panel
          title="Validate"
          subtitle="mizan validate: schema and constraint diagnostics"
          provenance={validate.provenance ?? missing(doc.validateId, validate.error)}
          bodyClassName="p-0"
        >
          {validate.loading ? (
            <p className="p-4 text-sm text-ck-fg-3">Loading snapshot.</p>
          ) : validate.data ? (
            <ValidateBody report={validate.data} exitCode={validate.exitCode} />
          ) : (
            <div className="p-4">
              <EmptyState kind="unknown" title="Validate output unavailable">
                {validate.error ?? "The snapshot has no validate output for this document."}
              </EmptyState>
            </div>
          )}
        </Panel>
      </div>

      <DesktopOnly>
        {status.kind === "live" ? (
          <LiveRunner onRecordLocal={onRecordLocal} />
        ) : (
          <p className="rounded-md border border-ck-hairline bg-ck-bg-1 px-4 py-3 text-sm text-ck-fg-3">
            Running inspect or validate on other files needs the local engine (
            <code className="font-mono text-xs">next dev</code> with the <code className="font-mono text-xs">mizan</code>{" "}
            binary available). This build is showing snapshot data only.
          </p>
        )}
      </DesktopOnly>
    </>
  );
}

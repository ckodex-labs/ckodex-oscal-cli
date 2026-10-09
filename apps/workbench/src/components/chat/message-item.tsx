"use client";

import * as React from "react";
import type { EngineData } from "@/lib/engine";
import type { BlastRadiusReport, FedrampReport } from "@/lib/oscal-types";
import { StateBadge } from "@/components/kit";
import { AnswerCard } from "../generative-ui/answer-card";
import { ControlCard } from "../generative-ui/control-card";
import { BlastRadiusCard } from "../generative-ui/blast-radius-card";
import { FedrampBadgeCard } from "../generative-ui/fedramp-badge-card";
import { ValidateCard } from "../generative-ui/validate-card";
import type { AtlasFile, ValidateReport } from "../generative-ui/engine-types";
import { COMMAND_HELP, type Entry, type ValidateDoc } from "./commands";

export interface AnswerSources {
  atlas: EngineData<AtlasFile>;
  blast: EngineData<BlastRadiusReport>;
  fedramp: EngineData<FedrampReport>;
  validate: Record<ValidateDoc, EngineData<ValidateReport>>;
}

function Note({ tone, children }: { tone: "info" | "warn" | "neg" | "unk"; children: React.ReactNode }) {
  const label = { info: "Note", warn: "Limit", neg: "Error", unk: "Unknown" }[tone];
  return (
    <div className="flex min-w-0 flex-wrap items-baseline gap-2 text-sm text-ck-fg-2">
      <StateBadge tone={tone}>{label}</StateBadge>
      <span className="min-w-0">{children}</span>
    </div>
  );
}

/** Render an EngineData source: loading, missing, or the given card. */
function FromSource<T>({
  source,
  what,
  children,
}: {
  source: EngineData<T>;
  what: string;
  children: (data: T, s: EngineData<T>) => React.ReactNode;
}) {
  if (source.loading) return <Note tone="info">Loading {what} from the snapshot.</Note>;
  if (!source.data || !source.provenance) {
    return (
      <Note tone="unk">
        {what} is not available: {source.error ?? "no data"}.
      </Note>
    );
  }
  return <>{children(source.data, source)}</>;
}

function HelpCard() {
  return (
    <div className="space-y-1 text-sm text-ck-fg-2">
      <p>Available commands. Each answer is read from captured engine output and carries its provenance.</p>
      <ul className="space-y-0.5">
        {COMMAND_HELP.map((c) => (
          <li key={c.cmd} className="flex min-w-0 flex-wrap gap-x-2">
            <code className="font-mono text-xs text-ck-fg-1">{c.cmd}</code>
            <span className="text-xs text-ck-fg-3">{c.what}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

export function MessageItem({
  entry,
  sources,
  onNavigateControl,
}: {
  entry: Entry;
  sources: AnswerSources;
  onNavigateControl?: (id: string) => void;
}) {
  const a = entry.answer;
  let body: React.ReactNode;
  switch (a.kind) {
    case "help":
      body = <HelpCard />;
      break;
    case "control":
      body = (
        <FromSource source={sources.atlas} what="The catalog projection">
          {(atlas, s) => {
            const c = atlas.controls.find((x) => x.id === a.id);
            if (!c) {
              return (
                <AnswerCard title="Not found" provenance={s.provenance!}>
                  <p>
                    No control <span className="font-mono">{a.id}</span> among the{" "}
                    <span className="font-mono">{atlas.controls.length}</span> controls in the catalog projection.
                  </p>
                </AnswerCard>
              );
            }
            return <ControlCard control={c} provenance={s.provenance!} onOpen={onNavigateControl} />;
          }}
        </FromSource>
      );
      break;
    case "blast-snapshot":
      body = (
        <FromSource source={sources.blast} what="The blast-radius capture">
          {(r, s) => <BlastRadiusCard report={r} provenance={s.provenance!} />}
        </FromSource>
      );
      break;
    case "blast-live":
      body = <BlastRadiusCard report={a.report} provenance={a.provenance} />;
      break;
    case "fedramp":
      body = (
        <FromSource source={sources.fedramp} what="The FedRAMP capture">
          {(r, s) => <FedrampBadgeCard report={r} provenance={s.provenance!} />}
        </FromSource>
      );
      break;
    case "validate":
      body = (
        <FromSource source={sources.validate[a.doc]} what="The validation capture">
          {(r, s) => <ValidateCard report={r} provenance={s.provenance!} exitCode={s.exitCode} />}
        </FromSource>
      );
      break;
    case "pending":
      body = <Note tone="info">{a.text}</Note>;
      break;
    case "note":
      body = <Note tone={a.tone}>{a.text}</Note>;
      break;
  }

  return (
    <li className="min-w-0 space-y-1.5">
      <p className="font-mono text-xs text-ck-fg-3">
        <span className="text-ck-fg-mute" aria-hidden>
          {">"}{" "}
        </span>
        {entry.input}
      </p>
      {body}
    </li>
  );
}

"use client";

import * as React from "react";
import type { Provenance } from "@/lib/provenance";
import type { BlastRadiusReport, ImpactedNode } from "@/lib/oscal-types";
import { StateBadge } from "@/components/kit";
import { AnswerCard, Fact } from "./answer-card";

function asNode(n: string | ImpactedNode): ImpactedNode {
  return typeof n === "string" ? { id: n } : n;
}

function NodeList({ label, nodes }: { label: string; nodes: (string | ImpactedNode)[] }) {
  return (
    <div className="min-w-0">
      <p className="ck-eyebrow">
        {label} <span className="font-mono ck-num">{nodes.length}</span>
      </p>
      {nodes.length === 0 ? (
        <p className="text-xs text-ck-fg-mute">None reported.</p>
      ) : (
        <ul className="mt-1 space-y-0.5">
          {nodes.map((raw, i) => {
            const n = asNode(raw);
            return (
              <li key={`${n.id}-${i}`} className="flex min-w-0 flex-wrap items-baseline gap-x-2 text-xs">
                <span className="text-ck-fg-1">{n.title ?? n.id}</span>
                {n.kind && <span className="text-ck-fg-mute">{n.kind}</span>}
                {n.relation && <span className="font-mono text-2xs text-ck-fg-mute">{n.relation}</span>}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}

/** Output of `mizan blast-radius --target <id> <doc>`. */
export function BlastRadiusCard({
  report,
  provenance,
}: {
  report: BlastRadiusReport;
  provenance: Provenance;
}) {
  return (
    <AnswerCard
      title={
        <span>
          Blast radius of <span className="font-mono">{report.target_id}</span>
        </span>
      }
      provenance={provenance}
    >
      <dl className="grid grid-cols-1 gap-1 sm:grid-cols-2">
        <Fact label="Target kind">{report.target_kind}</Fact>
        <Fact label="Risk exposure score">
          <span className="font-mono ck-num">{report.risk_exposure_score}</span>
        </Fact>
        <Fact label="Critical path">
          {report.is_critical_path ? (
            <StateBadge tone="warn">Yes</StateBadge>
          ) : (
            <StateBadge tone="neutral">No</StateBadge>
          )}
        </Fact>
        <Fact label="Documents analyzed">
          <span className="font-mono text-xs">{(report.documents_analyzed ?? []).join(", ") || "none"}</span>
        </Fact>
      </dl>
      <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
        <NodeList label="Direct dependents" nodes={report.direct_dependents ?? []} />
        <NodeList label="Transitive dependents" nodes={report.transitive_dependents ?? []} />
      </div>
    </AnswerCard>
  );
}

"use client";

import * as React from "react";
import type { Provenance } from "@/lib/provenance";
import { StateBadge } from "@/components/kit";
import { AnswerCard, Fact } from "./answer-card";
import type { AtlasControl } from "./engine-types";

/** One control from the NIST catalog projection (atlas.json). */
export function ControlCard({
  control,
  provenance,
  onOpen,
}: {
  control: AtlasControl;
  provenance: Provenance;
  onOpen?: (id: string) => void;
}) {
  const params = control.params ?? [];
  return (
    <AnswerCard
      title={
        <span>
          <span className="font-mono">{control.id.toUpperCase()}</span>{" "}
          <span className="font-normal">{control.title}</span>
        </span>
      }
      provenance={provenance}
      actions={
        onOpen ? (
          <button
            type="button"
            onClick={() => onOpen(control.id)}
            className="rounded-sm border border-ck-hairline-strong px-1.5 py-0.5 text-xs text-ck-fg-2 hover:text-ck-fg-1"
          >
            Open in Atlas
          </button>
        ) : null
      }
    >
      <div className="flex flex-wrap gap-1.5">
        {control.in_baseline ? (
          <StateBadge tone="pos">In Moderate baseline</StateBadge>
        ) : (
          <StateBadge tone="neutral">Not in Moderate baseline</StateBadge>
        )}
        {control.withdrawn && <StateBadge tone="warn">Withdrawn</StateBadge>}
        {control.parent && (
          <StateBadge tone="neutral" glyph={false}>
            enhancement of <span className="font-mono">{control.parent}</span>
          </StateBadge>
        )}
      </div>
      {control.statement ? (
        <p className="text-sm leading-5 text-ck-fg-2">{control.statement}</p>
      ) : (
        <p className="text-xs text-ck-fg-mute">No statement part in the catalog for this control.</p>
      )}
      <dl>
        <Fact label="Parameters">
          <span className="font-mono">{params.length}</span>
          {params.length > 0 && (
            <span className="text-xs text-ck-fg-3">
              {" "}
              ({params.slice(0, 3).map((p) => p.label ?? p.id).join("; ")}
              {params.length > 3 ? "; ..." : ""})
            </span>
          )}
        </Fact>
      </dl>
    </AnswerCard>
  );
}

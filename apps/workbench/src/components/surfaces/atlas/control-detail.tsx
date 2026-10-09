"use client";

import * as React from "react";
import { EmptyState, ProvenanceTag, StateBadge } from "@/components/kit";
import type { Provenance } from "@/lib/provenance";
import type { SspImplementedRequirement } from "@/lib/atlas-types";
import { displayId, implState, statusTone, type AtlasIndex } from "./model";
import { StatementText } from "./statement";
import { BTN_PRIMARY, LINK_BTN } from "./ui";

function Row({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <>
      <dt className="ck-eyebrow pt-0.5">{label}</dt>
      <dd className="min-w-0 text-sm text-ck-fg-2">{children}</dd>
    </>
  );
}

export function ControlDetail({
  index,
  id,
  byControl,
  sspProvenance,
  sspError,
  onSelect,
  onOpenInComposer,
}: {
  index: AtlasIndex;
  id: string;
  byControl: Map<string, SspImplementedRequirement>;
  sspProvenance: Provenance | null;
  sspError: string | null;
  onSelect: (id: string) => void;
  onOpenInComposer: (id: string) => void;
}) {
  const c = index.byId.get(id);
  if (!c) {
    return (
      <EmptyState
        kind="unknown"
        title={`Control '${id}' is not in the catalog projection`}
      >
        atlas.json lists {index.atlas.counts.catalogControls} controls; this id
        is not one of them. Select a control in the grid.
      </EmptyState>
    );
  }
  const family = index.familyById.get(c.family);
  const parent = c.parent ? index.byId.get(c.parent) : undefined;
  const kids = index.children.get(c.id) ?? [];
  const kidsBaseline = kids.filter((k) => k.in_baseline);
  const kidsWithdrawn = kids.filter((k) => k.withdrawn).length;
  const s = implState(c.id, byControl);

  return (
    <div className="space-y-4">
      <div className="space-y-1.5">
        <div className="flex flex-wrap items-baseline gap-x-2">
          <span className="font-mono text-lg font-semibold text-ck-fg-1">
            {displayId(c.id)}
          </span>
          <span className="font-mono text-xs text-ck-fg-mute">{c.id}</span>
        </div>
        <p
          className={
            c.withdrawn
              ? "text-md text-ck-fg-3 line-through"
              : "text-md text-ck-fg-1"
          }
        >
          {c.title}
        </p>
        <div className="flex flex-wrap gap-1.5">
          {c.in_baseline ? (
            <StateBadge tone="info">In Moderate baseline</StateBadge>
          ) : (
            <StateBadge tone="neutral">Not in Moderate baseline</StateBadge>
          )}
          {c.withdrawn && <StateBadge tone="neutral">Withdrawn</StateBadge>}
        </div>
      </div>

      <dl className="grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-2">
        <Row label="Family">
          <span className="font-mono text-xs">{c.family.toUpperCase()}</span>{" "}
          {family?.title ?? "UNKNOWN"}
        </Row>
        {parent && (
          <Row label="Parent">
            <button
              type="button"
              className={LINK_BTN}
              onClick={() => onSelect(parent.id)}
            >
              {displayId(parent.id)}
            </button>{" "}
            <span className="text-ck-fg-3">{parent.title}</span>
          </Row>
        )}
        {!c.parent && (
          <Row label="Enhancements">
            <span className="ck-num">
              {kidsBaseline.length} in baseline, {kids.length} in catalog
              {kidsWithdrawn > 0 && ` (${kidsWithdrawn} withdrawn)`}
            </span>
            {kidsBaseline.length > 0 && (
              <span className="mt-1 flex flex-wrap gap-x-2 gap-y-0.5">
                {kidsBaseline.map((k) => (
                  <button
                    key={k.id}
                    type="button"
                    className={LINK_BTN}
                    title={k.title}
                    onClick={() => onSelect(k.id)}
                  >
                    {displayId(k.id)}
                  </button>
                ))}
              </span>
            )}
          </Row>
        )}
      </dl>

      <div className="space-y-1.5">
        <h3 className="ck-eyebrow">Statement</h3>
        {c.statement !== undefined ? (
          <StatementText statement={c.statement} params={c.params ?? []} />
        ) : (
          <EmptyState
            kind="unknown"
            title={
              c.withdrawn
                ? "Withdrawn control"
                : "Statement not in this projection"
            }
          >
            {c.withdrawn
              ? "Withdrawn controls have no statement in the catalog."
              : "atlas.json carries statements and parameters only for the 287 baseline controls."}
          </EmptyState>
        )}
      </div>

      {c.params && (
        <div className="space-y-1.5">
          <h3 className="ck-eyebrow">
            Parameters <span className="ck-num">({c.params.length})</span>
          </h3>
          {c.params.length === 0 ? (
            <p className="text-sm text-ck-fg-3">
              This control declares no parameters.
            </p>
          ) : (
            <ul className="space-y-1">
              {c.params.map((p) => (
                <li key={p.id} className="text-sm">
                  <span className="font-mono text-2xs text-ck-fg-mute break-all">
                    {p.id}
                  </span>
                  <span className="block text-ck-fg-2">{p.label}</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}

      <div className="space-y-1.5 rounded-md border border-ck-hairline bg-ck-bg-0 p-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h3 className="ck-eyebrow">SSP implementation</h3>
          {sspProvenance && <ProvenanceTag provenance={sspProvenance} />}
        </div>
        {sspError ? (
          <p className="text-sm text-ck-fg-3">
            SSP status unavailable: {sspError}
          </p>
        ) : s.kind === "undeclared" ? (
          <>
            <StateBadge tone="unk">Not declared</StateBadge>
            <p className="text-xs text-ck-fg-3">
              The sample SSP does not list this control. Undeclared is not
              failed.
            </p>
          </>
        ) : (
          <>
            {s.kind === "declared" ? (
              <StateBadge tone={statusTone(s.status)}>{s.status}</StateBadge>
            ) : (
              <StateBadge tone="unk">Declared, status EMPTY</StateBadge>
            )}
            <p className="text-xs text-ck-fg-3">
              Implemented requirement{" "}
              <span className="font-mono break-all">{s.req.uuid}</span>,{" "}
              <span className="ck-num">{s.req.by_components.length}</span>{" "}
              component
              {s.req.by_components.length === 1 ? "" : "s"}.
              {s.kind === "declared-empty" &&
                " The SSP states no implementation-status."}
            </p>
          </>
        )}
      </div>

      <div className="space-y-1">
        <button
          type="button"
          className={BTN_PRIMARY}
          disabled={!c.in_baseline}
          onClick={() => onOpenInComposer(c.id)}
        >
          Open in Composer
        </button>
        {!c.in_baseline && (
          <p className="text-xs text-ck-fg-mute">
            Composer edits baseline controls only.
          </p>
        )}
      </div>
    </div>
  );
}

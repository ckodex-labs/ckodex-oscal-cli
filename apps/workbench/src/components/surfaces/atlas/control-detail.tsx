"use client";

import * as React from "react";
import { EmptyState, ProvenanceTag, StateBadge } from "@/components/kit";
import type { Provenance } from "@/lib/provenance";
import type { SspImplementedRequirement } from "@/lib/atlas-types";
import type { LensMode } from "@/lib/oscal-types";
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

// Authoritative family corridors for architectural analysis
const FAMILY_CORRIDORS_MAP: Record<string, string[]> = {
  ac: ["ia", "au", "sc", "cm", "pe", "ps", "ma"],
  ia: ["ac"],
  au: ["ac", "si"],
  sc: ["ac", "si", "ir"],
  cm: ["ac", "cp", "ca", "sa"],
  cp: ["cm"],
  ca: ["cm", "pl"],
  pl: ["ca"],
  pm: [],
  pt: [],
  pe: ["ac"],
  ps: ["ac"],
  ma: ["ac"],
  mp: [],
  sa: ["cm", "sr"],
  sr: ["sa", "ra"],
  si: ["sc", "ra", "au"],
  ra: ["si", "sr"],
  ir: ["sc", "at"],
  at: ["ir"],
};

export function ControlDetail({
  index,
  id,
  byControl,
  sspProvenance,
  sspError,
  onSelect,
  onOpenInComposer,
  lens = "architect",
}: {
  index: AtlasIndex;
  id: string;
  byControl: Map<string, SspImplementedRequirement>;
  sspProvenance: Provenance | null;
  sspError: string | null;
  onSelect: (id: string) => void;
  onOpenInComposer: (id: string) => void;
  lens?: LensMode;
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

      {/* Lens Perspective Intelligence Box */}
      <div className="rounded-md border border-ck-hairline-strong bg-ck-bg-1 p-2.5 space-y-2">
        <div className="flex items-center justify-between">
          <span className="ck-eyebrow text-ck-accent">Lens · {lens.toUpperCase()}</span>
          <span className="font-mono text-2xs text-ck-fg-mute">perspective</span>
        </div>

        {lens === "architect" && (
          <div className="space-y-1.5 text-xs text-ck-fg-2">
            <p className="font-medium text-ck-fg-1">Hexagonal Topology Corridors:</p>
            {FAMILY_CORRIDORS_MAP[c.family]?.length ? (
              <div className="flex flex-wrap gap-1">
                {FAMILY_CORRIDORS_MAP[c.family].map((targetFam) => (
                  <span
                    key={targetFam}
                    className="inline-flex items-center gap-1 rounded bg-ck-bg-0 border border-ck-hairline px-1.5 py-0.5 font-mono text-2xs"
                  >
                    <span className="font-bold text-ck-accent">{c.family.toUpperCase()}</span>
                    <span>&harr;</span>
                    <span className="font-semibold">{targetFam.toUpperCase()}</span>
                  </span>
                ))}
              </div>
            ) : (
              <p className="text-2xs text-ck-fg-mute">No direct boundary corridors mapped for this family.</p>
            )}
            <p className="text-2xs text-ck-fg-3">
              Coupling: {c.family.toUpperCase()} connects to {FAMILY_CORRIDORS_MAP[c.family]?.length ?? 0} adjacent family clusters in the Moderate baseline.
            </p>
          </div>
        )}

        {lens === "author" && (
          <div className="space-y-1.5 text-xs text-ck-fg-2">
            <p className="font-medium text-ck-fg-1">Prose & Parameter Tailoring:</p>
            <p className="text-2xs text-ck-fg-3">
              {c.params && c.params.length > 0
                ? `Declares ${c.params.length} configurable parameter${c.params.length === 1 ? "" : "s"} requiring organizational assignment.`
                : "Standard control with no parameter tailoring required."}
            </p>
            <button
              type="button"
              className={BTN_PRIMARY}
              disabled={!c.in_baseline}
              onClick={() => onOpenInComposer(c.id)}
            >
              Tailor in Composer
            </button>
          </div>
        )}

        {lens === "engineer" && (
          <div className="space-y-1 text-xs text-ck-fg-2 font-mono">
            <p className="text-2xs text-ck-fg-mute font-sans font-medium">CLI Inspection Syntax:</p>
            <pre className="rounded bg-ck-bg-0 border border-ck-hairline p-1.5 text-2xs text-ck-fg-1 overflow-x-auto">
              ckx oscal control inspect --id {c.id}
            </pre>
            <div className="text-2xs text-ck-fg-3 font-sans">
              AST Node: {c.id} &middot; Family: {c.family} &middot; Baseline: {String(c.in_baseline)}
            </div>
          </div>
        )}

        {lens === "assessor" && (
          <div className="space-y-1 text-xs text-ck-fg-2">
            <p className="font-medium text-ck-fg-1">Assessment Audit Rubric:</p>
            <div className="flex items-center gap-2">
              <span className="text-2xs text-ck-fg-mute">Declared Status:</span>
              {s.kind === "declared" ? (
                <StateBadge tone={statusTone(s.status)}>{s.status}</StateBadge>
              ) : s.kind === "declared-empty" ? (
                <StateBadge tone="unk">EMPTY</StateBadge>
              ) : (
                <StateBadge tone="neutral">UNDECLARED</StateBadge>
              )}
            </div>
            <p className="text-2xs text-ck-fg-3">
              FedRAMP Moderate test objective: verifies implementation presence and evidence artifact bindings.
            </p>
          </div>
        )}

        {lens === "risk-owner" && (
          <div className="space-y-1 text-xs text-ck-fg-2">
            <p className="font-medium text-ck-fg-1">Risk Exposure & Residual Impact:</p>
            <p className="text-2xs text-ck-fg-3">
              Baseline weight: Critical path control in Moderate profile. If unsatisfied, generates audit finding with required POA&amp;M milestone.
            </p>
          </div>
        )}

        {lens === "ciso" && (
          <div className="space-y-1 text-xs text-ck-fg-2">
            <p className="font-medium text-ck-fg-1">Executive Governance Rollup:</p>
            <p className="text-2xs text-ck-fg-3">
              Family {c.family.toUpperCase()} has {family?.in_baseline ?? 0} controls in the Moderate baseline. Contributes to organizational compliance authorization package.
            </p>
          </div>
        )}
      </div>

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

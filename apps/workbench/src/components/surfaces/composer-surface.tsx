"use client";

/**
 * Composer: read a baseline control as resolved by the engine and draft local
 * parameter values.
 *
 * Data:
 *   atlas.json                 (nist-moderate-resolve)  statement, params, context
 *   nist-moderate-validate.json(nist-moderate-validate) resolved catalog validity
 *   nist-catalog-validate.json (nist-catalog-validate)  source catalog validity
 *
 * Drafts are LOCAL component state. They are never saved, committed, or
 * signed; recording one only appends an unsigned entry to the session log.
 */

import * as React from "react";
import {
  DesktopOnly,
  EmptyState,
  PageHeader,
  Panel,
  ProvenanceTag,
  ReadOnlyNotice,
  StateBadge,
  Toolbar,
} from "@/components/kit";
import { cn } from "@/lib/utils";
import { useSnapshot } from "@/lib/engine";
import { local } from "@/lib/provenance";
import type {
  AtlasControl,
  AtlasParam,
  ValidateResult,
} from "@/lib/atlas-types";
import type { LensMode } from "@/lib/oscal-types";
import {
  displayId,
  matchesQuery,
  normalizeQuery,
  useAtlas,
  type AtlasIndex,
} from "./atlas/model";
import { StatementText } from "./atlas/statement";
import { BTN, BTN_PRIMARY, INPUT, LINK_BTN } from "./atlas/ui";

export interface ComposerSurfaceProps {
  selectedControlId: string;
  onSelectControl: (id: string) => void;
  onRecordLocal: (event: string) => Promise<void>;
  lens?: LensMode;
}

type Drafts = Record<string, Record<string, string>>;

const LOCAL_PROV = local(
  "this session",
  "Draft parameter values typed in this browser view. Not saved, not committed, not signed; discarded when you leave Composer.",
);

export function ComposerSurface({
  selectedControlId,
  onSelectControl,
  onRecordLocal,
  lens = "author",
}: ComposerSurfaceProps) {
  const atlasSnap = useAtlas();
  const modValidate = useSnapshot<ValidateResult>("nist-moderate-validate");
  const catValidate = useSnapshot<ValidateResult>("nist-catalog-validate");

  const [query, setQuery] = React.useState("");
  const [drafts, setDrafts] = React.useState<Drafts>({});
  const [activeParam, setActiveParam] = React.useState<string | null>(null);
  const [recorded, setRecorded] = React.useState<Set<string>>(() => new Set());
  const [recordError, setRecordError] = React.useState<string | null>(null);
  const editorRef = React.useRef<HTMLInputElement>(null);

  React.useEffect(() => {
    setActiveParam(null);
  }, [selectedControlId]);

  const header = (
    <PageHeader
      eyebrow="Authoring"
      title="Composer"
      description="Read a control as tailored by the Moderate baseline and draft local parameter values. Drafts stay in this view; nothing here is saved, committed, or signed."
    />
  );

  if (atlasSnap.loading) {
    return (
      <>
        {header}
        <p role="status" className="text-sm text-ck-fg-3">
          Loading engine snapshot...
        </p>
      </>
    );
  }
  const index = atlasSnap.index;
  if (!index || !atlasSnap.provenance) {
    return (
      <>
        {header}
        <EmptyState kind="error" title="Atlas snapshot unavailable">
          {atlasSnap.error ?? "atlas.json did not contain a control list."} Run{" "}
          <code className="font-mono text-xs">npm run snapshot</code> in
          apps/workbench.
        </EmptyState>
      </>
    );
  }
  const prov = atlasSnap.provenance;
  const control = index.byId.get(selectedControlId);
  const inBaseline = !!control?.in_baseline;
  const params = control?.params ?? [];
  const ctlDrafts = drafts[selectedControlId] ?? {};
  const draftEntries = params
    .filter((p) => (ctlDrafts[p.id] ?? "").trim() !== "")
    .map((p) => ({ param: p, value: ctlDrafts[p.id].trim() }));

  const setDraft = (paramId: string, value: string) =>
    setDrafts((prev) => ({
      ...prev,
      [selectedControlId]: {
        ...(prev[selectedControlId] ?? {}),
        [paramId]: value,
      },
    }));

  const openEditor = (paramId: string) => {
    setActiveParam(paramId);
    requestAnimationFrame(() => editorRef.current?.focus());
  };

  const record = async (entries: { param: AtlasParam; value: string }[]) => {
    setRecordError(null);
    try {
      for (const e of entries) {
        await onRecordLocal(
          `Draft for ${selectedControlId}: ${e.param.id}=${e.value}`,
        );
        setRecorded((prev) =>
          new Set(prev).add(`${selectedControlId}|${e.param.id}|${e.value}`),
        );
      }
    } catch (err) {
      setRecordError(err instanceof Error ? err.message : String(err));
    }
  };

  const active = params.find((p) => p.id === activeParam) ?? null;

  const COMPOSER_LENS_INFO: Record<
    LensMode,
    { title: string; desc: string; tone: "info" | "pos" | "warn" | "neutral" }
  > = {
    author: {
      title: "Author Lens Active",
      desc: "Drafting implementation narrative, configuring control parameter values, and applying organizational assignments.",
      tone: "pos",
    },
    engineer: {
      title: "Engineer Lens Active",
      desc: "OSCAL GitOps workspace split, YAML frontmatter schema, and machine-readable assembly.",
      tone: "neutral",
    },
    assessor: {
      title: "Assessor Lens Active",
      desc: "Auditing control tailoring rationale, parameter boundary compliance, and verification objectives.",
      tone: "warn",
    },
    architect: {
      title: "Architect Lens Active",
      desc: "Evaluating baseline control tailoring against component boundaries and service architecture.",
      tone: "info",
    },
    "risk-owner": {
      title: "Risk Owner Lens Active",
      desc: "Reviewing residual exposure resulting from parameter tailoring and baseline exceptions.",
      tone: "warn",
    },
    ciso: {
      title: "CISO Lens Active",
      desc: "Executive oversight of organizational tailoring policies, mandatory baselines, and parameter deviations.",
      tone: "info",
    },
  };

  return (
    <>
      {header}
      <ReadOnlyNotice />

      {/* Active Lens Status Banner */}
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-ck-hairline-strong bg-ck-bg-1 px-3 py-2 text-xs">
        <div className="flex items-center gap-2 min-w-0">
          <StateBadge tone={COMPOSER_LENS_INFO[lens].tone}>
            {COMPOSER_LENS_INFO[lens].title}
          </StateBadge>
          <span className="text-ck-fg-2 truncate font-medium">
            {COMPOSER_LENS_INFO[lens].desc}
          </span>
        </div>
        <div className="flex items-center gap-1.5 shrink-0 text-2xs text-ck-fg-mute font-mono">
          <span>Lens: {lens} (L key cycles)</span>
        </div>
      </div>

      <ControlPicker
        index={index}
        query={query}
        onQuery={setQuery}
        selectedId={selectedControlId}
        onSelect={onSelectControl}
      />

      <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(280px,320px)] lg:items-start">
        <div className="min-w-0 space-y-4">
          <Panel
            title={
              control ? (
                <span>
                  <span className="font-mono">{displayId(control.id)}</span>{" "}
                  {control.title}
                </span>
              ) : (
                "Statement"
              )
            }
            subtitle={
              inBaseline
                ? "Statement as resolved for the Moderate baseline. Click a parameter to draft a value."
                : undefined
            }
            provenance={prov}
          >
            {!control ? (
              <EmptyState
                kind="unknown"
                title={`Control '${selectedControlId}' not found`}
              >
                It is not among the {index.atlas.counts.catalogControls}{" "}
                controls in atlas.json. Pick a baseline control above.
              </EmptyState>
            ) : !inBaseline || control.statement === undefined ? (
              <EmptyState
                kind="unknown"
                title={`${displayId(control.id)} is not in the Moderate baseline`}
              >
                Composer works on the {index.atlas.counts.baselineControls}{" "}
                baseline controls; statements and parameters are projected only
                for those. Pick a baseline control above.
              </EmptyState>
            ) : (
              <div className="space-y-4">
                <StatementText
                  statement={control.statement}
                  params={params}
                  renderParam={(p) => (
                    <ParamButton
                      param={p}
                      draft={(ctlDrafts[p.id] ?? "").trim()}
                      active={p.id === activeParam}
                      onClick={() => openEditor(p.id)}
                    />
                  )}
                />

                <div className="space-y-1.5">
                  <h3 className="ck-eyebrow">
                    Parameters <span className="ck-num">({params.length})</span>
                  </h3>
                  {params.length === 0 ? (
                    <p className="text-sm text-ck-fg-3">
                      This control declares no parameters.
                    </p>
                  ) : (
                    <ul className="divide-y divide-ck-hairline rounded-md border border-ck-hairline">
                      {params.map((p) => {
                        const d = (ctlDrafts[p.id] ?? "").trim();
                        return (
                          <li key={p.id}>
                            <button
                              type="button"
                              onClick={() => openEditor(p.id)}
                              aria-pressed={p.id === activeParam}
                              className={cn(
                                "flex w-full min-w-0 flex-col items-start gap-0.5 px-3 py-2 text-left hover:bg-ck-bg-0 sm:flex-row sm:items-baseline sm:gap-3",
                                p.id === activeParam && "bg-ck-bg-0",
                              )}
                            >
                              <span
                                className="shrink-0 font-mono text-2xs text-ck-fg-mute sm:w-36 sm:truncate"
                                title={p.id}
                              >
                                {p.id}
                              </span>
                              <span className="min-w-0 flex-1 text-sm text-ck-fg-2">
                                {p.label}
                              </span>
                              {d && (
                                <StateBadge tone="unk" glyph={false}>
                                  Local draft
                                </StateBadge>
                              )}
                            </button>
                          </li>
                        );
                      })}
                    </ul>
                  )}
                </div>

                <DesktopOnly>
                  <div className="rounded-md border border-dashed border-ck-hairline-strong bg-ck-bg-0 p-3">
                    {active ? (
                      <div className="space-y-2">
                        <div className="flex flex-wrap items-center justify-between gap-2">
                          <label
                            htmlFor="composer-draft"
                            className="text-sm font-medium text-ck-fg-1"
                          >
                            Draft value for{" "}
                            <span className="font-mono text-xs">
                              {active.id}
                            </span>
                          </label>
                          <StateBadge tone="unk" glyph={false}>
                            Local draft, not saved
                          </StateBadge>
                        </div>
                        <p className="text-xs text-ck-fg-3">
                          Catalog label: {active.label}
                        </p>
                        <div className="flex flex-wrap gap-2">
                          <input
                            ref={editorRef}
                            id="composer-draft"
                            type="text"
                            value={ctlDrafts[active.id] ?? ""}
                            onChange={(e) =>
                              setDraft(active.id, e.target.value)
                            }
                            placeholder={active.label}
                            className={cn(INPUT, "flex-1 basis-56")}
                          />
                          <button
                            type="button"
                            className={BTN}
                            onClick={() => setDraft(active.id, "")}
                          >
                            Clear
                          </button>
                          <button
                            type="button"
                            className={BTN}
                            onClick={() => setActiveParam(null)}
                          >
                            Done
                          </button>
                        </div>
                      </div>
                    ) : (
                      <p className="text-sm text-ck-fg-3">
                        Select a parameter in the statement or the list to type
                        a draft value.
                      </p>
                    )}
                  </div>
                </DesktopOnly>
              </div>
            )}
          </Panel>

          {inBaseline && (
            <Panel
              title="Local drafts"
              subtitle="Difference between the catalog parameter label and your draft value."
              provenance={LOCAL_PROV}
            >
              {draftEntries.length === 0 ? (
                <EmptyState
                  kind="empty"
                  title={`No drafts for ${displayId(selectedControlId)}`}
                >
                  Draft values you type appear here. They are held in this view
                  only.
                </EmptyState>
              ) : (
                <div className="space-y-3">
                  <ul className="space-y-2">
                    {draftEntries.map(({ param, value }) => {
                      const isRecorded = recorded.has(
                        `${selectedControlId}|${param.id}|${value}`,
                      );
                      return (
                        <li
                          key={param.id}
                          className="min-w-0 rounded-md border border-ck-hairline bg-ck-bg-0"
                        >
                          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-ck-hairline px-3 py-1.5">
                            <span className="font-mono text-2xs text-ck-fg-2 break-all">
                              {param.id}
                            </span>
                            <div className="flex flex-wrap items-center gap-2">
                              {isRecorded && (
                                <StateBadge tone="unk" glyph={false}>
                                  In session log, unsigned
                                </StateBadge>
                              )}
                              <DesktopOnly>
                                <button
                                  type="button"
                                  className={LINK_BTN}
                                  onClick={() =>
                                    void record([{ param, value }])
                                  }
                                >
                                  Record
                                </button>
                              </DesktopOnly>
                            </div>
                          </div>
                          <div className="font-mono text-xs">
                            <p className="flex gap-2 px-3 py-1 text-ck-fg-3">
                              <span aria-hidden className="select-none">
                                -
                              </span>
                              <span className="min-w-0 break-words">
                                <span className="sr-only">Catalog label: </span>
                                {param.label}
                              </span>
                            </p>
                            <p className="flex gap-2 bg-ck-bg-2 px-3 py-1 text-ck-fg-1">
                              <span aria-hidden className="select-none">
                                +
                              </span>
                              <span className="min-w-0 break-words">
                                <span className="sr-only">Local draft: </span>
                                {value}
                              </span>
                            </p>
                          </div>
                        </li>
                      );
                    })}
                  </ul>
                  <p className="text-xs text-ck-fg-mute">
                    - catalog parameter label (atlas.json) / + local draft (this
                    view). The session log entry is a SHA-256 of the text, not a
                    signature.
                  </p>
                  <DesktopOnly>
                    <div className="flex flex-wrap items-center gap-2">
                      <button
                        type="button"
                        className={BTN_PRIMARY}
                        onClick={() => void record(draftEntries)}
                      >
                        Record draft in session log
                      </button>
                      <span className="text-xs text-ck-fg-3">
                        Appends {draftEntries.length} LOCAL entr
                        {draftEntries.length === 1 ? "y" : "ies"}; nothing is
                        written to the catalog.
                      </span>
                    </div>
                  </DesktopOnly>
                  {recordError && (
                    <p className="text-sm text-ck-neg">
                      Could not record: {recordError}
                    </p>
                  )}
                </div>
              )}
            </Panel>
          )}
        </div>

        <div className="min-w-0 space-y-4">
          <Panel title="Baseline context" provenance={prov}>
            {control ? (
              <BaselineContext
                index={index}
                control={control}
                onSelect={onSelectControl}
              />
            ) : (
              <EmptyState kind="unknown" title="No control selected" />
            )}
          </Panel>

          {modValidate.loading ? (
            <p role="status" className="text-sm text-ck-fg-3">
              Loading validation snapshot...
            </p>
          ) : !modValidate.data || !modValidate.provenance ? (
            <EmptyState kind="error" title="Validation snapshot unavailable">
              {modValidate.error ?? "No data"}
            </EmptyState>
          ) : (
            <Panel
              title="Engine validation"
              subtitle="mizan validate on the resolved Moderate catalog"
              provenance={modValidate.provenance}
            >
              <div className="space-y-4">
                <ValidateSummary
                  result={modValidate.data}
                  label="Resolved Moderate catalog"
                />
                {catValidate.data && (
                  <div className="space-y-2 border-t border-ck-hairline pt-3">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <span className="ck-eyebrow">Source catalog</span>
                      {catValidate.provenance && (
                        <ProvenanceTag provenance={catValidate.provenance} />
                      )}
                    </div>
                    <ValidateSummary
                      result={catValidate.data}
                      label="NIST SP 800-53 r5 catalog"
                    />
                  </div>
                )}
              </div>
            </Panel>
          )}
        </div>
      </div>
    </>
  );
}

/* ------------------------------------------------------------------ */

function ControlPicker({
  index,
  query,
  onQuery,
  selectedId,
  onSelect,
}: {
  index: AtlasIndex;
  query: string;
  onQuery: (q: string) => void;
  selectedId: string;
  onSelect: (id: string) => void;
}) {
  const nq = normalizeQuery(query);
  const options = index.baseline.filter(
    (c) => matchesQuery(c, nq) || c.id === selectedId,
  );
  const selectedInBaseline = index.baseline.some((c) => c.id === selectedId);
  const matchCount = index.baseline.filter((c) => matchesQuery(c, nq)).length;
  return (
    <Toolbar className="rounded-lg border border-ck-hairline-strong bg-ck-bg-1 p-3">
      <label htmlFor="composer-search" className="ck-eyebrow w-full sm:w-auto">
        Control
      </label>
      <input
        id="composer-search"
        type="search"
        value={query}
        onChange={(e) => onQuery(e.target.value)}
        placeholder="Filter by id or title"
        aria-label="Filter baseline controls by id or title"
        className={cn(INPUT, "w-full sm:w-52")}
      />
      <select
        aria-label="Baseline control"
        value={selectedInBaseline ? selectedId : ""}
        onChange={(e) => e.target.value && onSelect(e.target.value)}
        className={cn(
          INPUT,
          "w-full sm:w-auto sm:min-w-[16rem] sm:max-w-[24rem]",
        )}
      >
        {!selectedInBaseline && (
          <option value="">Select a baseline control</option>
        )}
        {options.map((c) => (
          <option key={c.id} value={c.id}>
            {displayId(c.id)} {c.title}
          </option>
        ))}
      </select>
      <span className="font-mono text-2xs text-ck-fg-mute ck-num">
        {matchCount} of {index.baseline.length} baseline controls
      </span>
    </Toolbar>
  );
}

function ParamButton({
  param,
  draft,
  active,
  onClick,
}: {
  param: AtlasParam;
  draft: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={active}
      title={
        draft
          ? `Local draft: ${draft} (catalog label: ${param.label})`
          : `${param.id}: ${param.label}`
      }
      className={cn(
        "mx-0.5 inline rounded-sm border px-1 py-px text-left text-sm [box-decoration-break:clone]",
        draft
          ? "border-dashed border-ck-accent bg-ck-bg-0 text-ck-fg-1"
          : "border-ck-hairline-strong bg-ck-bg-2 text-ck-fg-2 hover:border-ck-fg-3",
        active && "shadow-[0_0_0_2px_var(--ck-accent)]",
      )}
    >
      {draft ? (
        <>
          <span className="sr-only">Local draft: </span>
          {draft}
        </>
      ) : (
        param.label
      )}
    </button>
  );
}

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

function BaselineContext({
  index,
  control,
  onSelect,
}: {
  index: AtlasIndex;
  control: AtlasControl;
  onSelect: (id: string) => void;
}) {
  const fam = index.familyById.get(control.family);
  const parent = control.parent ? index.byId.get(control.parent) : undefined;
  const rootId = control.parent ?? control.id;
  const kids = index.children.get(rootId) ?? [];
  const kidsBaseline = kids.filter((k) => k.in_baseline);
  const kidsWithdrawn = kids.filter((k) => k.withdrawn).length;
  return (
    <dl className="grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-2">
      <Row label="Baseline">
        {control.in_baseline ? (
          <StateBadge tone="info">In Moderate</StateBadge>
        ) : (
          <StateBadge tone="neutral">Not in Moderate</StateBadge>
        )}
        {control.withdrawn && (
          <StateBadge tone="neutral" className="ml-1">
            Withdrawn
          </StateBadge>
        )}
      </Row>
      <Row label="Family">
        <span className="font-mono text-xs">
          {control.family.toUpperCase()}
        </span>{" "}
        {fam?.title ?? "UNKNOWN"}
        {fam && (
          <span className="block text-xs text-ck-fg-mute ck-num">
            {fam.in_baseline} of {fam.total_in_catalog} family controls in
            baseline
          </span>
        )}
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
          {parent.title}
        </Row>
      )}
      <Row label={parent ? "Siblings" : "Enhancements"}>
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
                title={k.title}
                className={cn(
                  LINK_BTN,
                  k.id === control.id && "font-semibold text-ck-fg-1",
                )}
                onClick={() => onSelect(k.id)}
              >
                {displayId(k.id)}
              </button>
            ))}
          </span>
        )}
      </Row>
      {control.params && (
        <Row label="Params">
          <span className="ck-num">{control.params.length}</span>
        </Row>
      )}
    </dl>
  );
}

function ValidateSummary({
  result,
  label,
}: {
  result: ValidateResult;
  label: string;
}) {
  const warnings = result.diagnostics.filter(
    (d) => d.level.toLowerCase() === "warning",
  ).length;
  const errors = result.diagnostics.filter(
    (d) => d.level.toLowerCase() === "error",
  ).length;
  return (
    <div className="space-y-2">
      <p className="text-sm text-ck-fg-1">{label}</p>
      <div className="flex flex-wrap gap-1.5">
        <StateBadge tone={result.is_valid ? "pos" : "neg"}>
          {result.is_valid ? "Valid" : "Invalid"}
        </StateBadge>
        <StateBadge tone={result.schema_valid ? "pos" : "neg"} glyph={false}>
          schema {result.schema_valid ? "ok" : "fail"}
        </StateBadge>
        <StateBadge
          tone={result.constraints_valid ? "pos" : "neg"}
          glyph={false}
        >
          constraints {result.constraints_valid ? "ok" : "fail"}
        </StateBadge>
      </div>
      <p className="text-xs text-ck-fg-3 ck-num">
        {result.diagnostics.length} diagnostic
        {result.diagnostics.length === 1 ? "" : "s"}
        {result.diagnostics.length > 0 &&
          ` (${errors} error, ${warnings} warning)`}
      </p>
      {result.diagnostics.length > 0 && (
        <ul className="space-y-1">
          {result.diagnostics.slice(0, 5).map((d, i) => (
            <li
              key={i}
              className="rounded-md border border-ck-hairline bg-ck-bg-0 px-2 py-1.5 text-xs"
            >
              <StateBadge
                tone={d.level.toLowerCase() === "error" ? "neg" : "warn"}
              >
                {d.level}
              </StateBadge>{" "}
              <span className="font-mono text-2xs text-ck-fg-2 break-all">
                {d.code}
              </span>
              <span className="mt-0.5 block break-words text-ck-fg-3">
                {d.message}
              </span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

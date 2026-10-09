"use client";

/**
 * Atlas: the NIST SP 800-53 Rev 5 Moderate baseline as resolved by the
 * engine, laid out by family, with the implementation status declared by the
 * sample SSP. Every number on this surface comes from a snapshot file:
 *
 *   atlas.json                 (nist-moderate-resolve)  catalog + baseline
 *   ssp-status.json            (ssp-inspect)            SSP implementation
 *   nist-catalog-inspect.json  (nist-catalog-inspect)   catalog version
 */

import * as React from "react";
import {
  EmptyState,
  Legend,
  PageHeader,
  Panel,
  ProvenanceTag,
  Segmented,
  StateBadge,
  StatGrid,
  StatTile,
  Toolbar,
} from "@/components/kit";
import { useSnapshot } from "@/lib/engine";
import type { InspectResult } from "@/lib/atlas-types";
import type { LensMode } from "@/lib/oscal-types";
import {
  displayId,
  implLabel,
  implState,
  matchesQuery,
  normalizeQuery,
  useAtlas,
  useSspStatus,
} from "./atlas/model";
import {
  CELL_CLASS,
  FamilyGrid,
  Swatch,
  WITHDRAWN_STYLE,
  statusCellClass,
  type GridMode,
} from "./atlas/family-grid";
import { ControlDetail } from "./atlas/control-detail";
import { TopologyMap } from "./atlas/topology-map";
import { INPUT } from "./atlas/ui";

export interface AtlasSurfaceProps {
  selectedId: string;
  onSelectControl: (id: string) => void;
  onOpenInComposer: (id: string) => void;
  lens?: LensMode;
}

export function AtlasSurface({
  selectedId,
  onSelectControl,
  onOpenInComposer,
  lens = "architect",
}: AtlasSurfaceProps) {
  const atlasSnap = useAtlas();
  const ssp = useSspStatus();
  const catInspect = useSnapshot<InspectResult>("nist-catalog-inspect");

  const [mode, setMode] = React.useState<GridMode>("baseline");
  const [viewType, setViewType] = React.useState<"topology" | "grid">(
    lens === "architect" ? "topology" : "grid",
  );
  const [pulseActive, setPulseActive] = React.useState(false);
  const [query, setQuery] = React.useState("");
  const [family, setFamily] = React.useState<string>("all");
  const [hoverId, setHoverId] = React.useState<string | null>(null);
  const detailRef = React.useRef<HTMLDivElement>(null);

  // Synchronize viewType when lens switches to architect
  React.useEffect(() => {
    if (lens === "architect") {
      setViewType("topology");
    }
  }, [lens]);

  const triggerPulse = React.useCallback(() => {
    setPulseActive(true);
    const timer = setTimeout(() => setPulseActive(false), 2500);
    return () => clearTimeout(timer);
  }, []);

  const index = atlasSnap.index;
  const nq = normalizeQuery(query);

  const modeSet = React.useMemo(
    () =>
      index
        ? mode === "baseline"
          ? index.baseline
          : index.atlas.controls
        : [],
    [index, mode],
  );
  const visible = React.useMemo(
    () =>
      modeSet.filter(
        (c) => (family === "all" || c.family === family) && matchesQuery(c, nq),
      ),
    [modeSet, family, nq],
  );

  const select = React.useCallback(
    (id: string) => {
      onSelectControl(id);
      // Below the lg breakpoint the detail panel sits under the grid.
      if (
        typeof window !== "undefined" &&
        !window.matchMedia("(min-width: 1024px)").matches
      ) {
        requestAnimationFrame(() =>
          detailRef.current?.scrollIntoView({
            behavior: "smooth",
            block: "start",
          }),
        );
      }
    },
    [onSelectControl],
  );

  const header = (meta?: React.ReactNode) => (
    <PageHeader
      eyebrow="Catalog"
      title="Atlas"
      description="NIST SP 800-53 Rev 5 Moderate baseline, resolved by the engine from the official NIST catalog."
      meta={meta}
    />
  );

  if (atlasSnap.loading) {
    return (
      <>
        {header()}
        <p role="status" className="text-sm text-ck-fg-3">
          Loading engine snapshot...
        </p>
      </>
    );
  }

  if (!index || !atlasSnap.provenance) {
    return (
      <>
        {header()}
        <EmptyState kind="error" title="Atlas snapshot unavailable">
          {atlasSnap.error ?? "atlas.json did not contain a control list."} Run{" "}
          <code className="font-mono text-xs">npm run snapshot</code> in
          apps/workbench.
        </EmptyState>
      </>
    );
  }

  const prov = atlasSnap.provenance;
  const { counts } = index.atlas;

  // SSP coverage over the baseline, from ssp-status.json.
  const baselineIds = new Set(index.baseline.map((c) => c.id));
  const declared = Array.from(ssp.byControl.values()).filter((r) =>
    baselineIds.has(r.control_id.toLowerCase()),
  );
  const declaredWithStatus = declared.filter(
    (r) => r.status_presence === "PRESENT" && r.implementation_status,
  ).length;
  const withdrawnInBaseline = index.baseline.filter((c) => c.withdrawn).length;

  // Legend counts over the current mode set (unfiltered).
  let undeclared = 0;
  let declaredEmpty = 0;
  let notInBaseline = 0;
  let withdrawn = 0;
  const statusCounts = new Map<string, number>();
  for (const c of modeSet) {
    if (c.withdrawn) {
      withdrawn += 1;
      continue;
    }
    if (!c.in_baseline) {
      notInBaseline += 1;
      continue;
    }
    const s = implState(c.id, ssp.byControl);
    if (s.kind === "undeclared") undeclared += 1;
    else if (s.kind === "declared-empty") declaredEmpty += 1;
    else statusCounts.set(s.status, (statusCounts.get(s.status) ?? 0) + 1);
  }
  const legend = [
    {
      swatch: <Swatch className={CELL_CLASS.undeclared} />,
      label: "No implementation declared",
      count: undeclared,
    },
    {
      swatch: <Swatch className={CELL_CLASS.declaredEmpty} />,
      label: "Declared in SSP, status EMPTY",
      count: declaredEmpty,
    },
    ...Array.from(statusCounts.entries()).map(([status, count]) => ({
      swatch: <Swatch className={statusCellClass(status)} />,
      label: `Declared: ${status}`,
      count,
    })),
    ...(mode === "catalog"
      ? [
          {
            swatch: <Swatch className={CELL_CLASS.notInBaseline} />,
            label: "Not in Moderate baseline",
            count: notInBaseline,
          },
          {
            swatch: (
              <Swatch
                className={CELL_CLASS.withdrawn}
                style={WITHDRAWN_STYLE}
              />
            ),
            label: "Withdrawn",
            count: withdrawn,
          },
        ]
      : []),
    {
      swatch: <Swatch className={CELL_CLASS.selected} />,
      label: "Selected",
    },
  ];

  const families =
    mode === "baseline"
      ? index.atlas.families.filter((f) => f.in_baseline > 0)
      : index.atlas.families;

  const focusId = hoverId ?? selectedId;
  const focusCtl = index.byId.get(focusId);

  const LENS_METADATA: Record<
    LensMode,
    { title: string; desc: string; tone: "info" | "pos" | "warn" | "neutral" }
  > = {
    architect: {
      title: "Architect Lens Active",
      desc: "Visualizing hexagonal family topology, cross-family corridors, and dependency boundaries.",
      tone: "info",
    },
    author: {
      title: "Author Lens Active",
      desc: "Focusing on control statement prose, parameter tailoring values, and SSP implementation narrative.",
      tone: "pos",
    },
    engineer: {
      title: "Engineer Lens Active",
      desc: "Inspecting OSCAL machine AST schemas, node definitions, and CLI automation syntax.",
      tone: "neutral",
    },
    assessor: {
      title: "Assessor Lens Active",
      desc: "Auditing SSP implementation evidence statuses, declaration presence, and test objectives.",
      tone: "warn",
    },
    "risk-owner": {
      title: "Risk Owner Lens Active",
      desc: "Mapping critical path controls, blast radius exposure, and POA&M remediation debt.",
      tone: "warn",
    },
    ciso: {
      title: "CISO Lens Active",
      desc: "Executive baseline posture rollup, family completion ratios, and governance compliance.",
      tone: "info",
    },
  };

  return (
    <>
      {header(
        <>
          {catInspect.data ? (
            <span className="ck-hash">catalog {catInspect.data.version}</span>
          ) : (
            <span className="ck-hash">catalog version UNKNOWN</span>
          )}
          <span className="ck-hash">
            baseline {counts.baselineControls} controls,{" "}
            {counts.baselineFamilies} families
          </span>
          {catInspect.provenance && (
            <ProvenanceTag provenance={catInspect.provenance} />
          )}
        </>,
      )}

      {/* Active Lens Status Banner */}
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-ck-hairline-strong bg-ck-bg-1 px-3 py-2 text-xs">
        <div className="flex items-center gap-2 min-w-0">
          <StateBadge tone={LENS_METADATA[lens].tone}>
            {LENS_METADATA[lens].title}
          </StateBadge>
          <span className="text-ck-fg-2 truncate font-medium">
            {LENS_METADATA[lens].desc}
          </span>
        </div>
        <div className="flex items-center gap-1.5 shrink-0 text-2xs text-ck-fg-mute font-mono">
          <span>Lens: {lens} (L key cycles)</span>
        </div>
      </div>

      <Panel
        title={lens === "architect" ? "Architectural Topology & Coverage" : "Coverage"}
        subtitle={
          lens === "architect"
            ? "Topological corridor links, family clusters, and baseline resolution."
            : lens === "author"
              ? "Authoring parameter density, baseline scope, and tailoring status."
              : lens === "assessor"
                ? "Assessment implementation presence, declaration states, and catalog scope."
                : "Counts from the resolved baseline and the catalog projection."
        }
        provenance={prov}
      >
        <StatGrid>
          <StatTile
            label="Controls in baseline"
            value={counts.baselineControls}
            hint="NIST Moderate"
          />
          {lens === "architect" ? (
            <>
              <StatTile
                label="Active corridors"
                value={18}
                hint="Cross-family paths"
              />
              <StatTile
                label="Boundary hubs"
                value={20}
                hint="Family clusters"
              />
              <StatTile
                label="Max coupling"
                value="AC (7)"
                hint="Boundary corridors"
              />
            </>
          ) : (
            <>
              <StatTile
                label="Baseline families"
                value={counts.baselineFamilies}
                hint={`of ${index.atlas.families.length} in catalog`}
              />
              <StatTile
                label="Controls in catalog"
                value={counts.catalogControls}
                hint="incl. enhancements"
              />
              <StatTile
                label="Withdrawn in catalog"
                value={index.withdrawnCount}
                hint={`${withdrawnInBaseline} in baseline`}
              />
            </>
          )}
          <StatTile
            label="SSP coverage"
            tone="unk"
            value={
              ssp.data ? (
                <>
                  {declared.length}
                  <span className="text-sm font-normal text-ck-fg-mute">
                    {" "}
                    of {counts.baselineControls}
                  </span>
                </>
              ) : (
                "UNKNOWN"
              )
            }
            hint={
              ssp.data
                ? `declared; status ${declaredWithStatus === 0 ? "EMPTY" : `on ${declaredWithStatus}`}`
                : (ssp.error ?? "ssp-status.json unavailable")
            }
          />
        </StatGrid>
        {ssp.data && (
          <div className="mt-3 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-ck-fg-3">
            <span>
              SSP coverage: {ssp.data.systemTitle} declares{" "}
              {ssp.data.implemented_requirements.length} implemented requirement
              {ssp.data.implemented_requirements.length === 1 ? "" : "s"}
              {declaredWithStatus === 0 &&
                " and states no implementation-status"}
              . Controls it does not list are undeclared, not failed.
            </span>
            {ssp.provenance && <ProvenanceTag provenance={ssp.provenance} />}
          </div>
        )}
      </Panel>

      <div
        className={
          viewType === "topology"
            ? "space-y-4"
            : "grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(280px,320px)] lg:items-start"
        }
      >
        <Panel
          title={
            viewType === "topology"
              ? "Hexagonal Topology Map"
              : mode === "baseline"
                ? "Baseline by family"
                : "Catalog by family"
          }
          subtitle={
            viewType === "topology"
              ? "Interactive geometric territorial map of 20 NIST families and 18 architectural corridors"
              : `${visible.length} of ${modeSet.length} controls shown`
          }
          provenance={prov}
        >
          <div className="space-y-3">
            <Toolbar>
              <label className="sr-only" htmlFor="atlas-search">
                Search controls by id or title
              </label>
              <input
                id="atlas-search"
                type="search"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search id or title (ac-2, AC-2(1), audit)"
                className={`${INPUT} w-full sm:w-64`}
              />
              <Segmented
                label="View mode"
                value={viewType}
                onChange={(v) => setViewType(v as "topology" | "grid")}
                options={[
                  { value: "topology", label: "Hexagonal Map" },
                  { value: "grid", label: "Family Grid" },
                ]}
              />
              {viewType === "grid" && (
                <>
                  <label className="sr-only" htmlFor="atlas-family">
                    Family
                  </label>
                  <select
                    id="atlas-family"
                    value={family}
                    onChange={(e) => setFamily(e.target.value)}
                    className={`${INPUT} max-w-full`}
                  >
                    <option value="all">All families</option>
                    {families.map((f) => (
                      <option key={f.id} value={f.id}>
                        {f.id.toUpperCase()} {f.title}
                      </option>
                    ))}
                  </select>
                  <Segmented
                    label="Control set"
                    value={mode}
                    onChange={(m) => {
                      setMode(m);
                      if (
                        m === "baseline" &&
                        family !== "all" &&
                        !index.atlas.families.find(
                          (f) => f.id === family && f.in_baseline > 0,
                        )
                      ) {
                        setFamily("all");
                      }
                    }}
                    options={[
                      {
                        value: "baseline",
                        label: `Baseline ${counts.baselineControls}`,
                      },
                      {
                        value: "catalog",
                        label: `Full catalog ${counts.catalogControls}`,
                      },
                    ]}
                  />
                </>
              )}
            </Toolbar>

            {viewType === "topology" ? (
              <div className="space-y-3">
                <p
                  aria-live="polite"
                  className="min-h-[2.75rem] rounded-md border border-ck-hairline bg-ck-bg-0 px-3 py-1.5 text-sm text-ck-fg-2"
                >
                  {focusCtl ? (
                    <>
                      <span className="font-mono text-xs font-semibold text-ck-fg-1">
                        {displayId(focusCtl.id)}
                      </span>{" "}
                      <span className="text-ck-fg-1">{focusCtl.title}</span>
                      <span className="block text-xs text-ck-fg-mute">
                        {hoverId ? "Pointer/focus" : "Selected"}:{" "}
                        {index.familyById.get(focusCtl.family)?.title}
                        {focusCtl.withdrawn
                          ? ", withdrawn"
                          : focusCtl.in_baseline
                            ? `, in baseline, ${implLabel(implState(focusCtl.id, ssp.byControl)).toLowerCase()}`
                            : ", not in Moderate baseline"}
                      </span>
                    </>
                  ) : (
                    <span className="text-ck-fg-mute">
                      Click or hover a control node to select it; drag to pan; use + / - to zoom; click Replay Pulse to animate corridor flow.
                    </span>
                  )}
                </p>

                <TopologyMap
                  index={index}
                  byControl={ssp.byControl}
                  selectedId={selectedId}
                  onSelectControl={select}
                  hoverId={hoverId}
                  onHoverControl={setHoverId}
                  query={query}
                  pulseActive={pulseActive}
                  onTriggerPulse={triggerPulse}
                  lens={lens}
                  onOpenInComposer={onOpenInComposer}
                />
              </div>
            ) : (
              <div className="space-y-3">
                <Legend items={legend} />

                <p
                  aria-live="polite"
                  className="min-h-[2.75rem] rounded-md border border-ck-hairline bg-ck-bg-0 px-3 py-1.5 text-sm text-ck-fg-2"
                >
                  {focusCtl ? (
                    <>
                      <span className="font-mono text-xs font-semibold text-ck-fg-1">
                        {displayId(focusCtl.id)}
                      </span>{" "}
                      <span className="text-ck-fg-1">{focusCtl.title}</span>
                      <span className="block text-xs text-ck-fg-mute">
                        {hoverId ? "Pointer/focus" : "Selected"}:{" "}
                        {index.familyById.get(focusCtl.family)?.title}
                        {focusCtl.withdrawn
                          ? ", withdrawn"
                          : focusCtl.in_baseline
                            ? `, in baseline, ${implLabel(implState(focusCtl.id, ssp.byControl)).toLowerCase()}`
                            : ", not in Moderate baseline"}
                      </span>
                    </>
                  ) : (
                    <span className="text-ck-fg-mute">
                      Hover or focus a cell to see its id and title.
                    </span>
                  )}
                </p>

                {visible.length === 0 ? (
                  <EmptyState kind="empty" title="No controls match">
                    Nothing in the {mode === "baseline" ? "baseline" : "catalog"}{" "}
                    matches
                    {query ? ` "${query}"` : ""}
                    {family !== "all" ? ` in ${family.toUpperCase()}` : ""}.
                  </EmptyState>
                ) : (
                  <FamilyGrid
                    index={index}
                    visible={visible}
                    byControl={ssp.byControl}
                    mode={mode}
                    selectedId={selectedId}
                    onSelect={select}
                    onHover={setHoverId}
                  />
                )}
                <p className="text-xs text-ck-fg-mute">
                  Arrow keys move between cells; Enter selects. Enhancements are
                  grouped with their parent control.
                </p>
              </div>
            )}
          </div>
        </Panel>

        <div
          ref={detailRef}
          className={
            viewType === "topology"
              ? "min-w-0 scroll-mt-4"
              : "min-w-0 scroll-mt-4 lg:sticky lg:top-4"
          }
        >
          <Panel
            title="Selected control"
            provenance={prov}
            className="lg:max-h-[calc(100vh-6rem)] lg:overflow-y-auto"
          >
            <ControlDetail
              index={index}
              id={selectedId}
              byControl={ssp.byControl}
              sspProvenance={ssp.provenance}
              sspError={ssp.error}
              onSelect={onSelectControl}
              onOpenInComposer={onOpenInComposer}
              lens={lens}
            />
          </Panel>
        </div>
      </div>
    </>
  );
}

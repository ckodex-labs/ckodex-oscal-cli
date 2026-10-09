"use client";

import * as React from "react";
import { cn } from "@/lib/utils";
import type { BridgeEdge, MappingRow } from "@/lib/atlas-data";
import {
  DataTable,
  DesktopOnly,
  Legend,
  PageHeader,
  Panel,
  ReadOnlyNotice,
  Segmented,
  StatTile,
  StateBadge,
} from "@/components/kit";
import {
  BridgeDiagram,
  LineSample,
  RELATIONSHIPS,
  RelationshipGlyph,
  confidenceBand,
  relLabel,
} from "./fixture/bridge-diagram";
import {
  ActionButton,
  BRIDGE_PROVENANCE,
  DetailList,
  StatRow,
  pct,
  useRevealOnChange,
} from "./fixture/shared";

interface BridgeSurfaceProps {
  mer: MappingRow[];
  iso: MappingRow[];
  csf: MappingRow[];
  edgesIso: BridgeEdge[];
  edgesCsf: BridgeEdge[];
  onEdgeConfirmHuman?: (edgeId: string) => void;
}

type Target = "iso" | "csf";
type View = "diagram" | "table";

const TARGET_LABEL: Record<Target, string> = {
  iso: "ISO/IEC 27001:2022 Annex A",
  csf: "NIST CSF 2.0",
};

const SOURCE_LABEL = "Internal practices (sample)";

export function BridgeSurface({
  mer,
  iso,
  csf,
  edgesIso,
  edgesCsf,
  onEdgeConfirmHuman,
}: BridgeSurfaceProps) {
  const [target, setTarget] = React.useState<Target>("iso");
  const [view, setView] = React.useState<View>("diagram");
  const targetRows = target === "iso" ? iso : csf;
  const edges = target === "iso" ? edgesIso : edgesCsf;
  const [selectedId, setSelectedId] = React.useState<string | null>(edges[0]?.id ?? null);

  const changeTarget = (t: Target) => {
    setTarget(t);
    const next = t === "iso" ? edgesIso : edgesCsf;
    setSelectedId(next[0]?.id ?? null);
  };

  const selected = edges.find((e) => e.id === selectedId) ?? null;
  const detailRef = useRevealOnChange<HTMLDivElement>(selectedId);

  const stats = React.useMemo(() => {
    const mappedTargets = new Set(edges.map((e) => e.r));
    const mappedSources = new Set(edges.map((e) => e.l));
    return {
      total: edges.length,
      complete: edges.filter((e) => e.st === "complete").length,
      draft: edges.filter((e) => e.st === "draft").length,
      unmappedTargets: targetRows.length - mappedTargets.size,
      unmappedSources: mer.length - mappedSources.size,
    };
  }, [edges, targetRows, mer]);

  const relCounts = React.useMemo(() => {
    const m = new Map<string, number>();
    edges.forEach((e) => m.set(e.rel, (m.get(e.rel) ?? 0) + 1));
    return m;
  }, [edges]);

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="Bridge · framework mapping"
        title="The Bridge"
        description="Illustrative data only. This surface shows how a crosswalk between internal practices and an external framework will be reviewed. The rows, relationships and confidences below are hand-written samples, not engine output."
        actions={
          <Segmented<Target>
            label="Target framework"
            value={target}
            onChange={changeTarget}
            options={[
              { value: "iso", label: <span className="whitespace-nowrap">ISO 27001</span> },
              { value: "csf", label: <span className="whitespace-nowrap">CSF 2.0</span> },
            ]}
          />
        }
      />

      <ReadOnlyNotice />

      <Panel title="Summary" subtitle={`${SOURCE_LABEL} to ${TARGET_LABEL[target]}`} provenance={BRIDGE_PROVENANCE}>
        <StatRow>
          <StatTile label="Mappings" value={stats.total} />
          <StatTile label="Complete" value={stats.complete} />
          <StatTile label="Draft" value={stats.draft} tone={stats.draft > 0 ? "info" : "neutral"} hint="awaiting review" />
          <StatTile
            label="Targets with no mapping"
            value={`${stats.unmappedTargets} of ${targetRows.length}`}
            tone={stats.unmappedTargets > 0 ? "unk" : "neutral"}
            hint="no edge recorded; not a failure"
          />
        </StatRow>
      </Panel>

      <div className="grid min-w-0 gap-5 xl:grid-cols-[minmax(0,1fr)_20rem]">
        <Panel
          title="Mappings"
          subtitle="Select a relationship marker to inspect it."
          provenance={BRIDGE_PROVENANCE}
          actions={
            <div className="hidden md:block">
              <Segmented<View>
                label="View"
                value={view}
                onChange={setView}
                options={[
                  { value: "diagram", label: "Diagram" },
                  { value: "table", label: "Table" },
                ]}
              />
            </div>
          }
        >
          <div className="space-y-4">
            <BridgeLegend relCounts={relCounts} />

            {/* Tablet and desktop */}
            <div className="hidden md:block">
              {view === "diagram" ? (
                <BridgeDiagram
                  source={mer}
                  target={targetRows}
                  edges={edges}
                  sourceLabel={SOURCE_LABEL}
                  targetLabel={TARGET_LABEL[target]}
                  selectedId={selectedId}
                  onSelect={setSelectedId}
                />
              ) : (
                <EdgeTable
                  edges={edges}
                  source={mer}
                  target={targetRows}
                  selectedId={selectedId}
                  onSelect={setSelectedId}
                />
              )}
            </div>

            {/* Phone: list of mappings */}
            <ul className="space-y-2 md:hidden">
              {edges.map((e) => (
                <li key={e.id}>
                  <button
                    type="button"
                    onClick={() => setSelectedId(e.id)}
                    aria-pressed={e.id === selectedId}
                    className={cn(
                      "w-full rounded-md border px-3 py-2.5 text-left",
                      e.id === selectedId ? "border-ck-accent bg-ck-bg-2" : "border-ck-hairline bg-ck-bg-0",
                    )}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="truncate font-mono text-xs font-medium text-ck-fg-1">
                        {mer[e.l]?.id}
                      </span>
                      <span className="font-mono text-2xs text-ck-fg-mute ck-num">{pct(e.conf)}</span>
                    </div>
                    <div className="mt-1 flex items-center gap-1.5 text-xs text-ck-fg-3">
                      <RelationshipGlyph rel={e.rel} size={14} className="text-ck-fg-2" />
                      <span>{relLabel(e.rel)}</span>
                      <span className="truncate font-mono text-ck-fg-1">{targetRows[e.r]?.id}</span>
                    </div>
                    <div className="mt-1.5">
                      <StatusBadge edge={e} />
                    </div>
                  </button>
                </li>
              ))}
            </ul>
          </div>
        </Panel>

        <div ref={detailRef} className="min-w-0 scroll-mt-4 xl:sticky xl:top-4 xl:self-start">
          <EdgeDetail
            edge={selected}
            source={mer}
            target={targetRows}
            targetLabel={TARGET_LABEL[target]}
            canConfirm={target === "iso" && !!onEdgeConfirmHuman}
            onConfirm={(id) => onEdgeConfirmHuman?.(id)}
          />
        </div>
      </div>
    </div>
  );
}

function StatusBadge({ edge }: { edge: BridgeEdge }) {
  return edge.st === "draft" ? (
    <StateBadge tone="info">draft</StateBadge>
  ) : (
    <StateBadge tone="neutral">complete</StateBadge>
  );
}

function BridgeLegend({ relCounts }: { relCounts: Map<string, number> }) {
  return (
    <div className="space-y-2 rounded-md border border-ck-hairline bg-ck-bg-0 px-3 py-2.5">
      <div className="flex flex-wrap items-start gap-x-3 gap-y-1.5">
        <span className="ck-eyebrow w-24 shrink-0">relationship</span>
        <Legend
          className="min-w-0 flex-1"
          items={RELATIONSHIPS.map((r) => ({
            swatch: <RelationshipGlyph rel={r.rel} size={16} className="text-ck-fg-2" />,
            label: r.label,
            count: relCounts.get(r.rel) ?? 0,
          }))}
        />
      </div>
      <div className="flex flex-wrap items-start gap-x-3 gap-y-1.5">
        <span className="ck-eyebrow w-24 shrink-0">confidence</span>
        <Legend
          className="min-w-0 flex-1"
          items={[
            { swatch: <LineSample conf={0.9} />, label: "high, 80% and above" },
            { swatch: <LineSample conf={0.7} />, label: "medium, 60 to 79%" },
            { swatch: <LineSample conf={0.4} />, label: "low, below 60%" },
          ]}
        />
      </div>
      <div className="flex flex-wrap items-start gap-x-3 gap-y-1.5">
        <span className="ck-eyebrow w-24 shrink-0">status</span>
        <Legend
          className="min-w-0 flex-1"
          items={[
            { swatch: <LineSample conf={0.9} />, label: "complete" },
            { swatch: <LineSample conf={0.9} st="draft" />, label: "draft" },
            { swatch: <LineSample conf={0.9} der />, label: "inferred via another framework" },
          ]}
        />
      </div>
    </div>
  );
}

function EdgeTable({
  edges,
  source,
  target,
  selectedId,
  onSelect,
}: {
  edges: BridgeEdge[];
  source: MappingRow[];
  target: MappingRow[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}) {
  return (
    <DataTable
      caption="Mapping edges (fixture)"
      columns={[
        { key: "src", label: "Source" },
        { key: "rel", label: "Relationship" },
        { key: "tgt", label: "Target" },
        { key: "conf", label: "Confidence", numeric: true },
        { key: "cov", label: "Coverage", numeric: true },
        { key: "st", label: "Status" },
      ]}
      rows={edges.map((e) => ({
        src: (
          <button
            type="button"
            onClick={() => onSelect(e.id)}
            aria-pressed={e.id === selectedId}
            className={cn(
              "font-mono text-xs underline-offset-2 hover:underline focus:outline-none focus-visible:ring-2 focus-visible:ring-ck-accent",
              e.id === selectedId ? "font-semibold text-ck-accent-text" : "text-ck-fg-1",
            )}
          >
            {source[e.l]?.id}
          </button>
        ),
        rel: (
          <span className="inline-flex items-center gap-1.5 whitespace-nowrap">
            <RelationshipGlyph rel={e.rel} size={14} className="text-ck-fg-2" />
            {relLabel(e.rel)}
          </span>
        ),
        tgt: <span className="font-mono text-xs text-ck-fg-1">{target[e.r]?.id}</span>,
        conf: pct(e.conf),
        cov: pct(e.cov),
        st: <StatusBadge edge={e} />,
      }))}
    />
  );
}

function EdgeDetail({
  edge,
  source,
  target,
  targetLabel,
  canConfirm,
  onConfirm,
}: {
  edge: BridgeEdge | null;
  source: MappingRow[];
  target: MappingRow[];
  targetLabel: string;
  canConfirm: boolean;
  onConfirm: (id: string) => void;
}) {
  if (!edge) {
    return (
      <Panel title="Selected mapping" provenance={BRIDGE_PROVENANCE}>
        <p className="text-sm text-ck-fg-3">Select a mapping to see its details.</p>
      </Panel>
    );
  }
  const s = source[edge.l];
  const t = target[edge.r];
  const needsReview = edge.st === "draft" || edge.m !== "human";
  const band = confidenceBand(edge.conf);

  return (
    <Panel
      title="Selected mapping"
      subtitle={`Edge ${edge.id}`}
      provenance={BRIDGE_PROVENANCE}
    >
      <div className="space-y-4">
        <div className="grid gap-4 md:grid-cols-2 md:items-start xl:grid-cols-1">
        <div className="space-y-2 rounded-md border border-ck-hairline bg-ck-bg-0 p-3">
          <div className="min-w-0">
            <p className="font-mono text-xs font-medium text-ck-fg-1">{s?.id}</p>
            <p className="text-sm text-ck-fg-2">{s?.t}</p>
          </div>
          <div className="flex items-center gap-2 text-xs text-ck-fg-3">
            <RelationshipGlyph rel={edge.rel} size={18} className="text-ck-fg-1" />
            <span className="font-medium text-ck-fg-1">{relLabel(edge.rel)}</span>
          </div>
          <div className="min-w-0">
            <p className="font-mono text-xs font-medium text-ck-fg-1">{t?.id}</p>
            <p className="text-sm text-ck-fg-2">{t?.t}</p>
            <p className="text-xs text-ck-fg-mute">{targetLabel}</p>
          </div>
        </div>

        <DetailList
          items={[
            { label: "Status", value: <StatusBadge edge={edge} /> },
            {
              label: "Confidence",
              value: (
                <span>
                  <span className="font-mono ck-num">{pct(edge.conf)}</span>{" "}
                  <span className="text-ck-fg-mute">({band})</span>
                </span>
              ),
            },
            { label: "Coverage of target", value: <span className="font-mono ck-num">{pct(edge.cov)}</span> },
            { label: "Rationale", value: edge.rat },
            { label: "Method", value: edge.m },
            { label: "Proposed by", value: edge.by },
            ...(edge.qual ? [{ label: "Qualifier", value: edge.qual }] : []),
            ...(edge.der
              ? [{ label: "Origin", value: "Inferred through another framework, not asserted directly." }]
              : []),
          ]}
        />
        </div>

        <DesktopOnly>
          <div className="space-y-2 border-t border-ck-hairline pt-3">
            {canConfirm ? (
              needsReview ? (
                <>
                  <ActionButton variant="primary" onClick={() => onConfirm(edge.id)}>
                    Confirm mapping in this session
                  </ActionButton>
                  <p className="text-xs text-ck-fg-mute">
                    Records a LOCAL receipt in this browser session. It is not signed, not persisted,
                    and does not change any OSCAL document.
                  </p>
                </>
              ) : (
                <p className="text-xs text-ck-fg-mute">
                  This sample edge is already marked complete by a human reviewer role. Nothing to confirm.
                </p>
              )
            ) : (
              <p className="text-xs text-ck-fg-mute">
                Session confirmation is available for the ISO 27001 sample only.
              </p>
            )}
          </div>
        </DesktopOnly>
      </div>
    </Panel>
  );
}

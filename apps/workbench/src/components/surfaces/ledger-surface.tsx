"use client";

import * as React from "react";
import { cn } from "@/lib/utils";
import type { EvidenceItem } from "@/lib/atlas-data";
import type { LensMode } from "@/lib/oscal-types";
import {
  DataTable,
  EmptyState,
  PageHeader,
  Panel,
  ReadOnlyNotice,
  Segmented,
  StatTile,
  StateBadge,
  Toolbar,
  type Tone,
} from "@/components/kit";
import {
  DetailList,
  FIXTURE_DATE_NOTE,
  FilterField,
  LEDGER_PROVENANCE,
  SearchInput,
  StatRow,
  useRevealOnChange,
} from "./fixture/shared";

type Freshness = "current" | "aging" | "stale";
type MethodFilter = "all" | EvidenceItem["m"];
type FreshFilter = "all" | Freshness;

function freshness(days: number): Freshness {
  if (days <= 30) return "current";
  if (days <= 180) return "aging";
  return "stale";
}

const FRESH_TONE: Record<Freshness, Tone> = {
  current: "neutral",
  aging: "warn",
  stale: "neg",
};

const FRESH_HINT: Record<Freshness, string> = {
  current: "collected within 30 days",
  aging: "31 to 180 days old",
  stale: "older than 180 days",
};

function FreshBadge({ days }: { days: number }) {
  const f = freshness(days);
  return <StateBadge tone={FRESH_TONE[f]}>{f}</StateBadge>;
}

export interface LedgerSurfaceProps {
  evidence: EvidenceItem[];
  lens?: LensMode;
}

export function LedgerSurface({
  evidence,
  lens = "assessor",
}: LedgerSurfaceProps) {
  const [method, setMethod] = React.useState<MethodFilter>("all");
  const [fresh, setFresh] = React.useState<FreshFilter>("all");
  const [query, setQuery] = React.useState("");
  const [selectedId, setSelectedId] = React.useState<string | null>(evidence[0]?.id ?? null);

  const LEDGER_LENS_INFO: Record<
    LensMode,
    { title: string; desc: string; tone: "info" | "pos" | "warn" | "neutral" }
  > = {
    assessor: {
      title: "Assessor Lens Active",
      desc: "Auditing evidence freshness, collection methodology (automated vs manual), and cryptographically bound attachments.",
      tone: "neutral",
    },
    engineer: {
      title: "Engineer Lens Active",
      desc: "Checking automated collector integrations, webhook event receipts, and infrastructure evidence artifacts.",
      tone: "pos",
    },
    author: {
      title: "Author Lens Active",
      desc: "Associating evidentiary artifacts with control implementation narratives and component descriptions.",
      tone: "neutral",
    },
    "risk-owner": {
      title: "Risk Owner Lens Active",
      desc: "Identifying stale evidence, unproven control claims, and impending audit expirations.",
      tone: "warn",
    },
    ciso: {
      title: "CISO Lens Active",
      desc: "Enterprise evidence posture, automated collection coverage ratio, and audit readiness health.",
      tone: "info",
    },
    architect: {
      title: "Architect Lens Active",
      desc: "Analyzing evidence collector coverage topology across boundary components and control families.",
      tone: "info",
    },
  };

  const filtered = React.useMemo(() => {
    const q = query.trim().toLowerCase();
    return evidence.filter(
      (e) =>
        (method === "all" || e.m === method) &&
        (fresh === "all" || freshness(e.days) === fresh) &&
        (!q ||
          e.t.toLowerCase().includes(q) ||
          e.by.toLowerCase().includes(q) ||
          e.ids.some((id) => id.toLowerCase().includes(q))),
    );
  }, [evidence, method, fresh, query]);

  const counts = React.useMemo(() => {
    const c = { current: 0, aging: 0, stale: 0 };
    evidence.forEach((e) => c[freshness(e.days)]++);
    return c;
  }, [evidence]);

  const selected = evidence.find((e) => e.id === selectedId) ?? null;
  const detailRef = useRevealOnChange<HTMLDivElement>(selectedId);

  const titleButton = (e: EvidenceItem) => (
    <button
      type="button"
      onClick={() => setSelectedId(e.id)}
      aria-pressed={e.id === selectedId}
      className={cn(
        "text-left text-sm underline-offset-2 hover:underline focus:outline-none focus-visible:ring-2 focus-visible:ring-ck-accent",
        e.id === selectedId ? "font-semibold text-ck-accent-text" : "font-medium text-ck-fg-1",
      )}
    >
      {e.t}
    </button>
  );

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="Ledger · evidence"
        title="The Ledger"
        description="Illustrative data only. This surface shows how collected evidence will be browsed by control, collector and age. The items below are hand-written samples with no stored object behind them."
      />

      <ReadOnlyNotice />

      {/* Active Lens Status Banner */}
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-ck-hairline-strong bg-ck-bg-1 px-3 py-2 text-xs">
        <div className="flex items-center gap-2 min-w-0">
          <StateBadge tone={LEDGER_LENS_INFO[lens].tone}>
            {LEDGER_LENS_INFO[lens].title}
          </StateBadge>
          <span className="text-ck-fg-2 truncate font-medium">
            {LEDGER_LENS_INFO[lens].desc}
          </span>
        </div>
        <div className="flex items-center gap-1.5 shrink-0 text-2xs text-ck-fg-mute font-mono">
          <span>Lens: {lens} (L key cycles)</span>
        </div>
      </div>

      <Panel title="Summary" subtitle={FIXTURE_DATE_NOTE} provenance={LEDGER_PROVENANCE}>
        <StatRow>
          <StatTile label="Evidence items" value={evidence.length} />
          <StatTile label="Current" value={counts.current} hint={FRESH_HINT.current} />
          <StatTile label="Aging" value={counts.aging} tone={counts.aging ? "warn" : "neutral"} hint={FRESH_HINT.aging} />
          <StatTile label="Stale" value={counts.stale} tone={counts.stale ? "neg" : "neutral"} hint={FRESH_HINT.stale} />
        </StatRow>
      </Panel>

      <div className="grid min-w-0 gap-5 2xl:grid-cols-[minmax(0,1fr)_22rem]">
        <Panel
          title="Evidence"
          subtitle={`${filtered.length} of ${evidence.length} items`}
          provenance={LEDGER_PROVENANCE}
        >
          <div className="space-y-4">
            <Toolbar className="gap-x-4 gap-y-2">
              <SearchInput
                label="Filter evidence"
                placeholder="Filter by title, collector or control"
                value={query}
                onChange={setQuery}
              />
              <FilterField label="Method">
                <Segmented<MethodFilter>
                  label="Collection method"
                  value={method}
                  onChange={setMethod}
                  options={[
                    { value: "all", label: "All" },
                    { value: "automation", label: "Automated" },
                    { value: "hybrid", label: "Hybrid" },
                    { value: "human", label: "Manual" },
                  ]}
                />
              </FilterField>
              <FilterField label="Age">
                <Segmented<FreshFilter>
                  label="Evidence age"
                  value={fresh}
                  onChange={setFresh}
                  options={[
                    { value: "all", label: "All" },
                    { value: "current", label: "Current" },
                    { value: "aging", label: "Aging" },
                    { value: "stale", label: "Stale" },
                  ]}
                />
              </FilterField>
            </Toolbar>

            {filtered.length === 0 ? (
              <EmptyState kind="empty" title="No evidence matches these filters">
                Clear the filter text or choose a different method or age.
              </EmptyState>
            ) : (
              <>
                <div className="hidden md:block">
                  <DataTable
                    caption="Evidence items (fixture)"
                    columns={[
                      { key: "t", label: "Evidence", className: "min-w-[14rem]" },
                      { key: "ids", label: "Controls" },
                      { key: "m", label: "Method" },
                      { key: "age", label: "Age", className: "whitespace-nowrap" },
                    ]}
                    rows={filtered.map((e) => ({
                      t: (
                        <span className="flex flex-col gap-0.5">
                          {titleButton(e)}
                          <span className="text-xs text-ck-fg-mute">{e.by}</span>
                        </span>
                      ),
                      ids: <ControlIds ids={e.ids} />,
                      m: <span className="text-xs">{e.m === "human" ? "manual" : e.m}</span>,
                      age: (
                        <span className="flex flex-col items-start gap-1">
                          <span className="text-xs">{e.age}</span>
                          <FreshBadge days={e.days} />
                        </span>
                      ),
                    }))}
                  />
                </div>
                <ul className="space-y-2 md:hidden">
                  {filtered.map((e) => (
                    <li
                      key={e.id}
                      className={cn(
                        "rounded-md border px-3 py-2.5",
                        e.id === selectedId ? "border-ck-accent bg-ck-bg-2" : "border-ck-hairline bg-ck-bg-0",
                      )}
                    >
                      {titleButton(e)}
                      <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-ck-fg-3">
                        <span>{e.age}</span>
                        <FreshBadge days={e.days} />
                        <ControlIds ids={e.ids} />
                      </div>
                    </li>
                  ))}
                </ul>
              </>
            )}
          </div>
        </Panel>

        <div ref={detailRef} className="min-w-0 scroll-mt-4 2xl:sticky 2xl:top-4 2xl:self-start">
        <Panel
          title="Evidence detail"
          subtitle={selected ? selected.id : undefined}
          provenance={LEDGER_PROVENANCE}
        >
          {selected ? (
            <div className="space-y-4">
              <p className="text-sm font-medium text-ck-fg-1">{selected.t}</p>
              <DetailList
                items={[
                  { label: "Kind", value: selected.kind },
                  { label: "Collected by", value: selected.by },
                  { label: "Method", value: selected.m === "human" ? "manual" : selected.m },
                  { label: "Age", value: selected.age },
                  { label: "Freshness", value: <FreshBadge days={selected.days} /> },
                  { label: "Controls", value: <ControlIds ids={selected.ids} /> },
                  {
                    label: "Content address",
                    value: <StateBadge tone="unk">empty</StateBadge>,
                  },
                  {
                    label: "Integrity",
                    value: <StateBadge tone="unk">unknown</StateBadge>,
                  },
                ]}
              />
              <p className="text-xs text-ck-fg-mute">
                This sample has no stored object, so there is nothing to hash or check. Real evidence
                would get a content address from <code className="font-mono">mizan cas put</code> and
                could then be checked with <code className="font-mono">mizan evidence verify</code>.
              </p>
            </div>
          ) : (
            <p className="text-sm text-ck-fg-3">Select an evidence item to see its details.</p>
          )}
        </Panel>
        </div>
      </div>
    </div>
  );
}

function ControlIds({ ids }: { ids: string[] }) {
  return (
    <span className="inline-flex flex-wrap gap-1">
      {ids.map((id) => (
        <span key={id} className="ck-hash">
          {id}
        </span>
      ))}
    </span>
  );
}

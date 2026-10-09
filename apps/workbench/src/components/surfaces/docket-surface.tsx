"use client";

import * as React from "react";
import { cn } from "@/lib/utils";
import { fixtureDateFromOffset, type PoamItem, type PoamStatus } from "@/lib/atlas-data";
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
import type { LensMode } from "@/lib/oscal-types";
import {
  ActionButton,
  DOCKET_PROVENANCE,
  FIXTURE_DATE_NOTE,
  FilterField,
  StatRow,
} from "./fixture/shared";

type SortKey = "due" | "age" | "risk" | "id";
type StatusFilter = "all" | PoamStatus;

const RISK_RANK: Record<PoamItem["risk"], number> = { high: 3, moderate: 2, low: 1 };
const RISK_TONE: Record<PoamItem["risk"], Tone> = { high: "neg", moderate: "warn", low: "neutral" };

const STATUS_LABEL: Record<PoamStatus, string> = {
  open: "open",
  "in-progress": "in progress",
  "pending-review": "pending review",
};
const STATUS_TONE: Record<PoamStatus, Tone> = {
  open: "neutral",
  "in-progress": "info",
  "pending-review": "info",
};

function dueText(n: number) {
  if (n < 0) return `Overdue by ${-n} ${-n === 1 ? "day" : "days"}`;
  if (n === 0) return "Due on reference date";
  return `Due in ${n} ${n === 1 ? "day" : "days"}`;
}

function DueCell({ n }: { n: number }) {
  return (
    <span className="flex flex-col">
      <span className={cn("text-sm", n < 0 ? "font-medium text-ck-neg" : n <= 14 ? "text-ck-warn" : "text-ck-fg-2")}>
        {dueText(n)}
      </span>
      <span className="font-mono text-2xs text-ck-fg-mute ck-num">{fixtureDateFromOffset(n)}</span>
    </span>
  );
}

export interface DocketSurfaceProps {
  poams: PoamItem[];
  onSelectPoamControl: (id: string) => void;
  lens?: LensMode;
}

export function DocketSurface({
  poams,
  onSelectPoamControl,
  lens = "risk-owner",
}: DocketSurfaceProps) {
  const [sort, setSort] = React.useState<SortKey>("due");
  const [desc, setDesc] = React.useState(false);
  const [status, setStatus] = React.useState<StatusFilter>("all");

  const rows = React.useMemo(() => {
    const val = (p: PoamItem): number | string =>
      sort === "due" ? p.dueN : sort === "age" ? p.age : sort === "risk" ? RISK_RANK[p.risk] : p.id;
    const list = poams.filter((p) => status === "all" || p.status === status);
    return [...list].sort((a, b) => {
      const va = val(a);
      const vb = val(b);
      const c = typeof va === "number" && typeof vb === "number" ? va - vb : String(va).localeCompare(String(vb));
      return desc ? -c : c;
    });
  }, [poams, sort, desc, status]);

  const stats = React.useMemo(() => {
    const controls = new Set(poams.flatMap((p) => p.ids));
    return {
      total: poams.length,
      overdue: poams.filter((p) => p.dueN < 0).length,
      due30: poams.filter((p) => p.dueN >= 0 && p.dueN <= 30).length,
      controls: controls.size,
    };
  }, [poams]);

  const controlButtons = (ids: string[]) => (
    <span className="inline-flex flex-wrap gap-1">
      {ids.map((id) => (
        <button
          key={id}
          type="button"
          onClick={() => onSelectPoamControl(id)}
          title={`Open ${id} in the Atlas`}
          className="rounded-sm border border-ck-hairline-strong bg-ck-bg-0 px-1.5 py-px font-mono text-2xs font-medium text-ck-accent-text hover:bg-ck-bg-2 focus:outline-none focus-visible:ring-2 focus-visible:ring-ck-accent"
        >
          {id}
        </button>
      ))}
    </span>
  );

  const sortLabel: Record<SortKey, string> = { due: "Due date", age: "Age open", risk: "Risk", id: "ID" };

  const DOCKET_LENS_INFO: Record<
    LensMode,
    { title: string; desc: string; tone: "info" | "pos" | "warn" | "neutral" }
  > = {
    "risk-owner": {
      title: "Risk Owner Lens Active",
      desc: "Managing remediation debt, scheduled milestone progress, and overdue vulnerability risks.",
      tone: "warn",
    },
    ciso: {
      title: "CISO Lens Active",
      desc: "Executive risk exposure burndown, liability milestones, and critical compliance exceptions.",
      tone: "info",
    },
    assessor: {
      title: "Assessor Lens Active",
      desc: "Auditing POA&M closure evidence, milestone verification artifacts, and acceptance justifications.",
      tone: "neutral",
    },
    engineer: {
      title: "Engineer Lens Active",
      desc: "Tracking technical patch remediation, configuration fixes, and automated PR remediations.",
      tone: "pos",
    },
    author: {
      title: "Author Lens Active",
      desc: "Correlating open POA&M deficiency descriptions with control implementation statements.",
      tone: "neutral",
    },
    architect: {
      title: "Architect Lens Active",
      desc: "Evaluating structural dependencies and blast radius of components carrying open POA&M items.",
      tone: "info",
    },
  };

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="Docket · plan of action and milestones"
        title="The Docket"
        description="Illustrative data only. This surface shows how open weaknesses will be tracked against controls, owners and due dates. The items below are hand-written samples; control ids are real NIST SP 800-53 ids and open in the Atlas."
      />

      <ReadOnlyNotice />

      {/* Active Lens Status Banner */}
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-ck-hairline-strong bg-ck-bg-1 px-3 py-2 text-xs">
        <div className="flex items-center gap-2 min-w-0">
          <StateBadge tone={DOCKET_LENS_INFO[lens].tone}>
            {DOCKET_LENS_INFO[lens].title}
          </StateBadge>
          <span className="text-ck-fg-2 truncate font-medium">
            {DOCKET_LENS_INFO[lens].desc}
          </span>
        </div>
        <div className="flex items-center gap-1.5 shrink-0 text-2xs text-ck-fg-mute font-mono">
          <span>Lens: {lens} (L key cycles)</span>
        </div>
      </div>

      <Panel title="Summary" subtitle={FIXTURE_DATE_NOTE} provenance={DOCKET_PROVENANCE}>
        <StatRow>
          <StatTile label="Open items" value={stats.total} />
          <StatTile label="Overdue" value={stats.overdue} tone={stats.overdue ? "neg" : "neutral"} />
          <StatTile label="Due within 30 days" value={stats.due30} tone={stats.due30 ? "warn" : "neutral"} />
          <StatTile label="Controls affected" value={stats.controls} />
        </StatRow>
      </Panel>

      <Panel
        title="POA&M items"
        subtitle={`${rows.length} of ${poams.length} items, sorted by ${sortLabel[sort].toLowerCase()} (${desc ? "descending" : "ascending"})`}
        provenance={DOCKET_PROVENANCE}
      >
        <div className="space-y-4">
          <Toolbar className="gap-x-4 gap-y-2">
            <FilterField label="Sort">
              <Segmented<SortKey>
                label="Sort by"
                value={sort}
                onChange={setSort}
                options={(Object.keys(sortLabel) as SortKey[]).map((k) => ({ value: k, label: sortLabel[k] }))}
              />
              <ActionButton
                variant="ghost"
                className="h-7 px-2 text-xs"
                onClick={() => setDesc((d) => !d)}
                aria-label={desc ? "Sort ascending" : "Sort descending"}
              >
                {desc ? "Descending" : "Ascending"}
              </ActionButton>
            </FilterField>
            <FilterField label="Status">
              <Segmented<StatusFilter>
                label="Status filter"
                value={status}
                onChange={setStatus}
                options={[
                  { value: "all", label: "All" },
                  { value: "open", label: "Open" },
                  { value: "in-progress", label: "In progress" },
                  { value: "pending-review", label: "Pending review" },
                ]}
              />
            </FilterField>
          </Toolbar>

          {rows.length === 0 ? (
            <EmptyState kind="empty" title="No items with this status" />
          ) : (
            <>
              <div className="hidden md:block">
                <DataTable
                  caption="POA&M items (fixture)"
                  columns={[
                    { key: "id", label: "ID", className: "whitespace-nowrap" },
                    { key: "t", label: "Weakness", className: "min-w-[13rem]" },
                    { key: "ids", label: "Controls" },
                    { key: "own", label: "Owner role" },
                    { key: "due", label: "Due", className: "whitespace-nowrap" },
                    { key: "st", label: "Status" },
                  ]}
                  rows={rows.map((p) => ({
                    id: <span className="font-mono text-xs text-ck-fg-1">{p.id}</span>,
                    t: (
                      <span className="flex flex-col">
                        <span className="text-sm font-medium text-ck-fg-1">{p.t}</span>
                        <span className="text-xs text-ck-fg-mute">
                          open {p.age} days · milestones{" "}
                          <span className="font-mono ck-num">
                            {p.milestones.done}/{p.milestones.total}
                          </span>
                        </span>
                      </span>
                    ),
                    ids: controlButtons(p.ids),
                    own: <span className="text-xs">{p.own}</span>,
                    due: <DueCell n={p.dueN} />,
                    st: (
                      <span className="flex flex-col items-start gap-1">
                        <StateBadge tone={STATUS_TONE[p.status]}>{STATUS_LABEL[p.status]}</StateBadge>
                        <StateBadge tone={RISK_TONE[p.risk]}>{p.risk} risk</StateBadge>
                      </span>
                    ),
                  }))}
                />
              </div>
              <ul className="space-y-2 md:hidden">
                {rows.map((p) => (
                  <li key={p.id} className="rounded-md border border-ck-hairline bg-ck-bg-0 px-3 py-2.5">
                    <div className="flex items-start justify-between gap-2">
                      <span className="font-mono text-xs text-ck-fg-mute">{p.id}</span>
                      <StateBadge tone={RISK_TONE[p.risk]}>{p.risk} risk</StateBadge>
                    </div>
                    <p className="mt-1 text-sm font-medium text-ck-fg-1">{p.t}</p>
                    <p className="mt-0.5 text-xs text-ck-fg-3">{p.own}</p>
                    <div className="mt-2 flex flex-wrap items-center justify-between gap-2">
                      <DueCell n={p.dueN} />
                      <StateBadge tone={STATUS_TONE[p.status]}>{STATUS_LABEL[p.status]}</StateBadge>
                    </div>
                    <div className="mt-2">{controlButtons(p.ids)}</div>
                  </li>
                ))}
              </ul>
            </>
          )}
        </div>
      </Panel>
    </div>
  );
}

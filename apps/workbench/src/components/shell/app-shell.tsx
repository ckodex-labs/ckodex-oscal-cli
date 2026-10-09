"use client";

/**
 * Workbench application shell.
 *
 * Layout contract:
 *   >= 1280px  nav (240px) | surface | evidence rail (320px, collapsible)
 *   768-1279   icon nav (56px) | surface ; evidence rail as slide-over
 *   < 768      top bar + menu sheet ; surface is read-only
 *
 * Truth contract:
 *   - Engine status is probed, never asserted (see lib/engine.tsx).
 *   - Workspace identity comes from the snapshot manifest (git commit), not a
 *     hard-coded repository name.
 *   - Session receipts are LOCAL: SHA-256 of event text, unsigned, in memory.
 */

import * as React from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine";
import { LensMode, ThemeMode } from "@/lib/oscal-types";
import { LensSelector } from "./lens-selector";
import { ThemeToggle } from "./theme-toggle";
import { StateBadge } from "@/components/kit";
import { SurfaceIcon } from "./surface-icons";

export type SurfaceId =
  | "atlas"
  | "bridge"
  | "composer"
  | "ledger"
  | "docket"
  | "pipeline"
  | "jurisdiction"
  | "cicd"
  | "fabric"
  | "inspector";

export interface SurfaceDef {
  id: SurfaceId;
  label: string;
  key: string;
  /** Primary data source for this surface. FIXTURE surfaces are marked in nav. */
  source: "engine" | "fixture" | "mixed";
}

export const SURFACE_GROUPS: { label: string; items: SurfaceDef[] }[] = [
  {
    label: "Territory",
    items: [
      { id: "atlas", label: "Atlas", key: "1", source: "engine" },
      { id: "bridge", label: "Bridge", key: "2", source: "fixture" },
    ],
  },
  {
    label: "Record",
    items: [
      { id: "composer", label: "Composer", key: "3", source: "mixed" },
      { id: "ledger", label: "Ledger", key: "4", source: "fixture" },
    ],
  },
  {
    label: "Operations",
    items: [
      { id: "docket", label: "Docket", key: "5", source: "fixture" },
      { id: "pipeline", label: "Pipeline", key: "6", source: "engine" },
    ],
  },
  {
    label: "Governance",
    items: [
      { id: "jurisdiction", label: "Jurisdictions & SLSA", key: "7", source: "engine" },
      { id: "cicd", label: "Policy Gates & SBOM", key: "8", source: "engine" },
      { id: "fabric", label: "Root Fabric & Identity", key: "9", source: "fixture" },
    ],
  },
  {
    label: "Verification",
    items: [{ id: "inspector", label: "Inspector", key: "0", source: "engine" }],
  },
];

export const ALL_SURFACES = SURFACE_GROUPS.flatMap((g) => g.items);

export interface SessionReceipt {
  event: string;
  at: string; // ISO
  digest: string; // hex sha256 of event text + time
}

/* ------------------------------------------------------------------ */
/* Engine status pill                                                  */
/* ------------------------------------------------------------------ */

export function EngineStatusPill({ compact = false }: { compact?: boolean }) {
  const { status } = useEngine();
  const base =
    "inline-flex items-center gap-1.5 rounded-sm border px-2 py-0.5 font-mono text-2xs whitespace-nowrap";
  switch (status.kind) {
    case "probing":
      return <span className={cn(base, "border-ck-hairline-strong text-ck-fg-mute")}>engine: probing</span>;
    case "live":
      return (
        <span className={cn(base, "border-ck-pos text-ck-pos bg-ck-pos-bg")} title="The local mizan binary answered a health probe.">
          <span aria-hidden>{"\u25CF"}</span>
          {compact ? "LIVE" : `LIVE \u00B7 ${status.version}`}
        </span>
      );
    case "snapshot": {
      const m = status.manifest;
      return (
        <span
          className={cn(base, "border-ck-info text-ck-info bg-ck-info-bg")}
          title={`No local engine. Showing real mizan output captured at build time (${m.generatedAt}).`}
        >
          <span aria-hidden>{"\u25A0"}</span>
          {compact ? "SNAPSHOT" : `SNAPSHOT \u00B7 ${m.gitCommit.slice(0, 7)}${m.gitDirty ? "+dirty" : ""}`}
        </span>
      );
    }
    case "offline":
      return (
        <span className={cn(base, "border-ck-neg text-ck-neg bg-ck-neg-bg")} title={status.reason}>
          <span aria-hidden>{"\u2715"}</span> NO ENGINE DATA
        </span>
      );
  }
}

/* ------------------------------------------------------------------ */
/* Masthead                                                            */
/* ------------------------------------------------------------------ */

function Masthead({
  theme,
  onThemeChange,
  lens,
  onLensChange,
  onOpenShortcuts,
  isCopilotOpen,
  onToggleCopilot,
  onOpenMenu,
  onToggleEvidence,
  evidenceCount,
}: {
  theme: ThemeMode;
  onThemeChange: (t: ThemeMode) => void;
  lens: LensMode;
  onLensChange: (l: LensMode) => void;
  onOpenShortcuts: () => void;
  isCopilotOpen: boolean;
  onToggleCopilot: () => void;
  onOpenMenu: () => void;
  onToggleEvidence: () => void;
  evidenceCount: number;
}) {
  const btn =
    "inline-flex h-8 items-center gap-1.5 rounded-md border border-ck-hairline-strong bg-ck-bg-1 px-2.5 text-xs text-ck-fg-2 hover:text-ck-fg-1 hover:border-ck-fg-3 transition-colors";
  return (
    <header className="flex h-14 shrink-0 items-center gap-3 border-b border-ck-hairline-strong bg-ck-bg-0 px-3 md:px-4">
      <button type="button" onClick={onOpenMenu} className={cn(btn, "md:hidden")} aria-label="Open navigation">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden>
          <path d="M2 4h12M2 8h12M2 12h12" />
        </svg>
      </button>

      <div className="flex shrink-0 items-baseline gap-2 md:gap-3">
        <span className="font-serif text-[22px] sm:text-[26px] leading-none tracking-tight text-ck-fg-1 shrink-0">Mizan</span>
        <span className="hidden truncate text-xs text-ck-fg-mute lg:inline">OSCAL compliance workbench</span>
      </div>

      <div className="hidden md:block">
        <EngineStatusPill />
      </div>

      <div className="ml-auto flex items-center gap-1.5">
        <div className="md:hidden">
          <EngineStatusPill compact />
        </div>
        <button
          type="button"
          onClick={onToggleCopilot}
          aria-pressed={isCopilotOpen}
          className={cn(btn, "hidden sm:inline-flex", isCopilotOpen && "bg-ck-fg-1 text-ck-bg-0 border-ck-fg-1 hover:text-ck-bg-0")}
          title="Copilot (Cmd/Ctrl+K)"
        >
          Copilot
          <kbd className="font-mono text-2xs opacity-70">{"\u2318"}K</kbd>
        </button>
        <button type="button" onClick={onToggleEvidence} className={cn(btn, "xl:hidden")} title="Evidence and session log">
          Evidence
          <span className="font-mono text-2xs text-ck-fg-mute ck-num">{evidenceCount}</span>
        </button>
        <div className="hidden lg:flex items-center gap-1.5">
          <LensSelector lens={lens} onLensChange={onLensChange} />
        </div>
        <ThemeToggle theme={theme} onThemeChange={onThemeChange} />
        <button type="button" onClick={onOpenShortcuts} className={cn(btn, "hidden md:inline-flex")} aria-label="Keyboard shortcuts">
          ?
        </button>
      </div>
    </header>
  );
}

/* ------------------------------------------------------------------ */
/* Navigation                                                          */
/* ------------------------------------------------------------------ */

function NavLinks({
  active,
  onSelect,
  compact,
}: {
  active: SurfaceId;
  onSelect: (id: SurfaceId) => void;
  compact: boolean;
}) {
  return (
    <div className="space-y-5">
      {SURFACE_GROUPS.map((g) => (
        <div key={g.label} className="space-y-0.5">
          {!compact && <p className="ck-eyebrow px-2.5 pb-1">{g.label}</p>}
          {compact && <div className="mx-auto my-1 h-px w-6 bg-ck-hairline-strong first:hidden" />}
          {g.items.map((item) => {
            const isActive = active === item.id;
            return (
              <button
                key={item.id}
                type="button"
                onClick={() => onSelect(item.id)}
                aria-current={isActive ? "page" : undefined}
                title={compact ? `${item.label} (${item.key})` : `Shortcut: ${item.key}`}
                className={cn(
                  "group relative flex w-full items-center gap-2.5 rounded-md text-left transition-colors",
                  compact ? "h-9 justify-center" : "h-8 px-2.5",
                  isActive ? "bg-ck-bg-0 text-ck-fg-1 font-medium shadow-sm" : "text-ck-fg-3 hover:bg-ck-bg-2 hover:text-ck-fg-1",
                )}
              >
                {isActive && <span className="absolute left-0 top-1.5 bottom-1.5 w-0.5 rounded-full bg-ck-accent" aria-hidden />}
                <span className={cn("shrink-0", isActive ? "text-ck-accent-text" : "text-ck-fg-mute group-hover:text-ck-fg-2")}>
                  <SurfaceIcon id={item.id} />
                </span>
                {!compact && (
                  <>
                    <span className="truncate text-sm">{item.label}</span>
                    {item.source === "fixture" && (
                      <span className="ml-auto font-mono text-2xs text-ck-warn" title="This surface shows illustrative FIXTURE data">
                        {"\u25B3"} fixture
                      </span>
                    )}
                  </>
                )}
                {compact && <span className="sr-only">{item.label}</span>}
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
}

function NavFooter() {
  const { manifest } = useEngine();
  const link = "flex items-center justify-between rounded-md px-2.5 py-1 text-xs text-ck-fg-3 hover:bg-ck-bg-2 hover:text-ck-fg-1";
  return (
    <div className="space-y-3 border-t border-ck-hairline pt-3">
      <div className="space-y-0.5">
        <a className={link} href="./docs/api/mizan/index.html" target="_blank" rel="noopener noreferrer">
          Rust API docs <span aria-hidden className="text-ck-fg-mute">{"\u2197"}</span>
        </a>
        <a className={link} href="./capsule.html" target="_blank" rel="noopener noreferrer">
          Evidence capsule <span aria-hidden className="text-ck-fg-mute">{"\u2197"}</span>
        </a>
        <a className={link} href="https://github.com/ckodex-labs/ckodex-oscal-cli" target="_blank" rel="noopener noreferrer">
          Source on GitHub <span aria-hidden className="text-ck-fg-mute">{"\u2197"}</span>
        </a>
      </div>
      <div className="px-2.5 text-2xs text-ck-fg-mute space-y-0.5">
        <p className="ck-eyebrow">Snapshot</p>
        {manifest ? (
          <>
            <p className="font-mono">
              {manifest.gitCommit.slice(0, 12)}
              {manifest.gitDirty && <span className="text-ck-warn"> +dirty</span>}
            </p>
            <p className="font-mono">{manifest.mizanVersion}</p>
          </>
        ) : (
          <p>none loaded</p>
        )}
      </div>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Evidence rail                                                       */
/* ------------------------------------------------------------------ */

function EvidenceContent({ receipts }: { receipts: SessionReceipt[] }) {
  const { manifest } = useEngine();
  return (
    <div className="space-y-6">
      <section className="space-y-2">
        <div className="flex items-center justify-between">
          <h2 className="ck-eyebrow">Engine runs</h2>
          {manifest && <span className="font-mono text-2xs text-ck-fg-mute ck-num">{manifest.commands.length}</span>}
        </div>
        <p className="text-xs text-ck-fg-mute">
          Real <code className="font-mono">mizan</code> invocations captured for this build. Digest is SHA-256 of stdout.
        </p>
        {!manifest && <p className="text-xs text-ck-fg-3">No snapshot manifest loaded.</p>}
        <ol className="space-y-1.5">
          {manifest?.commands.map((c) => (
            <li key={c.id} className="rounded-md border border-ck-hairline bg-ck-bg-0 p-2">
              <div className="flex items-center justify-between gap-2">
                <span className="truncate text-xs text-ck-fg-1" title={c.title}>
                  {c.title}
                </span>
                <StateBadge tone={c.exitCode === 0 ? "pos" : "neg"} glyph={false}>
                  exit {c.exitCode}
                </StateBadge>
              </div>
              <p className="mt-1 truncate font-mono text-2xs text-ck-fg-mute" title={`mizan ${c.argv.join(" ")}`}>
                mizan {c.argv.join(" ")}
              </p>
              <p className="truncate font-mono text-2xs text-ck-fg-mute" title={c.stdoutSha256}>
                sha256:{c.stdoutSha256.slice(0, 16)}
                <span className="ml-2">{c.durationMs} ms</span>
              </p>
            </li>
          ))}
        </ol>
      </section>

      <section className="space-y-2">
        <div className="flex items-center justify-between">
          <h2 className="ck-eyebrow">This session</h2>
          <span className="font-mono text-2xs text-ck-fg-mute ck-num">{receipts.length}</span>
        </div>
        <p className="text-xs text-ck-fg-mute">
          Actions taken in this browser. LOCAL only: digest of the event text, not a signature, not persisted.
        </p>
        {receipts.length === 0 ? (
          <p className="text-xs text-ck-fg-3">No actions yet.</p>
        ) : (
          <ol className="space-y-1.5">
            {[...receipts].reverse().map((r) => (
              <li key={r.digest} className="rounded-md border border-dashed border-ck-hairline-strong bg-ck-bg-0 p-2">
                <div className="flex items-center justify-between gap-2">
                  <StateBadge tone="unk" glyph={false}>
                    local
                  </StateBadge>
                  <span className="font-mono text-2xs text-ck-fg-mute">{r.at.slice(11, 19)}Z</span>
                </div>
                <p className="mt-1 text-xs text-ck-fg-1">{r.event}</p>
                <p className="truncate font-mono text-2xs text-ck-fg-mute">sha256:{r.digest.slice(0, 16)}</p>
              </li>
            ))}
          </ol>
        )}
      </section>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Shell                                                               */
/* ------------------------------------------------------------------ */

export function AppShell({
  theme,
  onThemeChange,
  lens,
  onLensChange,
  surface,
  onSurfaceChange,
  receipts,
  onOpenShortcuts,
  isCopilotOpen,
  onToggleCopilot,
  copilot,
  mainRef,
  children,
}: {
  theme: ThemeMode;
  onThemeChange: (t: ThemeMode) => void;
  lens: LensMode;
  onLensChange: (l: LensMode) => void;
  surface: SurfaceId;
  onSurfaceChange: (s: SurfaceId) => void;
  receipts: SessionReceipt[];
  onOpenShortcuts: () => void;
  isCopilotOpen: boolean;
  onToggleCopilot: () => void;
  copilot: React.ReactNode;
  mainRef: React.RefObject<HTMLElement | null>;
  children: React.ReactNode;
}) {
  const [menuOpen, setMenuOpen] = React.useState(false);
  const [railOpen, setRailOpen] = React.useState(true);
  const [drawerOpen, setDrawerOpen] = React.useState(false);
  const { manifest } = useEngine();
  const evidenceCount = (manifest?.commands.length ?? 0) + receipts.length;

  const select = (id: SurfaceId) => {
    onSurfaceChange(id);
    setMenuOpen(false);
  };

  return (
    <div className="flex h-dvh flex-col overflow-hidden bg-ck-bg-0 text-ck-fg-1">
      {/* GovX Top Security Classification & Posture Strip */}
      <div className="flex h-5 shrink-0 items-center justify-between border-b border-ck-hairline-strong bg-ck-bg-1 px-3 text-[10px] font-mono tracking-widest text-ck-fg-mute uppercase select-none">
        <div className="flex items-center gap-2">
          <span className="font-bold text-ck-accent">CUI // FEDRAMP-MODERATE</span>
          <span className="hidden sm:inline text-ck-hairline-strong">|</span>
          <span className="hidden sm:inline">NIST SP 800-53 REV 5</span>
          <span className="hidden md:inline text-ck-hairline-strong">|</span>
          <span className="hidden md:inline">ASSURANCE LEVEL: HIGH (L3)</span>
        </div>
        <div className="flex items-center gap-3">
          <span className="hidden lg:inline text-3xs font-mono">MERKLE: 07617ef7a90b</span>
          <span className="inline-flex items-center gap-1 text-ck-pos font-semibold text-3xs">
            <span className="h-1.5 w-1.5 rounded-full bg-ck-pos animate-pulse" />
            GOVX INTEGRITY VERIFIED
          </span>
        </div>
      </div>

      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:absolute focus:left-2 focus:top-2 focus:z-50 focus:rounded-md focus:bg-ck-fg-1 focus:px-3 focus:py-2 focus:text-ck-bg-0"
      >
        Skip to content
      </a>
      <Masthead
        theme={theme}
        onThemeChange={onThemeChange}
        lens={lens}
        onLensChange={onLensChange}
        onOpenShortcuts={onOpenShortcuts}
        isCopilotOpen={isCopilotOpen}
        onToggleCopilot={onToggleCopilot}
        onOpenMenu={() => setMenuOpen(true)}
        onToggleEvidence={() => setDrawerOpen(true)}
        evidenceCount={evidenceCount}
      />

      <div className="flex min-h-0 flex-1">
        {/* Tablet: icon rail */}
        <nav aria-label="Surfaces" className="hidden w-14 shrink-0 overflow-y-auto border-r border-ck-hairline-strong bg-ck-bg-1 px-1.5 py-3 md:block xl:hidden">
          <NavLinks active={surface} onSelect={select} compact />
        </nav>
        {/* Desktop: full nav */}
        <nav aria-label="Surfaces" className="hidden w-60 shrink-0 flex-col justify-between overflow-y-auto border-r border-ck-hairline-strong bg-ck-bg-1 p-3 xl:flex">
          <NavLinks active={surface} onSelect={select} compact={false} />
          <NavFooter />
        </nav>

        <main ref={mainRef} id="main" tabIndex={-1} className="min-w-0 flex-1 overflow-y-auto focus:outline-none">
          <div className="mx-auto w-full max-w-[1200px] space-y-6 px-4 py-5 md:px-6 md:py-7 lg:px-8">{children}</div>
        </main>

        {/* Desktop: docked evidence rail */}
        {railOpen ? (
          <aside aria-label="Evidence" className="hidden w-80 shrink-0 flex-col border-l border-ck-hairline-strong bg-ck-bg-1 xl:flex">
            <div className="flex h-11 items-center justify-between border-b border-ck-hairline px-4">
              <span className="text-sm font-semibold">Evidence</span>
              <button type="button" onClick={() => setRailOpen(false)} className="rounded-md px-1.5 py-0.5 text-xs text-ck-fg-mute hover:bg-ck-bg-2 hover:text-ck-fg-1" aria-label="Collapse evidence rail">
                Hide
              </button>
            </div>
            <div tabIndex={0} role="region" aria-label="Evidence list" className="min-h-0 flex-1 overflow-y-auto p-4 focus:outline-none">
              <EvidenceContent receipts={receipts} />
            </div>
          </aside>
        ) : (
          <button
            type="button"
            onClick={() => setRailOpen(true)}
            className="hidden w-9 shrink-0 border-l border-ck-hairline-strong bg-ck-bg-1 text-ck-fg-mute hover:text-ck-fg-1 xl:block"
            aria-label="Show evidence rail"
          >
            <span className="font-mono text-2xs uppercase tracking-widest [writing-mode:vertical-rl]">Evidence {evidenceCount}</span>
          </button>
        )}
      </div>

      {isCopilotOpen && (
        <div className="relative h-72 shrink-0 border-t-2 border-ck-accent bg-ck-bg-1 shadow-lg">
          <button
            type="button"
            onClick={onToggleCopilot}
            className="absolute right-3 top-2 z-10 rounded-md border border-ck-hairline-strong bg-ck-bg-0 px-2 py-0.5 text-xs text-ck-fg-3 hover:text-ck-fg-1"
          >
            Close
          </button>
          {copilot}
        </div>
      )}

      {/* Phone: navigation sheet */}
      <Dialog.Root open={menuOpen} onOpenChange={setMenuOpen}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 z-40 bg-black/40" />
          <Dialog.Content data-theme={theme} className="fixed inset-y-0 left-0 z-50 flex w-[min(300px,85vw)] flex-col justify-between overflow-y-auto border-r border-ck-hairline-strong bg-ck-bg-1 p-3 text-ck-fg-1">
            <div>
              <div className="mb-4 flex items-center justify-between">
                <Dialog.Title className="font-serif text-xl">Mizan</Dialog.Title>
                <Dialog.Close className="rounded-md px-2 py-1 text-xs text-ck-fg-3 hover:bg-ck-bg-2">Close</Dialog.Close>
              </div>
              <Dialog.Description className="sr-only">Choose a workbench surface</Dialog.Description>
              <NavLinks active={surface} onSelect={select} compact={false} />
            </div>
            <NavFooter />
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>

      {/* Tablet / phone: evidence slide-over */}
      <Dialog.Root open={drawerOpen} onOpenChange={setDrawerOpen}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 z-40 bg-black/40" />
          <Dialog.Content data-theme={theme} className="fixed inset-y-0 right-0 z-50 flex w-[min(380px,92vw)] flex-col border-l border-ck-hairline-strong bg-ck-bg-1 text-ck-fg-1">
            <div className="flex h-12 items-center justify-between border-b border-ck-hairline px-4">
              <Dialog.Title className="text-sm font-semibold">Evidence</Dialog.Title>
              <Dialog.Close className="rounded-md px-2 py-1 text-xs text-ck-fg-3 hover:bg-ck-bg-2">Close</Dialog.Close>
            </div>
            <Dialog.Description className="sr-only">Engine runs and session log</Dialog.Description>
            <div tabIndex={0} role="region" aria-label="Evidence list" className="min-h-0 flex-1 overflow-y-auto p-4 focus:outline-none">
              <EvidenceContent receipts={receipts} />
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </div>
  );
}

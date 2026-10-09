"use client";

import * as React from "react";
import { LensMode, ThemeMode } from "@/lib/oscal-types";
import { EngineProvider } from "@/lib/engine";
import { sha256Hex } from "@/lib/provenance";
import { createAtlasInitialData, BridgeEdge } from "@/lib/atlas-data";
import {
  AppShell,
  ALL_SURFACES,
  type SessionReceipt,
  type SurfaceId,
} from "@/components/shell/app-shell";
import { AtlasSurface } from "@/components/surfaces/atlas-surface";
import { BridgeSurface } from "@/components/surfaces/bridge-surface";
import { ComposerSurface } from "@/components/surfaces/composer-surface";
import { LedgerSurface } from "@/components/surfaces/ledger-surface";
import { DocketSurface } from "@/components/surfaces/docket-surface";
import { PipelineSurface } from "@/components/surfaces/pipeline-surface";
import { JurisdictionSlsaPanel } from "@/components/generative-ui/jurisdiction-slsa-panel";
import { CicdSbomPanel } from "@/components/generative-ui/cicd-sbom-panel";
import { FabricSurface } from "@/components/surfaces/fabric-surface";
import { InspectorSurface } from "@/components/surfaces/inspector-surface";
import { ChatPanel } from "@/components/chat/chat-panel";
import { ShortcutsModal } from "@/components/modals/shortcuts-modal";

const LENSES: LensMode[] = [
  "author",
  "architect",
  "engineer",
  "assessor",
  "risk-owner",
  "ciso",
];

export default function WorkbenchPage() {
  return (
    <EngineProvider>
      <Workbench />
    </EngineProvider>
  );
}

function Workbench() {
  const [theme, setTheme] = React.useState<ThemeMode>("ledger");
  const [lens, setLens] = React.useState<LensMode>("architect");
  const [surface, setSurface] = React.useState<SurfaceId>("atlas");
  const [selectedControlId, setSelectedControlId] = React.useState("ac-2");
  const [receipts, setReceipts] = React.useState<SessionReceipt[]>([]);
  const [isShortcutsOpen, setIsShortcutsOpen] = React.useState(false);
  const [isCopilotOpen, setIsCopilotOpen] = React.useState(false);
  const mainRef = React.useRef<HTMLElement>(null);

  // FIXTURE data for surfaces the engine cannot back yet (Bridge, Ledger, Docket).
  const fixture = React.useMemo(() => createAtlasInitialData(), []);
  const [edgesIso, setEdgesIso] = React.useState<BridgeEdge[]>(fixture.edgesIso);

  React.useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  React.useEffect(() => {
    mainRef.current?.scrollTo({ top: 0, behavior: "instant" });
  }, [surface]);

  /** Record a LOCAL session receipt: SHA-256 of event text + time. Unsigned. */
  const recordLocal = React.useCallback(async (event: string) => {
    const at = new Date().toISOString();
    const digest = await sha256Hex(`${at} ${event}`);
    setReceipts((prev) => [...prev, { event, at, digest }]);
  }, []);

  React.useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setIsCopilotOpen((o) => !o);
        return;
      }
      const t = e.target;
      if (
        t instanceof HTMLInputElement ||
        t instanceof HTMLTextAreaElement ||
        t instanceof HTMLSelectElement ||
        (t instanceof HTMLElement && t.isContentEditable)
      ) {
        return;
      }
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      const match = ALL_SURFACES.find((s) => s.key === e.key);
      if (match) setSurface(match.id);
      if (e.key === "?") setIsShortcutsOpen((o) => !o);
      if (e.key === "t" || e.key === "T") {
        setTheme((p) => (p === "ledger" ? "vault" : p === "vault" ? "hc" : "ledger"));
      }
      if (e.key === "l" || e.key === "L") {
        setLens((p) => LENSES[(LENSES.indexOf(p) + 1) % LENSES.length]);
      }
      if (e.key === "Escape") {
        setIsShortcutsOpen(false);
        setIsCopilotOpen(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const openInComposer = (id: string) => {
    setSelectedControlId(id);
    setSurface("composer");
  };

  return (
    <>
      <AppShell
        theme={theme}
        onThemeChange={setTheme}
        lens={lens}
        onLensChange={setLens}
        surface={surface}
        onSurfaceChange={setSurface}
        receipts={receipts}
        onOpenShortcuts={() => setIsShortcutsOpen(true)}
        isCopilotOpen={isCopilotOpen}
        onToggleCopilot={() => setIsCopilotOpen((o) => !o)}
        mainRef={mainRef}
        copilot={
          <ChatPanel
            selectedControl={selectedControlId}
            lens={lens}
            onNavigateControl={setSelectedControlId}
          />
        }
      >
        {surface === "atlas" && (
          <AtlasSurface
            selectedId={selectedControlId}
            onSelectControl={setSelectedControlId}
            onOpenInComposer={openInComposer}
            lens={lens}
          />
        )}
        {surface === "bridge" && (
          <BridgeSurface
            mer={fixture.mer}
            iso={fixture.iso}
            csf={fixture.csf}
            edgesIso={edgesIso}
            edgesCsf={fixture.edgesCsf}
            onEdgeConfirmHuman={(edgeId) => {
              setEdgesIso((prev) =>
                prev.map((e) =>
                  e.id === edgeId ? { ...e, m: "human", st: "complete", by: "You (this session)" } : e,
                ),
              );
              void recordLocal(`Mapping edge ${edgeId} confirmed (fixture data)`);
            }}
          />
        )}
        {surface === "composer" && (
          <ComposerSurface
            selectedControlId={selectedControlId}
            onSelectControl={setSelectedControlId}
            onRecordLocal={recordLocal}
          />
        )}
        {surface === "ledger" && <LedgerSurface evidence={fixture.evidence} />}
        {surface === "docket" && (
          <DocketSurface
            poams={fixture.poams}
            onSelectPoamControl={(id: string) => {
              setSelectedControlId(id.toLowerCase());
              setSurface("atlas");
            }}
          />
        )}
        {surface === "pipeline" && <PipelineSurface onRecordLocal={recordLocal} />}
        {surface === "jurisdiction" && <JurisdictionSlsaPanel />}
        {surface === "cicd" && <CicdSbomPanel />}
        {surface === "fabric" && <FabricSurface />}
        {surface === "inspector" && <InspectorSurface onRecordLocal={recordLocal} />}
      </AppShell>

      <ShortcutsModal isOpen={isShortcutsOpen} onClose={() => setIsShortcutsOpen(false)} />
    </>
  );
}

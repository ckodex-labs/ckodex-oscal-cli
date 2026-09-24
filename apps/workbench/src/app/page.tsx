"use client";

import * as React from "react";
import { LensMode, ThemeMode } from "@/lib/oscal-types";
import { Masthead } from "@/components/shell/masthead";
import {
  createAtlasInitialData,
  RegionFamily,
  AtlasControl,
  BridgeEdge,
  EvidenceItem,
  PoamItem,
  MappingRow,
} from "@/lib/atlas-data";

import { AtlasNav, SurfaceId } from "@/components/shell/atlas-nav";
import {
  AtlasEvidenceMargin,
  ReceiptEntry,
} from "@/components/shell/atlas-evidence-margin";
import { AtlasSurface } from "@/components/surfaces/atlas-surface";
import { BridgeSurface } from "@/components/surfaces/bridge-surface";
import { ComposerSurface } from "@/components/surfaces/composer-surface";
import { LedgerSurface } from "@/components/surfaces/ledger-surface";
import { DocketSurface } from "@/components/surfaces/docket-surface";
import { PipelineSurface } from "@/components/surfaces/pipeline-surface";
import { JurisdictionSlsaPanel } from "@/components/generative-ui/jurisdiction-slsa-panel";
import { CicdSbomPanel } from "@/components/generative-ui/cicd-sbom-panel";
import { FabricSurface } from "@/components/surfaces/fabric-surface";
import { ChatPanel } from "@/components/chat/chat-panel";
import { PromotionModal } from "@/components/modals/promotion-modal";
import { ShortcutsModal } from "@/components/modals/shortcuts-modal";

export default function WorkbenchPage() {
  const [theme, setTheme] = React.useState<ThemeMode>("ledger");
  const [lens, setLens] = React.useState<LensMode>("architect");
  const [surface, setSurface] = React.useState<SurfaceId>("atlas");

  // Initial domain data from Atlas.dc.html
  const initialData = React.useMemo(() => createAtlasInitialData(), []);
  const [fams] = React.useState<RegionFamily[]>(initialData.fams);
  const [ctrls, setCtrls] = React.useState<AtlasControl[]>(initialData.ctrls);
  const [selectedControlId, setSelectedControlId] =
    React.useState<string>("AC-2");

  const [mer] = React.useState<MappingRow[]>(initialData.mer);
  const [iso] = React.useState<MappingRow[]>(initialData.iso);
  const [csf] = React.useState<MappingRow[]>(initialData.csf);
  const [edgesIso, setEdgesIso] = React.useState<BridgeEdge[]>(
    initialData.edgesIso,
  );
  const [edgesCsf] = React.useState<BridgeEdge[]>(initialData.edgesCsf);
  const [evidence] = React.useState<EvidenceItem[]>(initialData.evidence);
  const [poams] = React.useState<PoamItem[]>(initialData.poams);

  // Atlas Surface state
  const [overlay, setOverlay] = React.useState<
    "state" | "freshness" | "drift" | "poam"
  >("state");
  const [semanticZoom, setSemanticZoom] = React.useState<
    "posture" | "controls"
  >("controls");
  const [isOutline, setIsOutline] = React.useState(false);
  const [pulseActive, setPulseActive] = React.useState(false);

  // Shell state
  const [isMarginOpen, setIsMarginOpen] = React.useState(true);
  const [isPromotionModalOpen, setIsPromotionModalOpen] = React.useState(false);
  const [isShortcutsOpen, setIsShortcutsOpen] = React.useState(false);
  const [isCopilotOpen, setIsCopilotOpen] = React.useState(false);

  const mainRef = React.useRef<HTMLElement>(null);

  // Scroll to top immediately on surface change
  React.useEffect(() => {
    mainRef.current?.scrollTo({ top: 0, behavior: "instant" });
  }, [surface]);

  // Evidence Receipts
  const [receipts, setReceipts] = React.useState<ReceiptEntry[]>([
    {
      ev: "oscal 1.2.3 metaschema verified",
      d: "09:12:40Z",
      hash: "sha256:be41…f001",
      type: "signed",
    },
    {
      ev: "profile MER-MOD resolved",
      d: "09:12:41Z",
      hash: "sha256:71ae…c402",
      type: "observed",
    },
    {
      ev: "Regorus rulepack CIS-K8s passed",
      d: "09:12:42Z",
      hash: "sha256:9f3c…a217",
      type: "signed",
    },
    {
      ev: "SLSA v1.2 In-Toto statement signed",
      d: "09:12:43Z",
      hash: "sha256:c984…5f5d",
      type: "signed",
    },
    {
      ev: "responsible-role assigned to AC-2",
      d: "09:12:45Z",
      hash: "sha256:4b18…c390",
      type: "observed",
    },
  ]);

  // Keyboard Shortcuts (1-9 to switch surfaces, ? for cheatsheet, T for theme, L for lens, ⌘K for copilot, Esc to close modals)
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && (e.key === "k" || e.key === "K")) {
        e.preventDefault();
        setIsCopilotOpen((o) => !o);
        return;
      }
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }
      if (e.key === "1") setSurface("atlas");
      if (e.key === "2") setSurface("bridge");
      if (e.key === "3") setSurface("composer");
      if (e.key === "4") setSurface("ledger");
      if (e.key === "5") setSurface("docket");
      if (e.key === "6") setSurface("pipeline");
      if (e.key === "7") setSurface("jurisdiction");
      if (e.key === "8") setSurface("cicd");
      if (e.key === "9") setSurface("fabric");
      if (e.key === "?") setIsShortcutsOpen((o) => !o);
      if (e.key === "t" || e.key === "T") {
        setTheme((prev) =>
          prev === "ledger" ? "vault" : prev === "vault" ? "hc" : "ledger",
        );
      }
      if (e.key === "l" || e.key === "L") {
        const lenses: LensMode[] = [
          "author",
          "architect",
          "engineer",
          "assessor",
          "risk-owner",
          "ciso",
        ];
        setLens((prev) => {
          const nextIdx = (lenses.indexOf(prev) + 1) % lenses.length;
          return lenses[nextIdx];
        });
      }
      if (e.key === "Escape") {
        setIsPromotionModalOpen(false);
        setIsShortcutsOpen(false);
        setIsCopilotOpen(false);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const handleReplayPulse = () => {
    setPulseActive(true);
    setTimeout(() => setPulseActive(false), 2400);
  };

  const handleEdgeConfirmHuman = (edgeId: string) => {
    setEdgesIso((prev) =>
      prev.map((e) =>
        e.id === edgeId
          ? {
              ...e,
              m: "human",
              st: "complete",
              by: "Human Architect (Verified)",
            }
          : e,
      ),
    );
    setReceipts((prev) => [
      ...prev,
      {
        ev: `Mapping edge ${edgeId} confirmed by human architect`,
        d: new Date().toISOString().slice(11, 19) + "Z",
        hash: "sha256:human_conf_" + Math.random().toString(36).slice(2, 8),
        type: "observed",
      },
    ]);
  };

  const handlePublishControl = (id: string) => {
    setCtrls((prev) =>
      prev.map((c) =>
        c.id === id ? { ...c, st: "implemented", att: true } : c,
      ),
    );
    setReceipts((prev) => [
      ...prev,
      {
        ev: `Control ${id} published into MER-MOD baseline`,
        d: new Date().toISOString().slice(11, 19) + "Z",
        hash: "sha256:pub_" + Math.random().toString(36).slice(2, 8),
        type: "signed",
      },
    ]);
  };

  const counts: Record<string, string | number> = {
    atlas: ctrls.length,
    bridge: edgesIso.length,
    composer: "⊭ 1",
    ledger: evidence.length,
    docket: poams.length,
    pipeline: "#1424",
    jurisdiction: "4",
    cicd: "12",
    fabric: "3",
  };

  return (
    <div
      data-theme={theme}
      className="flex h-screen flex-col overflow-hidden bg-ck-bg-0 text-ck-fg-1 transition-colors duration-200"
    >
      {/* Top Masthead */}
      <Masthead
        theme={theme}
        onThemeChange={setTheme}
        lens={lens}
        onLensChange={setLens}
        onOpenPromotion={() => setIsPromotionModalOpen(true)}
        onOpenShortcuts={() => setIsShortcutsOpen(true)}
        isCopilotOpen={isCopilotOpen}
        onToggleCopilot={() => setIsCopilotOpen((o) => !o)}
      />

      {/* Main 3-Column Shell (Nav Rail | Main Active Surface | Evidence Margin) */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left Nav Rail */}
        <AtlasNav
          activeSurface={surface}
          onSelectSurface={setSurface}
          counts={counts}
        />

        {/* Center Main Work Surface */}
        <main
          ref={mainRef}
          className="flex-1 overflow-y-auto p-6 lg:p-8 bg-ck-bg-0"
        >
          <div className="max-w-7xl mx-auto space-y-6">
            {surface === "atlas" && (
              <AtlasSurface
                fams={fams}
                ctrls={ctrls}
                selectedId={selectedControlId}
                onSelectControl={(id) => setSelectedControlId(id)}
                overlay={overlay}
                onOverlayChange={setOverlay}
                semanticZoom={semanticZoom}
                onSemanticZoomChange={setSemanticZoom}
                isOutline={isOutline}
                onToggleOutline={() => setIsOutline((o) => !o)}
                onReplayPulse={handleReplayPulse}
                pulseActive={pulseActive}
                onOpenInComposer={(id) => {
                  setSelectedControlId(id);
                  setSurface("composer");
                }}
              />
            )}

            {surface === "bridge" && (
              <BridgeSurface
                mer={mer}
                iso={iso}
                csf={csf}
                edgesIso={edgesIso}
                edgesCsf={edgesCsf}
                onEdgeConfirmHuman={handleEdgeConfirmHuman}
              />
            )}

            {surface === "composer" && (
              <ComposerSurface
                selectedControlId={selectedControlId}
                onSelectControl={setSelectedControlId}
                onPublishControl={handlePublishControl}
              />
            )}

            {surface === "ledger" && <LedgerSurface evidence={evidence} />}

            {surface === "docket" && (
              <DocketSurface
                poams={poams}
                onSelectPoamControl={(ctrlId) => {
                  setSelectedControlId(ctrlId);
                  setSurface("atlas");
                }}
              />
            )}

            {surface === "pipeline" && (
              <PipelineSurface
                onTriggerPipelineRun={() => {
                  setReceipts((prev) => [
                    ...prev,
                    {
                      ev: "Pipeline #1425 executed · SARIF & GitLab emitted",
                      d: new Date().toISOString().slice(11, 19) + "Z",
                      hash:
                        "sha256:pipe_" + Math.random().toString(36).slice(2, 8),
                      type: "signed",
                    },
                  ]);
                }}
              />
            )}

            {surface === "jurisdiction" && <JurisdictionSlsaPanel />}

            {surface === "cicd" && <CicdSbomPanel />}

            {surface === "fabric" && <FabricSurface />}
          </div>
        </main>

        {/* Right Collapsible Evidence Margin */}
        <AtlasEvidenceMargin
          isOpen={isMarginOpen}
          onToggle={() => setIsMarginOpen((m) => !m)}
          receipts={receipts}
        />
      </div>

      {/* Collapsible AI Copilot Drawer (Toggle with ⌘K or Masthead button) */}
      {isCopilotOpen && (
        <div className="border-t-2 border-ck-accent bg-ck-bg-1 h-64 overflow-hidden relative shadow-lg z-30 animate-in slide-in-from-bottom duration-200">
          <button
            type="button"
            onClick={() => setIsCopilotOpen(false)}
            title="Close Copilot (Esc)"
            className="absolute top-2.5 right-4 z-40 text-ck-fg-mute hover:text-ck-fg-1 font-mono text-xs px-2 py-0.5 border border-ck-hairline-strong bg-ck-bg-0"
          >
            [X] Close Copilot
          </button>
          <ChatPanel
            selectedControl={selectedControlId}
            lens={lens}
            onNavigateControl={setSelectedControlId}
          />
        </div>
      )}

      {/* Governed Promotion Modal */}
      <PromotionModal
        isOpen={isPromotionModalOpen}
        onClose={() => setIsPromotionModalOpen(false)}
        onConfirmPromote={() => {
          setReceipts((prev) => [
            ...prev,
            {
              ev: "Promoted to production release tag v1.4.3",
              d: new Date().toISOString().slice(11, 19) + "Z",
              hash: "sha256:release_" + Math.random().toString(36).slice(2, 8),
              type: "signed",
            },
          ]);
        }}
      />

      {/* Keyboard Shortcuts Navigation Modal */}
      <ShortcutsModal
        isOpen={isShortcutsOpen}
        onClose={() => setIsShortcutsOpen(false)}
      />
    </div>
  );
}

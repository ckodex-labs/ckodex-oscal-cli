"use client";

import * as React from "react";
import { LensMode, ThemeMode } from "@/lib/oscal-types";
import { LensSelector } from "./lens-selector";
import { ThemeToggle } from "./theme-toggle";

interface MastheadProps {
  theme: ThemeMode;
  onThemeChange: (t: ThemeMode) => void;
  lens: LensMode;
  onLensChange: (l: LensMode) => void;
  onOpenPromotion?: () => void;
  onOpenShortcuts?: () => void;
  isCopilotOpen?: boolean;
  onToggleCopilot?: () => void;
}

export function Masthead({
  theme,
  onThemeChange,
  lens,
  onLensChange,
  onOpenPromotion,
  onOpenShortcuts,
  isCopilotOpen,
  onToggleCopilot,
}: MastheadProps) {
  const [copied, setCopied] = React.useState(false);
  const [env, setEnv] = React.useState<"dev" | "stage" | "prod">("stage");

  const handleCopyUrn = () => {
    navigator.clipboard.writeText("urn:meridian:atlas:workspace");
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <header className="flex flex-col border-b border-ck-hairline-strong bg-ck-bg-0 px-4 py-2 text-ck-fg-1">
      {/* Upper Masthead Bar */}
      <div className="flex flex-wrap items-center gap-3">
        <div className="flex items-center gap-2 shrink-0">
          <span className="font-serif text-2xl font-normal tracking-tight text-ck-fg-1 whitespace-nowrap shrink-0">
            MIZAN
          </span>
          <span className="border-l border-ck-hairline pl-2 font-mono text-[11px] text-ck-fg-mute whitespace-nowrap shrink-0 hidden sm:inline">
            OSCAL Graph of Record &amp; Compliance Workbench
          </span>
        </div>

        <div className="ml-auto flex items-center gap-2">
          {onToggleCopilot && (
            <button
              type="button"
              onClick={onToggleCopilot}
              title="Toggle Atlas Copilot (⌘K)"
              className={`px-2.5 py-1 border font-mono text-[11px] transition-colors rounded-xs flex items-center gap-1.5 ${
                isCopilotOpen
                  ? "bg-ck-fg-1 text-ck-bg-0 border-ck-fg-1 font-semibold"
                  : "border-ck-hairline-strong bg-ck-bg-1 text-ck-fg-2 hover:text-ck-fg-1 hover:border-ck-accent"
              }`}
            >
              <span className="text-ck-accent">✨</span>
              <span>Copilot</span>
              <span className="text-[9px] opacity-70 ml-0.5">⌘K</span>
            </button>
          )}
          <LensSelector lens={lens} onLensChange={onLensChange} />
          <ThemeToggle theme={theme} onThemeChange={onThemeChange} />
          {onOpenShortcuts && (
            <button
              type="button"
              onClick={onOpenShortcuts}
              title="Keyboard Navigation Sheet (?)"
              className="px-2 py-1 border border-ck-hairline-strong bg-ck-bg-1 font-mono text-[11px] text-ck-fg-2 hover:text-ck-fg-1 hover:border-ck-accent transition-colors rounded-xs"
            >
              ?
            </button>
          )}
        </div>
      </div>

      {/* Lower Context & Metadata Sub-bar */}
      <div className="mt-2 flex flex-wrap items-center gap-3 border-t border-ck-hairline pt-1.5 font-mono text-[11px] text-ck-fg-2">
        <span className="text-[10px] uppercase tracking-wider text-ck-fg-mute">
          workspace:
        </span>
        <button
          type="button"
          onClick={handleCopyUrn}
          className="ck-hash"
          title="Click to copy workspace URN"
        >
          {copied ? "copied!" : "urn:meridian:atlas:workspace"}
        </button>

        <span className="text-ck-fg-mute">·</span>
        <span className="text-ck-fg-mute">oscal 1.2.3</span>
        <span className="text-ck-fg-mute">·</span>
        <span>profile NIST SP 800-53 r5 (MOD)</span>

        {/* Environment Selector Segment */}
        <div className="ck-segment ml-1" role="group" aria-label="environment">
          {(["dev", "stage", "prod"] as const).map((e) => (
            <button
              key={e}
              type="button"
              data-active={env === e}
              onClick={() => setEnv(e)}
              className="ck-segment-btn uppercase"
            >
              {e}
            </button>
          ))}
        </div>

        {/* Governed Promotion Button */}
        {env !== "prod" && onOpenPromotion && (
          <button
            type="button"
            onClick={onOpenPromotion}
            title="Promotion is a governed act — blocking gates must hold"
            className="px-2.5 py-0.5 border border-ck-accent bg-ck-accent/10 font-mono text-[10.5px] font-medium text-ck-accent hover:bg-ck-accent hover:text-white transition-colors rounded-xs flex items-center gap-1"
          >
            <span>promote → prod</span>
          </button>
        )}

        <div className="ml-auto flex items-center gap-2">
          <span className="text-[10px] text-ck-fg-mute">engine:</span>
          <span className="inline-flex items-center gap-1 font-semibold text-green-700 dark:text-green-400">
            <span className="h-1.5 w-1.5 rounded-full bg-green-500 animate-pulse" />
            RUST METASCHEMA CORE ONLINE
          </span>
        </div>
      </div>
    </header>
  );
}

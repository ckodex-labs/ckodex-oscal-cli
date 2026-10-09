"use client";

/**
 * Copilot drawer.
 *
 * No language model is connected in this build. The drawer accepts a small
 * fixed set of commands (see ./commands.ts) and answers each one from
 * captured engine output (SNAPSHOT) or, when the local engine is reachable,
 * from a read-only engine call (LIVE). It never composes free-form answers.
 */

import * as React from "react";
import { runLive, useEngine, useSnapshot } from "@/lib/engine";
import type { BlastRadiusReport, FedrampReport, LensMode } from "@/lib/oscal-types";
import { StateBadge } from "@/components/kit";
import type { AtlasFile, ValidateReport } from "../generative-ui/engine-types";
import { MessageItem, type AnswerSources } from "./message-item";
import {
  COMMAND_HELP,
  parseCommand,
  SAMPLE_SSP,
  SNAPSHOT_BLAST_TARGET,
  type Answer,
  type Entry,
} from "./commands";

interface ChatPanelProps {
  selectedControl?: string;
  /** Accepted for shell compatibility; lookups do not vary by lens. */
  lens?: LensMode;
  onNavigateControl?: (id: string) => void;
}

const ATLAS_OPTS = { file: "atlas.json" };

export function ChatPanel({ selectedControl, onNavigateControl }: ChatPanelProps = {}) {
  const { status } = useEngine();
  const atlas = useSnapshot<AtlasFile>("nist-moderate-resolve", ATLAS_OPTS);
  const blast = useSnapshot<BlastRadiusReport>("ssp-blast-radius");
  const fedramp = useSnapshot<FedrampReport>("ssp-fedramp-validate");
  const vCatalog = useSnapshot<ValidateReport>("nist-catalog-validate");
  const vBaseline = useSnapshot<ValidateReport>("nist-moderate-validate");
  const vSsp = useSnapshot<ValidateReport>("ssp-validate");

  const sources: AnswerSources = {
    atlas,
    blast,
    fedramp,
    validate: { catalog: vCatalog, baseline: vBaseline, ssp: vSsp },
  };

  const [entries, setEntries] = React.useState<Entry[]>([]);
  const [input, setInput] = React.useState("");
  const [busy, setBusy] = React.useState(false);
  const nextId = React.useRef(1);
  const listRef = React.useRef<HTMLDivElement>(null);
  const inputRef = React.useRef<HTMLInputElement>(null);

  React.useEffect(() => {
    const el = listRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [entries]);

  React.useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const push = (inputText: string, answer: Answer) => {
    const id = nextId.current++;
    setEntries((prev) => [...prev.slice(-29), { id, input: inputText, answer }]);
    return id;
  };
  const replace = (id: number, answer: Answer) =>
    setEntries((prev) => prev.map((e) => (e.id === id ? { ...e, answer } : e)));

  const run = async (raw: string) => {
    const text = raw.trim();
    if (!text || busy) return;
    setInput("");
    const p = parseCommand(text);
    switch (p.type) {
      case "help":
        push(text, { kind: "help" });
        return;
      case "control":
        push(text, { kind: "control", id: p.id });
        return;
      case "fedramp":
        push(text, { kind: "fedramp" });
        return;
      case "validate":
        push(text, { kind: "validate", doc: p.doc });
        return;
      case "blast": {
        if (p.target === SNAPSHOT_BLAST_TARGET) {
          push(text, { kind: "blast-snapshot" });
          return;
        }
        if (status.kind !== "live") {
          push(text, {
            kind: "note",
            tone: "warn",
            text: `The snapshot only captured the blast radius of ${SNAPSHOT_BLAST_TARGET}. Computing ${p.target} needs the local engine, which is not connected.`,
          });
          return;
        }
        const id = push(text, { kind: "pending", text: `Running mizan blast-radius --target ${p.target} ${SAMPLE_SSP}` });
        setBusy(true);
        try {
          const r = await runLive<BlastRadiusReport>(["blast-radius", "--target", p.target, SAMPLE_SSP]);
          replace(id, { kind: "blast-live", report: r.data, provenance: r.provenance });
        } catch (e) {
          replace(id, { kind: "note", tone: "neg", text: e instanceof Error ? e.message : String(e) });
        } finally {
          setBusy(false);
        }
        return;
      }
      case "unknown":
        push(text, {
          kind: "note",
          tone: "info",
          text: "Not a recognised command. No language model is connected, so free-form questions cannot be answered. Type help for the command list.",
        });
        return;
    }
  };

  const chips = [
    ...(selectedControl ? [`show ${selectedControl}`] : []),
    "show ac-2",
    "blast radius",
    "fedramp",
    "validate ssp",
    "help",
  ].filter((c, i, a) => a.indexOf(c) === i);

  return (
    <div className="flex h-full min-h-0 flex-col bg-ck-bg-1 text-ck-fg-1">
      <header className="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1 border-b border-ck-hairline px-3 py-2 pr-20">
        <h2 className="text-sm font-semibold">Copilot</h2>
        <StateBadge tone="unk">No model</StateBadge>
        <p className="min-w-0 text-xs text-ck-fg-3">
          No language model is connected in this build. Only the fixed lookups below work.
        </p>
      </header>

      <div ref={listRef} className="min-h-0 flex-1 overflow-y-auto px-3 py-2">
        {entries.length === 0 ? (
          <div className="space-y-1 text-sm text-ck-fg-3">
            <p>Type a command or pick one below. Answers come from captured engine output:</p>
            <ul className="space-y-0.5">
              {COMMAND_HELP.map((c) => (
                <li key={c.cmd} className="flex min-w-0 flex-wrap gap-x-2 text-xs">
                  <code className="font-mono text-ck-fg-1">{c.cmd}</code>
                  <span>{c.what}</span>
                </li>
              ))}
            </ul>
          </div>
        ) : (
          <ol className="space-y-3" aria-live="polite">
            {entries.map((e) => (
              <MessageItem key={e.id} entry={e} sources={sources} onNavigateControl={onNavigateControl} />
            ))}
          </ol>
        )}
      </div>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          void run(input);
        }}
        className="flex min-w-0 flex-col gap-1.5 border-t border-ck-hairline px-3 py-2 md:flex-row md:items-center"
      >
        <div className="flex min-w-0 gap-1.5 overflow-x-auto md:shrink-0" role="group" aria-label="Example commands">
          {chips.map((c) => (
            <button
              key={c}
              type="button"
              onClick={() => void run(c)}
              disabled={busy}
              className="shrink-0 whitespace-nowrap rounded-sm border border-ck-hairline-strong bg-ck-bg-0 px-2 py-0.5 font-mono text-xs text-ck-fg-2 hover:text-ck-fg-1 disabled:opacity-50"
            >
              {c}
            </button>
          ))}
        </div>
        <div className="flex min-w-0 flex-1 items-center gap-2">
          <label htmlFor="copilot-input" className="sr-only">
            Command
          </label>
          <input
            id="copilot-input"
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="show ac-2"
            autoComplete="off"
            spellCheck={false}
            className="h-8 min-w-0 flex-1 rounded-md border border-ck-hairline-strong bg-ck-bg-0 px-2 font-mono text-sm text-ck-fg-1 placeholder:text-ck-fg-mute focus:outline-none focus:ring-1 focus:ring-ck-accent"
          />
          <button
            type="submit"
            disabled={busy || !input.trim()}
            className="h-8 shrink-0 rounded-md border border-ck-fg-1 bg-ck-fg-1 px-3 text-xs font-medium text-ck-bg-0 disabled:opacity-50"
          >
            Run
          </button>
        </div>
      </form>
    </div>
  );
}

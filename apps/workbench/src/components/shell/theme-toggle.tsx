"use client";

import * as React from "react";
import type { ThemeMode } from "@/lib/oscal-types";

interface ThemeToggleProps {
  theme: ThemeMode;
  onThemeChange: (t: ThemeMode) => void;
}

const THEMES: { id: ThemeMode; label: string; full: string }[] = [
  { id: "ledger", label: "Ledger", full: "Ledger (light)" },
  { id: "vault", label: "Vault", full: "Vault (dark)" },
  { id: "hc", label: "HC", full: "High contrast" },
];

/** Three-way theme switch (T cycles). Radio semantics with arrow-key support. */
export function ThemeToggle({ theme, onThemeChange }: ThemeToggleProps) {
  const refs = React.useRef<(HTMLButtonElement | null)[]>([]);
  const onKeyDown = (e: React.KeyboardEvent, i: number) => {
    const d = e.key === "ArrowRight" || e.key === "ArrowDown" ? 1 : e.key === "ArrowLeft" || e.key === "ArrowUp" ? -1 : 0;
    if (!d) return;
    e.preventDefault();
    const n = (i + d + THEMES.length) % THEMES.length;
    onThemeChange(THEMES[n].id);
    refs.current[n]?.focus();
  };
  return (
    <div className="ck-segment h-8" role="radiogroup" aria-label="Theme">
      {THEMES.map((t, i) => {
        const active = theme === t.id;
        return (
          <button
            key={t.id}
            ref={(el) => {
              refs.current[i] = el;
            }}
            type="button"
            role="radio"
            aria-checked={active}
            aria-label={t.full}
            title={`${t.full} (T cycles)`}
            tabIndex={active ? 0 : -1}
            data-active={active}
            onClick={() => onThemeChange(t.id)}
            onKeyDown={(e) => onKeyDown(e, i)}
            className="ck-segment-btn"
          >
            {t.label}
          </button>
        );
      })}
    </div>
  );
}

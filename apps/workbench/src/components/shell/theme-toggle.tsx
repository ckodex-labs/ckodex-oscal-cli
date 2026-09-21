"use client";

import * as React from "react";
import { ThemeMode } from "@/lib/oscal-types";

interface ThemeToggleProps {
  theme: ThemeMode;
  onThemeChange: (t: ThemeMode) => void;
}

export function ThemeToggle({ theme, onThemeChange }: ThemeToggleProps) {
  const themes: ThemeMode[] = ["ledger", "vault", "hc"];

  return (
    <div className="ck-segment" role="group" aria-label="theme switcher">
      {themes.map((t) => (
        <button
          key={t}
          type="button"
          data-active={theme === t}
          onClick={() => onThemeChange(t)}
          className="ck-segment-btn uppercase"
        >
          {t}
        </button>
      ))}
    </div>
  );
}

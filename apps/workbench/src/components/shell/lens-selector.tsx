"use client";

import * as React from "react";
import type { LensMode } from "@/lib/oscal-types";

interface LensSelectorProps {
  lens: LensMode;
  onLensChange: (l: LensMode) => void;
}

const LENSES: { id: LensMode; name: string }[] = [
  { id: "author", name: "Author" },
  { id: "architect", name: "Architect" },
  { id: "engineer", name: "Engineer" },
  { id: "assessor", name: "Assessor" },
  { id: "risk-owner", name: "Risk owner" },
  { id: "ciso", name: "CISO" },
];

/** Compact native select; fits the 56px masthead and is keyboard/AT friendly. */
export function LensSelector({ lens, onLensChange }: LensSelectorProps) {
  const id = React.useId();
  return (
    <div className="inline-flex h-8 items-center gap-1.5 rounded-md border border-ck-hairline-strong bg-ck-bg-1 pl-2 pr-1 text-xs">
      <label htmlFor={id} className="text-ck-fg-mute">
        Lens
      </label>
      <select
        id={id}
        value={lens}
        onChange={(e) => onLensChange(e.target.value as LensMode)}
        title="Workbench lens (L cycles)"
        className="h-6 cursor-pointer rounded-sm bg-transparent pr-1 text-xs font-medium text-ck-fg-1 focus:outline-none focus-visible:ring-1 focus-visible:ring-ck-accent"
      >
        {LENSES.map((l) => (
          <option key={l.id} value={l.id}>
            {l.name}
          </option>
        ))}
      </select>
    </div>
  );
}

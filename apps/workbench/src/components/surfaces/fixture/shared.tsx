"use client";

/**
 * Shared pieces for the FIXTURE-backed surfaces (Bridge, Ledger, Docket).
 * Provenance copy names the real engine input that would replace each panel.
 */

import * as React from "react";
import { cn } from "@/lib/utils";
import { fixture } from "@/lib/provenance";
import { FIXTURE_REFERENCE_DATE } from "@/lib/atlas-data";

export const BRIDGE_PROVENANCE = fixture(
  "illustrative mapping",
  "Hand-written sample mapping. A real Bridge would render an OSCAL mapping-collection validated and inspected with `mizan mapping`. These rows, edges, confidences and proposers describe no real system.",
);

export const LEDGER_PROVENANCE = fixture(
  "illustrative evidence",
  "Hand-written sample evidence list. A real Ledger would list objects stored with `mizan cas put` and inspected or checked with `mizan evidence get` / `mizan evidence verify`. No object here is stored, hashed, or signed.",
);

export const DOCKET_PROVENANCE = fixture(
  "illustrative POA&M",
  "Hand-written sample plan of action and milestones. A real Docket would render an OSCAL POA&M validated and inspected with `mizan poam`. Owners are role names; dates are offsets from a fixed fixture reference date.",
);

export const FIXTURE_DATE_NOTE = `Dates and ages are relative to the fixture reference date ${FIXTURE_REFERENCE_DATE}, not today.`;

/** Two-column key/value list used by detail panels. */
export function DetailList({
  items,
  className,
}: {
  items: { label: string; value: React.ReactNode }[];
  className?: string;
}) {
  return (
    <dl
      className={cn(
        "grid grid-cols-[minmax(0,9rem)_minmax(0,1fr)] gap-x-3 gap-y-2 text-sm",
        className,
      )}
    >
      {items.map((it) => (
        <React.Fragment key={it.label}>
          <dt className="text-xs text-ck-fg-mute pt-px">{it.label}</dt>
          <dd className="min-w-0 break-words text-ck-fg-2">{it.value}</dd>
        </React.Fragment>
      ))}
    </dl>
  );
}

/** Small labelled field wrapper for filter controls. */
export function FilterField({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-w-0 flex-wrap items-center gap-2">
      <span className="ck-eyebrow">{label}</span>
      {children}
    </div>
  );
}

export function SearchInput({
  value,
  onChange,
  placeholder,
  label,
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder: string;
  label: string;
}) {
  return (
    <input
      type="search"
      aria-label={label}
      value={value}
      placeholder={placeholder}
      onChange={(e) => onChange(e.target.value)}
      className="h-8 w-full min-w-0 rounded-md border border-ck-hairline-strong bg-ck-bg-0 px-2.5 text-sm text-ck-fg-1 placeholder:text-ck-fg-mute focus:outline-none focus-visible:ring-2 focus-visible:ring-ck-accent sm:w-64"
    />
  );
}

export function ActionButton({
  className,
  variant = "secondary",
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "secondary" | "ghost";
}) {
  return (
    <button
      type="button"
      {...props}
      className={cn(
        "inline-flex h-8 items-center justify-center gap-1.5 whitespace-nowrap rounded-md px-3 text-sm font-medium transition-colors",
        "focus:outline-none focus-visible:ring-2 focus-visible:ring-ck-accent disabled:cursor-not-allowed disabled:text-ck-fg-mute",
        variant === "primary" && "bg-ck-fg-1 text-ck-bg-0 hover:bg-ck-fg-2",
        variant === "secondary" &&
          "border border-ck-hairline-strong bg-ck-bg-0 text-ck-fg-1 hover:bg-ck-bg-2",
        variant === "ghost" && "text-ck-fg-2 hover:bg-ck-bg-2",
        className,
      )}
    />
  );
}

/**
 * Scroll the referenced element into view (nearest edge) when `key` changes
 * after mount. A no-op when the element is already visible, e.g. a sticky
 * side panel on desktop; useful when the detail panel is stacked below.
 */
export function useRevealOnChange<T extends HTMLElement>(key: unknown) {
  const ref = React.useRef<T>(null);
  const first = React.useRef(true);
  React.useEffect(() => {
    if (first.current) {
      first.current = false;
      return;
    }
    ref.current?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, [key]);
  return ref;
}

/** Four stat tiles: 2 columns, then 4 from lg. Avoids a 3+1 orphan row. */
export function StatRow({ children }: { children: React.ReactNode }) {
  return <div className="grid grid-cols-2 gap-2 lg:grid-cols-4">{children}</div>;
}

export function pct(n: number) {
  return `${Math.round(n * 100)}%`;
}

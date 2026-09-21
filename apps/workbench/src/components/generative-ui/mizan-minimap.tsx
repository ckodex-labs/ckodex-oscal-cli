"use client";

import * as React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Compass, ShieldCheck } from "lucide-react";

interface MizanMinimapProps {
  activeControl?: string;
  onSelectControl?: (controlIdentifier: string) => void;
}

export function MizanMinimap({
  activeControl = "ac-1",
  onSelectControl,
}: MizanMinimapProps) {
  const controlFamilies = [
    {
      identifier: "AC",
      name: "Access Control",
      x: 20,
      y: 20,
      width: 120,
      height: 80,
      controls: ["ac-1", "ac-2", "ac-3"],
    },
    {
      identifier: "AT",
      name: "Awareness & Training",
      x: 150,
      y: 20,
      width: 120,
      height: 80,
      controls: ["at-1", "at-2"],
    },
    {
      identifier: "AU",
      name: "Audit & Accountability",
      x: 280,
      y: 20,
      width: 120,
      height: 80,
      controls: ["au-1", "au-2", "au-3"],
    },
    {
      identifier: "CA",
      name: "Assessment & Auth",
      x: 20,
      y: 110,
      width: 120,
      height: 80,
      controls: ["ca-1", "ca-2"],
    },
    {
      identifier: "CM",
      name: "Config Management",
      x: 150,
      y: 110,
      width: 120,
      height: 80,
      controls: ["cm-1", "cm-2", "cm-8"],
    },
    {
      identifier: "CP",
      name: "Contingency Planning",
      x: 280,
      y: 110,
      width: 120,
      height: 80,
      controls: ["cp-1", "cp-2"],
    },
    {
      identifier: "IA",
      name: "Ident & Auth",
      x: 20,
      y: 200,
      width: 120,
      height: 80,
      controls: ["ia-1", "ia-2", "ia-5"],
    },
    {
      identifier: "SC",
      name: "System & Comms",
      x: 150,
      y: 200,
      width: 120,
      height: 80,
      controls: ["sc-1", "sc-7", "sc-13"],
    },
    {
      identifier: "SI",
      name: "System & Info Integrity",
      x: 280,
      y: 200,
      width: 120,
      height: 80,
      controls: ["si-1", "si-2", "si-4"],
    },
  ];

  return (
    <Card className="border-ck-hairline-strong bg-ck-bg-1 shadow-[3px_3px_0_var(--ck-fg-1)]">
      <CardHeader className="flex flex-row items-center justify-between pb-2 bg-ck-bg-2/50">
        <div className="flex items-center gap-2">
          <Compass className="h-4 w-4 text-accent" />
          <CardTitle className="text-base font-serif text-ck-fg-1">
            Mizan Spatial Territory
          </CardTitle>
        </div>
        <div className="flex items-center gap-1.5 font-mono text-[10px] text-ck-fg-mute">
          <ShieldCheck className="h-3.5 w-3.5 text-accent" />
          <span>NIST SP 800-53 r5 Grid</span>
        </div>
      </CardHeader>

      <CardContent className="p-3 bg-ck-bg-0">
        <svg
          viewBox="0 0 420 300"
          className="w-full h-auto border border-ck-hairline bg-ck-bg-0"
        >
          {/* Spatial Corridors */}
          <line
            x1="80"
            y1="60"
            x2="210"
            y2="60"
            stroke="var(--ck-hairline-strong)"
            strokeDasharray="3 3"
          />
          <line
            x1="210"
            y1="60"
            x2="340"
            y2="60"
            stroke="var(--ck-hairline-strong)"
            strokeDasharray="3 3"
          />
          <line
            x1="80"
            y1="60"
            x2="80"
            y2="150"
            stroke="var(--ck-hairline-strong)"
            strokeDasharray="3 3"
          />
          <line
            x1="210"
            y1="150"
            x2="210"
            y2="240"
            stroke="var(--ck-hairline-strong)"
            strokeDasharray="3 3"
          />
          <line
            x1="80"
            y1="240"
            x2="210"
            y2="240"
            stroke="var(--ck-hairline-strong)"
            strokeDasharray="3 3"
          />

          {/* Control Family Regions */}
          {controlFamilies.map((family) => {
            const isContained = family.controls.includes(
              activeControl.toLowerCase(),
            );
            return (
              <g
                key={family.identifier}
                className="cursor-pointer transition-opacity hover:opacity-90"
                onClick={() =>
                  onSelectControl && onSelectControl(family.controls[0])
                }
              >
                <rect
                  x={family.x}
                  y={family.y}
                  width={family.width}
                  height={family.height}
                  fill="var(--ck-bg-1)"
                  stroke={
                    isContained
                      ? "var(--ck-accent)"
                      : "var(--ck-hairline-strong)"
                  }
                  strokeWidth={isContained ? 2 : 1}
                />
                <text
                  x={family.x + 8}
                  y={family.y + 18}
                  fill="var(--ck-accent)"
                  fontSize="12"
                  fontWeight="bold"
                  fontFamily="var(--font-mono)"
                >
                  {family.identifier}
                </text>
                <text
                  x={family.x + 8}
                  y={family.y + 32}
                  fill="var(--ck-fg-1)"
                  fontSize="9"
                  fontFamily="sans-serif"
                >
                  {family.name}
                </text>

                {/* Control Nodes */}
                <g transform={`translate(${family.x + 8}, ${family.y + 44})`}>
                  {family.controls.map((controlId, index) => {
                    const isNodeActive =
                      controlId.toLowerCase() === activeControl.toLowerCase();
                    return (
                      <g
                        key={controlId}
                        transform={`translate(${index * 34}, 0)`}
                      >
                        <rect
                          width="28"
                          height="18"
                          fill={
                            isNodeActive ? "var(--ck-accent)" : "var(--ck-bg-0)"
                          }
                          stroke="var(--ck-hairline-strong)"
                          strokeWidth="1"
                        />
                        <text
                          x="14"
                          y="12"
                          textAnchor="middle"
                          fill={
                            isNodeActive ? "var(--ck-bg-0)" : "var(--ck-fg-1)"
                          }
                          fontSize="8"
                          fontWeight="bold"
                          fontFamily="var(--font-mono)"
                        >
                          {controlId.toUpperCase()}
                        </text>
                      </g>
                    );
                  })}
                </g>
              </g>
            );
          })}
        </svg>
      </CardContent>
    </Card>
  );
}

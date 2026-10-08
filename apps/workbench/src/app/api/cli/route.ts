/**
 * Local Development HTTP-to-CLI Route (/api/cli)
 *
 * ARCHITECTURAL CONTRACT:
 * - Local Development Bridge: This endpoint exposes the native `mizan` CLI toolchain to the workbench
 *   during local development by invoking `callOscalCli`.
 * - Offline / Static Export Mode: Next.js API routes do not execute in static web distributions
 *   (e.g., GitHub Pages). Client surfaces fall back gracefully to in-browser AST evaluation or Tauri IPC.
 */

import { NextRequest, NextResponse } from "next/server";
import { callOscalCli } from "@/lib/mcp-client";

export async function POST(req: NextRequest) {
  try {
    const body = await req.json();
    const { command, args = [] } = body;

    if (!command || typeof command !== "string") {
      return NextResponse.json(
        {
          success: false,
          transport: "http-cli-bridge",
          error: "Missing required parameter 'command'",
        },
        { status: 400 },
      );
    }

    const output = await callOscalCli([command, ...args, "--format", "json"]);
    let parsed: unknown;
    try {
      parsed = JSON.parse(output);
    } catch {
      parsed = { raw: output };
    }
    return NextResponse.json({
      success: true,
      transport: "http-cli-bridge",
      data: parsed,
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    return NextResponse.json(
      {
        success: false,
        transport: "http-cli-bridge",
        error: message,
      },
      { status: 500 },
    );
  }
}

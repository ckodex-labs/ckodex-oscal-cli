import { NextRequest, NextResponse } from "next/server";
import { callOscalCli } from "@/lib/mcp-client";

export async function POST(req: NextRequest) {
  try {
    const body = await req.json();
    const { command, args = [] } = body;

    const output = await callOscalCli([command, ...args, "--format", "json"]);
    let parsed: unknown;
    try {
      parsed = JSON.parse(output);
    } catch {
      parsed = { raw: output };
    }
    return NextResponse.json({ success: true, data: parsed });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    return NextResponse.json(
      { success: false, error: message },
      { status: 500 },
    );
  }
}

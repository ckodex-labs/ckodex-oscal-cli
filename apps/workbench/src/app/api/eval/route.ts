import { NextRequest, NextResponse } from "next/server";
import { callOscalCli } from "@/lib/mcp-client";
import fs from "fs";
import os from "os";
import path from "path";

export async function POST(req: NextRequest) {
  let tempFilePath: string | null = null;
  try {
    const body = await req.json();
    const { rule, payload } = body;

    if (!rule || !payload) {
      return NextResponse.json(
        { success: false, error: "Missing required 'rule' or 'payload'" },
        { status: 400 },
      );
    }

    const tempDir = os.tmpdir();
    tempFilePath = path.join(tempDir, `mizan-eval-${Date.now()}-${Math.random().toString(36).slice(2)}.json`);
    fs.writeFileSync(tempFilePath, payload, "utf-8");

    const output = await callOscalCli([
      "policy",
      "rulepack",
      "eval",
      "-r",
      rule,
      "-i",
      tempFilePath,
      "--format",
      "json",
    ]);

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
  } finally {
    if (tempFilePath && fs.existsSync(tempFilePath)) {
      try {
        fs.unlinkSync(tempFilePath);
      } catch {
        // ignore cleanup error
      }
    }
  }
}

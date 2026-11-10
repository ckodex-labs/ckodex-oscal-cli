/**
 * Shared request guards for the local dev bridges (/api/cli, /api/eval).
 *
 * These routes spawn the local `mizan` binary, so they must not be reachable
 * from other origins while `next dev` is running (drive-by CSRF).
 *
 * - Content-Type must be application/json. Cross-origin JSON POSTs require a
 *   CORS preflight, which these routes never approve.
 * - When a browser sends an Origin header it must match the Host header.
 */

import { NextRequest, NextResponse } from "next/server";

export const MAX_BODY_BYTES = 1_048_576;

export function bridgeError(status: number, error: string) {
  return NextResponse.json(
    { success: false, transport: "http-cli-bridge", error },
    { status },
  );
}

/** Returns an error response if the request must be rejected, else null. */
export function guardBridgeRequest(req: NextRequest): NextResponse | null {
  const origin = req.headers.get("origin");
  if (origin) {
    let ok = false;
    try {
      ok = new URL(origin).host === req.headers.get("host");
    } catch {
      ok = false;
    }
    if (!ok) return bridgeError(403, "Cross-origin requests are not accepted");
  }
  const ct = req.headers.get("content-type") ?? "";
  if (!ct.startsWith("application/json")) {
    return bridgeError(415, "Content-Type must be application/json");
  }
  const contentLength = req.headers.get("content-length");
  if (contentLength !== null) {
    const parsedLen = Number(contentLength);
    if (Number.isFinite(parsedLen) && parsedLen > MAX_BODY_BYTES) {
      return bridgeError(
        413,
        `Request body exceeds ${MAX_BODY_BYTES} byte limit`,
      );
    }
  }
  return null;
}

export async function readBoundedJsonBody(
  req: NextRequest,
  maxBytes = MAX_BODY_BYTES,
): Promise<
  | { ok: true; value: unknown }
  | { ok: false; response: NextResponse }
> {
  const contentLength = req.headers.get("content-length");
  if (contentLength !== null) {
    const parsedLen = Number(contentLength);
    if (Number.isFinite(parsedLen) && parsedLen > maxBytes) {
      return {
        ok: false,
        response: bridgeError(413, `Request body exceeds ${maxBytes} byte limit`),
      };
    }
  }

  let rawText: string;
  if (req.body) {
    const reader = req.body.getReader();
    const chunks: Uint8Array[] = [];
    let total = 0;
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        if (value) {
          total += value.byteLength;
          if (total > maxBytes) {
            await reader.cancel().catch(() => {});
            return {
              ok: false,
              response: bridgeError(
                413,
                `Request body exceeds ${maxBytes} byte limit`,
              ),
            };
          }
          chunks.push(value);
        }
      }
    } finally {
      reader.releaseLock();
    }
    rawText = Buffer.concat(chunks).toString("utf8");
  } else {
    rawText = await req.text();
    if (Buffer.byteLength(rawText, "utf8") > maxBytes) {
      return {
        ok: false,
        response: bridgeError(413, `Request body exceeds ${maxBytes} byte limit`),
      };
    }
  }

  try {
    return { ok: true, value: JSON.parse(rawText) };
  } catch {
    return { ok: false, response: bridgeError(400, "Body is not valid JSON") };
  }
}

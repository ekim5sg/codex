import { Ai } from "@cloudflare/ai";

interface Env {
  AI: any; // Cloudflare AI binding
}

interface DailyWonderCard {
  fun_fact: string;
  thought: string;
  kindness: string;
}

const fallbackCard: DailyWonderCard = {
  fun_fact: "Honeybees can recognize human faces, remembering patterns the way we do.",
  thought: "Small curiosities can become big discoveries when we keep asking why.",
  kindness: "Send a short thank-you note to someone who helped you recently.",
};

function json(data: unknown, status = 200, origin?: string, cacheSeconds = 0) {
  const corsOrigin = origin ?? "*";

  const headers: Record<string, string> = {
    "content-type": "application/json; charset=utf-8",
    "access-control-allow-origin": corsOrigin,
    "access-control-allow-methods": "GET,OPTIONS",
    "access-control-allow-headers": "content-type",
  };

  if (cacheSeconds > 0) {
    headers["cache-control"] = `public, max-age=${cacheSeconds}`;
    headers["cf-cache-status"] = "dynamic";
  } else {
    headers["cache-control"] = "no-store";
  }

  return new Response(JSON.stringify(data), { status, headers });
}

function extractJson(text: string): DailyWonderCard | null {
  try {
    return JSON.parse(text) as DailyWonderCard;
  } catch {
    const match = text.match(/\{[\s\S]*\}/);
    if (!match) return null;
    try {
      return JSON.parse(match[0]) as DailyWonderCard;
    } catch {
      return null;
    }
  }
}

function looksValid(card: any): card is DailyWonderCard {
  return (
    card &&
    typeof card.fun_fact === "string" &&
    typeof card.thought === "string" &&
    typeof card.kindness === "string"
  );
}

/**
 * Pick a stable “day key”.
 * If you prefer America/Chicago midnight instead of UTC, set TZ_OFFSET_MIN = -360 for CST
 * (ignores DST). Keeping UTC is simplest & consistent.
 */
function dayKeyUTC(date = new Date()): string {
  const yyyy = date.getUTCFullYear();
  const mm = String(date.getUTCMonth() + 1).padStart(2, "0");
  const dd = String(date.getUTCDate()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}`;
}

/**
 * Stable-ish seed from string (simple hash) so daily content can vary
 * while still being deterministic for the day.
 */
function hashStr(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

async function generateCard(env: Env, dayKey: string): Promise<DailyWonderCard> {
  const ai = new Ai(env.AI);

  const response = await ai.run("@cf/meta/llama-3.1-8b-instruct", {
    messages: [
      {
        role: "system",
        content:
          "Return ONLY valid JSON with keys fun_fact, thought, kindness. Keep each value under 25 words. No markdown. No extra keys.",
      },
      {
        role: "user",
        content:
          `Create the Daily Wonder Card for ${dayKey}. Make it kid-friendly, upbeat, and not repetitive.`,
      },
    ],
    max_tokens: 200,
  });

  if (typeof response === "string") {
    const parsed = extractJson(response);
    if (parsed && looksValid(parsed)) return parsed;
  } else if (response && typeof response === "object") {
    const maybe = (response as any).response;
    if (typeof maybe === "string") {
      const parsed = extractJson(maybe);
      if (parsed && looksValid(parsed)) return parsed;
    }
    if (looksValid(response)) return response as DailyWonderCard;
  }

  return fallbackCard;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const origin = request.headers.get("Origin") ?? undefined;

    // CORS preflight
    if (request.method === "OPTIONS") {
      return json({ ok: true }, 204, origin, 0);
    }

    if (request.method !== "GET") {
      return json({ error: "Method not allowed" }, 405, origin, 0);
    }

    if (url.pathname === "/") {
      return json({ ok: true, routes: ["/card"] }, 200, origin, 0);
    }

    if (url.pathname !== "/card") {
      return json({ error: "Not found" }, 404, origin, 0);
    }

    // ✅ True Daily: stable per day
    const dk = dayKeyUTC();

    // ✅ Edge-cache key (cache per day)
    // Cache at the edge for 24 hours to minimize AI calls/cost.
    const cache = caches.default;
    const cacheUrl = new URL(url.toString());
    cacheUrl.pathname = "/__daily_card__";
    cacheUrl.search = `?day=${encodeURIComponent(dk)}`;

    const cacheKey = new Request(cacheUrl.toString(), request);
    const cached = await cache.match(cacheKey);
    if (cached) return cached;

    // Generate once (per edge) and cache
    let card: DailyWonderCard = fallbackCard;
    try {
      card = await generateCard(env, dk);
    } catch {
      card = fallbackCard;
    }

    const resp = json(
      {
        day: dk,
        ...card,
      },
      200,
      origin,
      86400 // 24 hours
    );

    // Tell Cloudflare to cache this response
    await cache.put(cacheKey, resp.clone());
    return resp;
  },
};
import { Ai } from "@cloudflare/ai";

interface Env {
  AI: any; // Cloudflare AI binding
}

interface DailyWonderCard {
  fun_fact: string;
  thought: string;
  kindness: string;
}

interface Meta {
  mode: "daily" | "surprise";
  day_utc: string;            // YYYY-MM-DD (UTC)
  next_reset_utc_ms: number;  // epoch ms
  ttl_seconds: number;        // seconds until next UTC midnight
}

interface CardResponse {
  card: DailyWonderCard;
  meta: Meta;
}

const fallbackCard: DailyWonderCard = {
  fun_fact: "Honeybees can recognize human faces, remembering patterns the way we do.",
  thought: "Small curiosities can become big discoveries when we keep asking why.",
  kindness: "Send a short thank-you note to someone who helped you recently.",
};

// Always allow all origins (prevents cache fragmentation & CORS mismatch issues)
const CORS_ORIGIN = "*";

function baseHeaders() {
  return {
    "content-type": "application/json; charset=utf-8",
    "access-control-allow-origin": CORS_ORIGIN,
    "access-control-allow-methods": "GET,OPTIONS",
    "access-control-allow-headers": "content-type",
  } as Record<string, string>;
}

/**
 * cacheSeconds:
 *  - 0 => no-store
 *  - >0 => cacheable; we use s-maxage so edge caching aligns with TTL.
 */
function json(data: unknown, status = 200, cacheSeconds = 0) {
  const headers = baseHeaders();

  if (cacheSeconds > 0) {
    // Browser can revalidate; edge holds it for ttl.
    headers["cache-control"] = `public, max-age=0, s-maxage=${cacheSeconds}`;
	headers["cf-cache-control"] = `max-age=${cacheSeconds}`;
	headers["vary"] = "Origin";
  } else {
    headers["cache-control"] = "no-store";
	headers["vary"] = "Origin";
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

/** YYYY-MM-DD in UTC */
function dayKeyUTC(date = new Date()): string {
  const yyyy = date.getUTCFullYear();
  const mm = String(date.getUTCMonth() + 1).padStart(2, "0");
  const dd = String(date.getUTCDate()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}`;
}

/** Next UTC midnight for a given Date */
function nextUtcMidnightMs(date = new Date()): number {
  return Date.UTC(
    date.getUTCFullYear(),
    date.getUTCMonth(),
    date.getUTCDate() + 1,
    0, 0, 0, 0
  );
}

/** Seconds until next UTC midnight (min 1 to avoid 0-second caching edge cases) */
function ttlUntilNextUtcMidnightSeconds(date = new Date()): number {
  const now = date.getTime();
  const next = nextUtcMidnightMs(date);
  return Math.max(1, Math.floor((next - now) / 1000));
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

function makeResponse(mode: "daily" | "surprise", dk: string, ttl: number, card: DailyWonderCard): CardResponse {
  const now = new Date();
  return {
    card,
    meta: {
      mode,
      day_utc: dk,
      next_reset_utc_ms: nextUtcMidnightMs(now),
      ttl_seconds: ttl,
    },
  };
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // CORS preflight
    if (request.method === "OPTIONS") {
      return json({ ok: true }, 204, 0);
    }

    if (request.method !== "GET") {
      return json({ error: "Method not allowed" }, 405, 0);
    }

    // Routes
    if (url.pathname === "/") {
      return json({ ok: true, routes: ["/card", "/surprise"] }, 200, 0);
    }

    const dk = dayKeyUTC();
    const ttl = ttlUntilNextUtcMidnightSeconds();

    // ---------- DAILY (cached until next UTC midnight) ----------
    if (url.pathname === "/card") {
      const cache = caches.default;

      // Cache key is stable per day
      const cacheUrl = new URL(url.toString());
      cacheUrl.pathname = "/__daily_card__";
      cacheUrl.search = `?day=${encodeURIComponent(dk)}`;

      const cacheKey = new Request(cacheUrl.toString(), { method: "GET" });
      const cached = await cache.match(cacheKey);
      if (cached) return cached;

      let card: DailyWonderCard = fallbackCard;
      try {
        card = await generateCard(env, dk);
      } catch {
        card = fallbackCard;
      }

      const payload = makeResponse("daily", dk, ttl, card);
      const resp = json(payload, 200, ttl);

      // Cache at edge; TTL comes from Cache-Control s-maxage
      await cache.put(cacheKey, resp.clone());
      return resp;
    }

    // ---------- SURPRISE (never cached) ----------
    if (url.pathname === "/surprise") {
      // Use dk in prompt seed just for “today’s vibe”, but DO NOT cache result.
      let card: DailyWonderCard = fallbackCard;
      try {
        // You can vary the prompt more if you want, but simplest: reuse generator.
        card = await generateCard(env, dk);
      } catch {
        card = fallbackCard;
      }

      const payload = makeResponse("surprise", dk, ttl, card);
      return json(payload, 200, 0); // no-store
    }

    return json({ error: "Not found" }, 404, 0);
  },
};
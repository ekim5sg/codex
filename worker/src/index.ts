import { Ai } from "@cloudflare/ai";

interface Env {
  AI: Ai;
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

function extractJson(text: string): DailyWonderCard | null {
  try {
    return JSON.parse(text) as DailyWonderCard;
  } catch (error) {
    const match = text.match(/\{[\s\S]*\}/);
    if (!match) {
      return null;
    }
    try {
      return JSON.parse(match[0]) as DailyWonderCard;
    } catch (innerError) {
      return null;
    }
  }
}

async function generateCard(env: Env): Promise<DailyWonderCard> {
  const response = await env.AI.run("@cf/meta/llama-3.1-8b-instruct", {
    messages: [
      {
        role: "system",
        content:
          "You are a gentle creativity assistant. Return ONLY valid JSON with keys fun_fact, thought, kindness. Keep each value under 25 words.",
      },
      {
        role: "user",
        content: "Create a Daily Wonder Card.",
      },
    ],
    max_tokens: 200,
  });

  const raw = typeof response === "string" ? response : (response as { response?: string }).response;
  if (raw) {
    const parsed = extractJson(raw);
    if (parsed) {
      return parsed;
    }
  }

  return fallbackCard;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname !== "/card") {
      return new Response("Not found", { status: 404 });
    }

    try {
      const card = await generateCard(env);
      return Response.json(card, {
        headers: {
          "Cache-Control": "no-store",
          "Access-Control-Allow-Origin": "*",
        },
      });
    } catch (error) {
      return Response.json(fallbackCard, {
        headers: {
          "Cache-Control": "no-store",
          "Access-Control-Allow-Origin": "*",
        },
      });
    }
  },
};

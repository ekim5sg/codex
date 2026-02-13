# Daily Wonder Card

This repo contains a Rust Yew WASM front-end and a Cloudflare Worker that uses Workers AI to generate a Daily Wonder Card.

## Front-end (Yew)

```bash
cd app
trunk serve
```

The UI calls the worker at `/card`. When developing locally, proxy or deploy the worker to keep the same path.

## Cloudflare Worker

```bash
cd worker
npm install
npm run dev
```

The worker responds to `GET /card` with JSON:

```json
{
  "fun_fact": "...",
  "thought": "...",
  "kindness": "..."
}
```

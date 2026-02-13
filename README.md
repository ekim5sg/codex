# Orbit of Ideas

Orbit of Ideas is a client-side Rust + Yew + WASM app where ideas orbit a central mission.

## Run locally

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve
```

Open `http://127.0.0.1:8080`.

## Build

```bash
trunk build --release
```

## Checks

```bash
cargo fmt
trunk build
```

## Features

- Animated SVG orbit simulator with requestAnimationFrame.
- Mission-centered rings and moving idea satellites.
- Live controls for impact, urgency, effort, and simulator gains.
- Dependency links between ideas with collision highlighting.
- Auto-stabilize tuner to rebalance orbit spacing.
- JSON export/import for idea/link/simulator bundles.
- Scenario snapshots (saved to localStorage) with A/B comparison deltas.
- Add/delete ideas locally, with selection + telemetry panel.
- Mobile-friendly responsive layout and dark studio styling.

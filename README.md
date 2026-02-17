# Mobile Directory Zipper (Rust + WASM)

A Rust/WASM app that runs in mobile browsers (including iPhone Chrome) and zips a selected folder into a clean archive.

## Why this works for your use case

- Uses a browser file picker with `webkitdirectory` to select a directory exposed by apps like Koder on iOS.
- Reads files in-browser and builds the ZIP in WASM.
- Automatically strips one redundant top-level folder when all selected files share the same root, so you avoid extra nesting inside the resulting ZIP.

## Run locally

1. Install tools:
   ```bash
   rustup target add wasm32-unknown-unknown
   cargo install trunk wasm-bindgen-cli
   ```
2. Start dev server:
   ```bash
   trunk serve --open
   ```
3. Build production bundle:
   ```bash
   trunk build --release
   ```

## iPhone / Koder flow

1. Open the hosted app in Chrome on iPhone.
2. Tap **Folder** and select the directory exposed by Koder.
3. Tap **Create ZIP**.
4. Save/share the generated `archive.zip` file.


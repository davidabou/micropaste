# Micropaste

A visual clipboard library for **Windows 11**: everything that gets copied (text, links, images, screenshots, colors, code, files) is kept, previewed as rich cards, organized by type, by source app and into custom categories, then reused in one gesture (paste, drag and drop, shortcut).

It is **not** just a Win+V replacement: Win+V is only one entry point among others to the **Shelf** and the **Library**.
Local-first, fast, with a premium, native Windows 11 design (inspired by Supaste on Mac, without copying its visuals).

## How to work

- The app UI is in **English**.
- **Do not launch the app** (`bun run tauri dev`), nor screenshot or monitor it: the user tests it themself. Stick to static checks:
  - `bun run typecheck`;
  - in `src-tauri/`: `cargo fmt` and `cargo clippy --all-targets -- -D warnings`.
- Package manager: **Bun** only (no pnpm or npm).
- The user often edits files themself: re-read before editing, and keep their changes.
- Look at the code directly instead of guessing.
- CI (`.github/workflows/ci.yml`) fails on type errors, unformatted Rust or any clippy warning.

## Stack

- **Tauri 2** (Rust, `src-tauri/`, crate `tauri-app`, lib `tauri_app_lib`).
  - The identifier `com.abou.micropaste` must **not** change: the data lives in `%APPDATA%\com.abou.micropaste`.
  - Build: `bun run tauri build`, NSIS installer in `src-tauri/target/release/bundle/nsis/`.
  - Release: bump `version` in `tauri.conf.json`, then push a matching tag (`v0.2.0`): `release.yml` builds the installer into a draft GitHub release.
- **Nuxt 4** (`app/`, static output via `nuxt generate`), **Tailwind v4**.
- **Reka UI** (`reka-ui/nuxt`) for Dialog, ContextMenu, Tooltip, AlertDialog…
- **motion-v** (`motion-v/nuxt`) for all animations (springs).
- **@nuxt/icon** with the `gravity-ui` collection. Any new icon requires a full restart of `tauri dev`.

## Architecture

### Rust (`src-tauri/src/`)

| File | Role |
| --- | --- |
| `lib.rs` | Setup, Tauri commands |
| `clipboard.rs` | Clipboard watching, saving copies |
| `copy.rs` | Copying a clip back; `is_own_copy` ignores copies made from tiles |
| `db.rs` | SQLite, migrations via `user_version` (bump it for any schema change) |
| `history.rs` | Cleanup: unused for 30 days, except favorites and clips in collections; at startup then hourly, emits `clips-removed` |
| `images.rs` | Images stored as PNG by hash in `$APPDATA/images` |
| `drag_out.rs` | Native drag of images and files to other apps |
| `drop.rs` | External drop into a collection (files via WebView2 `postMessageWithAdditionalObjects`, text, images) |
| `source_app.rs` | Source app (foreground app) |
| `link_preview.rs` | Link previews |
| `background.rs` | Tray icon, close = hide, autostart, single-instance, `--hidden` |
| `snap_layouts.rs` | Hovering the Maximize button: Win+Z via SendInput |
| `window_icon.rs`, `app_icon.rs` | Sharp icon at the right DPI |

### Frontend (`app/`)

- `app.vue`: tabs, History grid always mounted, collection grids, search, favorites filter, grid/list layout, external drop.
- `components/`: `ClipCard` (tile), `ClipGrid`, `ClipContextMenu`, `ClipDetails` (side panel), `CollectionTabs`, `Tooltip`, `TitleBar`, `Button`, `Input`.
- `composables/`: `useClips`, `useCollections`, `useTileDrag`, link and file previews.
- `utils/`: colors, SVG, links, files, `menu.ts` (shared menu animation).

## Product rules

- **History** = real copies (Ctrl+C). Content **dropped** from another app only goes into a collection (`in_history = 0`), never into the history, until it is actually copied.
- External drop is only available in collections.
- Copies made from the window have "Micropaste" as their source.
- No "restore" detection: the app mirrors the real clipboard.

## Known pitfalls

- `backdrop-filter` doesn't blur text in this (transparent) window: blur the content itself (`blur-sm`) or use `mask-image`.
- Motion stops the animations of an unmounted element: for exits, use `<Transition>` + `animate()` on a non-Motion div, or manual presence. `AnimatePresence` `custom` doesn't work for the exit direction.
- Performance: lazily mount anything per tile (tooltip and menu portals, corner buttons on hover).
- Bash heredocs mangle backslashes: use the Edit tool for code containing them. Python on Windows: write with `newline=''` (CRLF).

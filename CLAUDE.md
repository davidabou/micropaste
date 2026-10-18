# Micropaste

A visual clipboard library for **Windows 11**: everything that gets copied (text, links, images, screenshots, colors, code, files) is kept, previewed as rich cards, organized by type, by source app and into custom categories, then reused in one gesture (paste, drag and drop, shortcut).

It is **not** just a Win+V replacement: Win+V is only one entry point among others to the **Shelf** and the **Library**.
Local-first, fast, with a premium, native Windows 11 design (inspired by Supaste on Mac, without copying its visuals).

## How to work

- The app UI is in **English**.
- **Do not launch the app** (`bun run tauri dev`), nor screenshot or monitor it: the user tests it themself. Stick to static checks:
  - `bun run lint` (`lint:fix` to fix), `bun run typecheck` and `bun run test` (Vitest, `test/`);
  - in `src-tauri/`: `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` (unit tests at the bottom of each file, the database ones on SQLite in memory).
- Package manager: **Bun** only (no pnpm or npm).
- The user often edits files themself: re-read before editing, and keep their changes.
- Look at the code directly instead of guessing.
- CI: `.github/workflows/app.yml` (the app) fails on any lint, type, test, rustfmt or clippy error, on a `cargo deny` issue (advisories, licenses: `src-tauri/deny.toml`). `site.yml` lints and type checks the showcase site (Vercel builds it); each ignores the other one's files. `release.yml` runs `app.yml` before building.
- Git hooks (simple-git-hooks): ESLint on staged files (nano-staged), and conventional commit messages (commitlint), which the release notes rely on.
- Commits that only touch the showcase site (`site/`) never use `feat:`, `fix:` or `perf:`: use `chore(site):`, `docs(site):` or `style(site):`, so they stay out of the app's release notes.

## Stack

- **Tauri 2** (Rust, `src-tauri/`, crate `Micropaste`, lib `tauri_app_lib`).
  - The identifier `com.abou.micropaste` must **not** change: the data lives in `%APPDATA%\com.abou.micropaste`.
  - Build: `bun run tauri build`, NSIS installer in `src-tauri/target/release/bundle/nsis/`.
  - Release: `bun run release` (bumpp, see `bump.config.ts`) asks for the new version, writes it in `package.json`, `tauri.conf.json`, `Cargo.toml` and `Cargo.lock`, then commits and tags it (`v0.2.0`). It refuses to run with uncommitted changes. Then `git push --follow-tags`: `release.yml` builds the installer into a draft GitHub release, with release notes written by changelogithub from the `feat:` and `fix:` commits since the previous tag.
- **Nuxt 4** (`app/`, static output via `nuxt generate`), **Tailwind v4**.
- **Reka UI** (`reka-ui/nuxt`) for Dialog, ContextMenu, Tooltip, AlertDialog…
- **motion-v** (`motion-v/nuxt`) for all animations (springs).
- **VueUse** (`@vueuse/nuxt`, auto-imported): prefer its composables over hand-written listeners, timers and observers.
- **@nuxt/icon** with the `gravity-ui` collection. Any new icon requires a full restart of `tauri dev`.

## Architecture

### Rust (`src-tauri/src/`)

| File | Role |
| --- | --- |
| `lib.rs` | Setup, Tauri commands |
| `clipboard.rs` | Clipboard watching, saving copies |
| `copy.rs` | Copying a clip back, or a single file of a group; `is_own_copy` ignores copies made from the window; `clip_file` only accepts paths of the clip |
| `db.rs` | SQLite; for any schema change, append a migration to the end of `MIGRATIONS` (never edit or reorder existing ones: `user_version` is how many have run) |
| `history.rs` | Cleanup: unused for the retention set in the settings (30 days by default), except favorites and clips in collections; at startup then hourly, emits `clips-removed` |
| `images.rs` | Images stored as PNG by hash in `$APPDATA/images` |
| `drag_out.rs` | Native drag of images and files to other apps |
| `drop.rs` | External drop into a collection (files via WebView2 `postMessageWithAdditionalObjects`, text, images) |
| `source_app.rs` | Source app (foreground app) |
| `link_preview.rs` | Link previews |
| `settings.rs` | Settings stored in the `settings` table: global shortcut, history retention, launch at startup, grid/list layout |
| `background.rs` | Tray icon, close = hide, autostart, single-instance, `--hidden`, global shortcut (Alt+Shift+V by default), window placed in the bottom right corner at launch |
| `window_icon.rs` | Sharp window icon at the right DPI |
| `app_icon.rs` | Source app icons, file thumbnails and file type names as the Explorer shows them |

### Frontend (`app/`)

- `app.vue`: tabs, History grid always mounted, collection grids, search, favorites filter, grid/list layout, external drop.
- `components/`: `ClipCard` (tile), `ClipGrid`, `ClipContextMenu`, `ClipDetails` (side panel), `SettingsPanel`, `ClipImage`, `AppIcon`, `CollectionTabs`, `TabPill`, `Tooltip`, `TitleBar`, `Button`, `Input`.
- `composables/`: `useClips`, `useCollections`, `useTileDrag`, `useClipNotice` (shows a notice on a tile from anywhere), link and file previews, `usePrefersDark` (VueUse's `usePreferredDark`, shared by every tile).
- `assets/css/main.css`: global styles, including the `.skeleton` class for content still loading.
- `utils/`: colors, SVG, links, files, bytes, `timeAgo.ts` (compact format for VueUse's `useTimeAgo`), `menu.ts` (shared menu animation).

## Product rules

- **History** = real copies (Ctrl+C). Content **dropped** from another app only goes into a collection (`in_history = 0`), never into the history, until it is actually copied.
- External drop is only available in collections.
- Copies made from the window have "Micropaste" as their source.
- Copying a single file of a group creates (or brings back) a clip of that file alone at the top of the history; the group itself never changes.
- No "restore" detection: the app mirrors the real clipboard.

## Known pitfalls

- `backdrop-filter` doesn't blur text in this (transparent) window: blur the content itself (`blur-sm`) or use `mask-image`.
- Motion stops the animations of an unmounted element: for exits, use `<Transition>` + `animate()` on a non-Motion div, or manual presence. `AnimatePresence` `custom` doesn't work for the exit direction.
- Performance: lazily mount anything per tile (tooltip and menu portals, corner buttons on hover).
- Windows sometimes fails to make a file thumbnail: the file preview cache only keeps previews that worked.
- File type names come in the Windows language while the UI is in English: accepted until the app is translated.
- The taskbar and window icon is `icons/icon.ico` embedded in the exe by `tauri-build`, which doesn't watch it: after changing it, run `cargo clean -p Micropaste` in `src-tauri/`, or `tauri dev` keeps the old one.
- Bash heredocs mangle backslashes: use the Edit tool for code containing them. Python on Windows: write with `newline=''` (CRLF).

# Hoverdo

Floating-widget-first notes & to-do app for Windows. The main window is the
library; the day-to-day workflow happens in small, always-available widgets
pinned to the desktop with Mica/Acrylic translucency.

## Status

Single-user, offline, Windows-targeted. Schema and HLC are sync-ready from
day one but no transport is shipped yet.

## What's in MVP

- **Notes**: title + body, debounced auto-save (500 ms), soft-delete with
  cross-window broadcast.
- **To-do lists**: rename in place, add/check/delete tasks, tasks ordered by
  monotonic sort key (fractional indexing slot reserved for drag-reorder).
- **Floating widgets**: pin a note as a sticky window or a list as a to-do
  window. Per-widget opacity slider, always-on-top toggle, geometry
  persisted across restarts. Closing a widget unpins it.
- **System tray**: left-click toggles main window, menu offers explicit
  show / show all widgets / hide all widgets / quit. Main window's X is
  rewired to hide-to-tray.
- **Reminders**: per-task, with OS notifications fired by a 20-second
  polling scheduler. Quick presets (15 m / 1 h / 4 h / Tomorrow 9 am) plus
  a native datetime-local picker.
- **Search**: FTS5-backed full-text across notes, lists and tasks.
  Ctrl/Cmd+F focuses the title-bar search; results live in a popover with
  inline-highlighted snippets.
- **Theme**: Mica Light + Mica Dark presets (CSS-variable token contract).
  Auto-follows the Windows theme, manual override via the title-bar toggle.

Post-MVP (architecturally prepared, not shipped):

- Markdown rendering on notes (`notes.is_markdown` flag slot reserved)
- Tags (schema + repo ready; no UI yet)
- Global quick-capture hotkey
- Multi-device sync via `SyncProvider` trait + `change_log` table
- Custom theme editor / more presets
- Kanban widget (extends `widget_instances.kind`)

## Stack

| Layer              | Choice                                          |
| ------------------ | ----------------------------------------------- |
| Shell              | **Tauri 2** (Rust core)                         |
| Frontend           | **Svelte 5** (runes) + Vite + TypeScript        |
| Styling            | **Tailwind CSS 4** + CSS custom properties      |
| UI primitives      | bits-ui (Svelte port of Radix) + lucide icons   |
| Storage            | **SQLite** via `sqlx` (WAL, foreign keys on)    |
| ID & clock         | UUID v7 + Hybrid Logical Clock (38-char hex)    |
| Mica/Acrylic       | `window-vibrancy` (Tauri plugin)                |
| Notifications      | `tauri-plugin-notification` (OS-native toasts)  |

## Repo layout

```
hoverdo/
├── src/                       Svelte frontend
│   ├── main/                  Main library window entry
│   ├── sticky/                Sticky-note widget window entry
│   ├── todo/                  To-do list widget window entry
│   └── lib/
│       ├── actions/           Svelte actions (e.g. clickOutside)
│       ├── components/        Reusable UI (Sidebar, NoteList, TaskBoard…)
│       ├── ipc/               Typed wrappers over invoke()/listen()
│       ├── stores/            Svelte 5 rune stores (notes / lists / nav)
│       └── theme/             Token contract + light/dark presets
├── src-tauri/                 Rust shell
│   ├── src/
│   │   ├── main.rs            Bootstrap
│   │   ├── lib.rs             Tauri builder, command registry
│   │   ├── clock.rs           HLC implementation + tests
│   │   ├── models.rs          Domain structs
│   │   ├── error.rs           HoverdoError + Result alias
│   │   ├── theme.rs           Mica/Acrylic application
│   │   ├── tray.rs            System tray menu + close-to-tray
│   │   ├── widgets.rs         Widget window spawn/restore
│   │   ├── reminders.rs       Background notification scheduler
│   │   ├── sync.rs            SyncProvider trait stub (post-MVP)
│   │   ├── commands/          IPC: notes, lists, widgets, reminders, search
│   │   └── db/
│   │       ├── migrations/    SQL migrations
│   │       └── repos/         One module per domain (no SQL outside)
│   └── capabilities/          Window permissions
├── index.html | sticky.html | todo.html   Vite multi-window entries
└── .claude/                   Session-start hook for Claude Code on the web
```

## Quickstart

Prerequisites (Windows):

1. **Node 20+** and **pnpm 10+** — `winget install OpenJS.NodeJS.LTS && npm i -g pnpm`
2. **Rust stable** — `winget install Rustlang.Rustup && rustup default stable`
3. **MSVC build tools** — `winget install Microsoft.VisualStudio.2022.BuildTools`,
   tick "Desktop development with C++" in the installer
4. **WebView2** — preinstalled on Windows 11; auto-installs with Edge on 10

Then:

```bash
git clone <your fork>
cd hoverdo
git checkout claude/hoverdo-floating-widget-Bt3Bj
pnpm install
pnpm tauri:dev
```

First Rust build takes 5-15 min cold; subsequent builds are incremental.

## Scripts

| Command            | Effect                                  |
| ------------------ | --------------------------------------- |
| `pnpm dev`         | Frontend-only (Vite, no shell)          |
| `pnpm tauri:dev`   | Full app in dev mode                    |
| `pnpm tauri:build` | Production bundle (`src-tauri/target/`) |
| `pnpm check`       | `svelte-check` across the TS+Svelte tree |
| `pnpm fmt`         | Prettier                                |
| `cargo test`       | Rust unit + integration tests           |
| `cargo clippy`     | Rust linter                             |

## Keyboard shortcuts

| Key                | Effect (main window)                      |
| ------------------ | ----------------------------------------- |
| Ctrl/Cmd+N         | New note (Notes section) / new list (Lists section) |
| Ctrl/Cmd+F         | Focus global search                       |
| Esc                | Clear/blur search; close open popover     |

## Architecture notes

**HLC** — every mutation is tagged with a Hybrid Logical Clock (38-char
lex-sortable hex). Even though MVP is single-device, having the clock in
place from day one means adding a sync transport later is non-breaking.

**change_log** — every domain mutation appends an entry inside the same
SQLx transaction as the data write. Today it's used only by tests as an
audit trail; tomorrow it's the canonical source for `SyncProvider::push`.

**FTS5** — `notes_fts` / `lists_fts` / `tasks_fts` virtual tables are kept
in sync with their source via AFTER INSERT/UPDATE/DELETE triggers that
respect `deleted_at` (soft-deleted rows never show up in search results).

**Multi-window** — each pinned widget is a separate Tauri window with
label `<kind>-<uuid>`. Capabilities use the `sticky-*` / `todo-*` glob
patterns, so future widget kinds opt in by naming convention.

**Cross-window sync** — every mutating IPC command emits a Tauri event
(`notes:changed`, `lists:changed`, `tasks:changed`, `reminders:changed`).
Stores subscribe with echo-suppression so the caller doesn't refetch the
same data they just merged optimistically.

## Data location

Application data lives in `%APPDATA%/dev.hoverdo.app/hoverdo.db`. You can
open it with any SQLite browser to inspect schema, change_log, or back up.

## License

Private (pet project).

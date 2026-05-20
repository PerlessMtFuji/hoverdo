# Hoverdo

Floating-widget-first notes & todos. Windows-only for now, single-user, offline.

## Stack

- **Shell**: [Tauri 2](https://tauri.app) (Rust)
- **Frontend**: Svelte 5 (runes) + Vite + TypeScript
- **Styling**: Tailwind CSS 4 + CSS custom properties for theming
- **Storage**: SQLite via `sqlx` (lands in next commit)
- **OS effects**: Mica/Acrylic via `window-vibrancy`

## Project layout

```
hoverdo/
├── src/                      Svelte frontend
│   ├── main/                 Main library window
│   ├── sticky/               Sticky-note widget window
│   ├── todo/                 To-do list widget window
│   └── lib/
│       └── theme/            Token contract, light/dark, runtime
├── src-tauri/                Rust shell
│   ├── src/
│   │   ├── main.rs           Bootstrap
│   │   ├── lib.rs            Tauri builder, plugin wiring
│   │   └── theme.rs          Native Mica/Acrylic application
│   ├── capabilities/         Window permissions
│   ├── icons/                Bundle icons (placeholder set; replace before ship)
│   ├── tauri.conf.json
│   └── Cargo.toml
├── index.html | sticky.html | todo.html   Vite multi-window entries
├── vite.config.ts
├── tsconfig.json
├── svelte.config.js
└── package.json
```

## Quickstart

Prerequisites: Node ≥ 20, pnpm ≥ 10, Rust stable. On Windows, install
[Tauri prerequisites](https://tauri.app/start/prerequisites/) (WebView2 is
preinstalled on Windows 11).

```bash
pnpm install
pnpm tauri dev
```

The first `tauri dev` will compile the Rust shell - allow a few minutes for
cold builds. Subsequent runs are incremental.

## Scripts

| Command            | What it does                              |
| ------------------ | ----------------------------------------- |
| `pnpm dev`         | Frontend-only (Vite) without the shell    |
| `pnpm tauri:dev`   | Full app in dev mode                      |
| `pnpm tauri:build` | Production bundle (`src-tauri/target/`)   |
| `pnpm check`       | `svelte-check` over the TS+Svelte tree    |
| `pnpm fmt`         | Format with Prettier                      |

## MVP roadmap

See `/root/.claude/plans/rozpocznijmy-prac-nad-moim-velvety-snail.md` for the
detailed plan. High-level commit sequence on
`claude/hoverdo-floating-widget-Bt3Bj`:

1. Scaffold (this commit)
2. DB + migrations + HLC + repos
3. Theme system polish + Mica wiring
4. Main window CRUD
5. Sticky note widget end-to-end
6. To-do widget end-to-end
7. Tray + always-on-top + opacity
8. Reminders
9. Polish & a11y
10. README + screenshots

## License

Private (pet project).

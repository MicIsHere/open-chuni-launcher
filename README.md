# Open Chunithm Launcher

Open-source desktop launcher for CHUNITHM, built with Tauri.

## Tech Stack

- [Tauri 2](https://v2.tauri.app/) + Rust — desktop shell & backend
- [Vue 3](https://vuejs.org/) + TypeScript + [Vite](https://vite.dev/) — frontend
- [Tailwind CSS v4](https://tailwindcss.com/) + [shadcn-vue](https://www.shadcn-vue.com/) — UI components
- [lucide](https://lucide.dev/) (`@lucide/vue`) — icons

## Development

Prerequisites: [Node.js](https://nodejs.org/) ≥ 20 and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (Rust toolchain + platform webview dependencies).

```bash
npm install
npm run tauri dev
```

## Production Build

```bash
npm run tauri build
```

## Naming Conventions

| Place                                  | Format       | Value                                  |
| -------------------------------------- | ------------ | -------------------------------------- |
| npm package / Rust crate / productName | `kebab-case` | `open-chunithm-launcher`               |
| Rust lib target                        | `snake_case` | `open_chunithm_launcher_lib`           |
| Window title / README                  | `Title Case` | `Open Chunithm Launcher`               |
| Bundle identifier                      | reverse-DNS  | `cc.moeneko.open-chunithm-launcher`    |

shadcn-vue components live under `src/components/ui/` (added via `npx shadcn-vue@latest add <component>`), shared helpers under `src/lib/`.

## Theming

The theme system is registry-driven and applies globally:

- `src/lib/themes.ts` — theme registry (single source of truth)
- `src/composables/useTheme.ts` — applies the theme to `<html>`, persists the choice, and follows the OS color-scheme preference until the user overrides it
- `src/style.css` — design tokens (`:root` light palette, `.dark` dark palette)

To add a theme: append a `ThemeDefinition` in the registry and, if it needs a custom palette, a `[data-theme="<id>"]` token block in `style.css`. The settings page renders the registry automatically.

## Internationalization

Translations live in `src/locales/<id>.lang` — a flat `key=value` format with `#` comments. Every file is automatically registered as a locale (via `import.meta.glob`); `language.name` inside the file is its native display name.

To add a language: drop a new `.lang` file (copy `zh-CN.lang` as a template). Missing keys fall back to `zh-CN`. `useI18n()` (in `src/i18n/`) exposes `t("key", { params })`, the locale list, and persistence; the language can be switched in Settings.

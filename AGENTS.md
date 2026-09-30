# AGENTS.md

Agent / contributor guide for **Open Chunithm Launcher** — a desktop launcher built with Tauri 2 + Vue 3.

## Commands

```bash
npm install          # install dependencies
npm run dev          # frontend dev server (http://localhost:1420, fixed port for Tauri)
npm run build        # type check (vue-tsc) + production build to dist/
npm run tauri dev    # full desktop app in dev mode
npm run tauri build  # production desktop bundle
cargo check          # Rust side only (run inside src-tauri/)
```

## Naming conventions

| Place                                  | Format       | Value                               |
| -------------------------------------- | ------------ | ----------------------------------- |
| npm package / Rust crate / productName | `kebab-case` | `open-chunithm-launcher`            |
| Rust lib target                        | `snake_case` | `open_chunithm_launcher_lib`        |
| Window title / README                  | `Title Case` | `Open Chunithm Launcher`            |
| Bundle identifier                      | reverse-DNS  | `cc.moeneko.open-chunithm-launcher` |

Vue/TS: components and pages in `PascalCase.vue`, composables `useX.ts`, everything else camelCase.
Rust: snake_case throughout; the lib name keeps the `_lib` suffix (Windows bin/lib name clash, see Cargo.toml comment).

## Project layout

```
src/
  components/
    layout/   AppSidebar (nav shell)
    pages/    HomePage, PluginsPage, LogsPage, SettingsPage  (matched by id in lib/navigation.ts)
    settings/ SettingCard + controls/ (per-control-type wrappers)
    ui/       shadcn-vue components (button, input, number-field, select, tooltip)
  composables/  useTheme, useSettings
  i18n/         useI18n + .lang parser
  lib/          themes.ts, navigation.ts, utils.ts (cn)
  locales/      *.lang files (one per locale)
  styles/       tokens / base / layout / sidebar / settings / form / overlay
src-tauri/
  src/
    lib.rs      Builder、插件注册、命令注册
    commands.rs 前端命令（launch_game 阻塞至游戏退出、stop_game、list_plugin_dlls）+ 运行状态
    launcher.rs 启动流程编排（清理 → amdaemon → 游戏 → 清理，对应原 bat 时序）
    inject.rs   注入解耦：把「注入器 + 目标 + DLL 清单」组装成命令行（支持绝对路径）
    plugins.rs  内置插件：扫描打包的插件目录、解析同名 JSON 清单、图标转 data URL
  plugins/      内置插件目录（每插件一个文件夹：DLL + 英文默认清单 + `<name>.<locale>.json` 语言覆盖 + 图标，经 bundle.resources 打包）
  （capabilities/ 权限、icons/ 图标、tauri.conf.json 配置）
```

## CSS architecture (Tailwind CSS v4)

`src/style.css` is only the entry point — it imports Tailwind and the split style files under `src/styles/`, one per domain, each commented in concise Chinese:

| File | 内容 |
| --- | --- |
| `tokens.css` | 设计令牌：`@custom-variant dark`、`@theme inline` 映射、`:root`/`.dark` 调色板（唯一的令牌定义处） |
| `base.css` | 元素基础样式、页面切换过渡（`.page-*`）、主题切换淡变（`theme-switching`）、`prefers-reduced-motion` 守卫 |
| `layout.css` | 应用壳：`.app-shell`、`.app-header*`、`.app-content`、`.page-center` |
| `sidebar.css` | 侧边栏：`.app-sidebar`（`data-collapsed` 驱动折叠）、`.sidebar-label/nav/footer` 等 |
| `settings.css` | 设置页：`.settings-page/section*`、`.setting-card*` |
| `form.css` | 表单控件：`.control-ring`、`.text-field`、`.select-trigger`、`.number-field-step` |
| `overlay.css` | 浮层：`.select-content`、`.select-viewport/item*`、`.tooltip-content` |

**规则**

1. 新样式先判断归属领域放入对应文件；类名用应用前缀 + 语义名（`.setting-card-icon`，而非 `.icon`），每个类配一行简洁中文注释描述效果。
2. **层叠层级**：`@layer components` 中的类恒输给 utilities 层。若共享样式必须覆盖同一元素上的变体工具类（如 `buttonVariants` 自带的 `justify-center`），不能放 CSS 文件——用模板内的共享常量 + `cn()`/tw-merge（范例：`AppSidebar.vue` 的 `sidebarItemClass`）。
3. 仅切换类名的组件状态用 `data-*` 属性 + CSS 描述（`[data-collapsed]`、`[data-active]`），避免模板里的条件类字符串——前提是无工具类冲突（规则 2）。
4. `@apply` 只能引用工具类，不能引用 `@layer components` 里的自定义类。
5. 动画保持基础简洁：使用 `tw-animate-css` 的 `animate-in/out` + `fade/zoom/slide` 词表，并遵守 `base.css` 的 `prefers-reduced-motion` 守卫。

## 组件库与 UI 库

**UI 库（`src/components/ui/`，基于 shadcn-vue + reka-ui 的本地化副本）**

- 现有组件：`button`、`input`、`number-field`、`select`（Aria 行为风格：弹层恒等于触发器宽度、选中项右侧勾选）、`tooltip`。
- 移植约定：`cn(...)` + `props.class` 合并（tw-merge 处理冲突）、`data-slot` 标注、`useForwardPropsEmits`/`useForwardProps` 转发。
- reka-ui 只输出 `data-state="open|closed"` 等带值属性，动画变体必须写 `data-[state=open]:` 形式；`data-highlighted`、`data-disabled` 为布尔属性，可用 `data-highlighted:` 简写。
- 弹层组件的定位属性（align、avoidCollisions 等）在封装层 `withDefaults` 中给出与 reka 运行时一致的默认值，否则 JetBrains IDE 会误判为必填属性。
- 新增组件：`npx shadcn-vue@latest add <name>`，随后按上述约定本地化调整。

**业务组件库**

- `components/layout/AppSidebar.vue` — 侧边栏导航；折叠逻辑（手动 + 窄窗口媒体查询）与持久化都在此，样式走 `styles/sidebar.css` + `sidebarItemClass`。
- `components/settings/SettingCard.vue` — 设置项卡片（图标/标题/描述 + 控件插槽）；控件封装在 `components/settings/controls/`：`SettingText`、`SettingPath`（Tauri 目录对话框）、`SettingSelect`（枚举，选项 `{ value, label, icon }`）、`SettingNumber`。
- `pages/` 页面由 `lib/navigation.ts` 的 `navSections` 驱动，`App.vue` 按 id 动态挂载；新增页面 = 加导航项 + 页面组件 + `pages` 映射。

**图标**：统一使用 `@lucide/vue`，命名导入（如 `Disc3`、`ChevronsLeft`），装饰性图标加 `aria-hidden="true"`。

## Theming

`src/lib/themes.ts` is the single registry. A theme = `{ id, icon, dark }`; `dark: true` toggles the `.dark` class, and a custom palette (optional) is a `[data-theme="<id>"]` block in `tokens.css`. Adding a theme also means adding `theme.<id>` keys to every locale file. `useTheme()` applies the theme to `<html>` before mount, persists the choice, and follows the OS preference until the user overrides it.

## Internationalization

Locales are `src/locales/<id>.lang` — flat `key=value`, `#` comments, parsed by `src/i18n/parser.ts`. Dropping a file registers a locale; `language.name` inside it is the native display name. All user-visible strings go through `t("key")` from `useI18n()` — no hardcoded UI text. Missing keys fall back to `zh-CN`. The language itself is switched in Settings, shown in its native name.

## Settings

Persisted via `useSettings()` (one JSON document in localStorage, merged over defaults so new keys survive old storage). Theme and language have their own composables/persistence. Each setting renders as a `SettingCard` (icon + title + description + control slot); pick the control wrapper from `components/settings/controls/` matching the value type (text, path, select, number).

## Tauri specifics

- Every plugin needs three touchpoints: npm package, `src-tauri/Cargo.toml`, `.plugin(...)` in `lib.rs`, plus a permission in `src-tauri/capabilities/default.json` (see `tauri-plugin-dialog` for the path picker).
- Browser-only previews (vite preview) have no Tauri runtime — guard native APIs (e.g. `SettingPath` disables browsing when `"__TAURI_INTERNALS__" not in window`).
- Long-running commands (like `launch_game`, which blocks until the game exits) must be `async` + `spawn_blocking` — a sync command runs on the main thread and freezes the window. Progress goes through `launch://log` events, session state through `launch://state` + the `is_running` command; the frontend keeps both in the `useLaunch()` store (logs persisted to localStorage, capped at 500 lines).
- 内置插件：每个插件是 `plugins/` 下的独立文件夹——`<name>.dll` + `<name>.json`（英文默认清单：name/description/icon/version/author）+ `<name>.<locale>.json`（语言覆盖）+ 同名图片（图标回退）。名称与描述由清单提供、均为只读，前端经 `resolveLocalizedText()` 随当前语言解析（精确 → 语言前缀 → 英文默认）；启用的内置插件在启动时由后端从资源目录复制到游戏 bin 目录再注入，DLL 清单支持绝对路径。
- 游戏路径设置指向游戏**根目录**（如 `D:\...\HDD`），后端经 `launcher::resolve_bin_dir()` 进入其 bin 子目录运行注入器与游戏（用户直接选到 bin 时也兼容）；内置插件 DLL 同样复制到 bin。
- Icons are generated from `public/favicon.svg` via `npx tauri icon public/favicon.svg`; `src-tauri/gen/` is generated output.

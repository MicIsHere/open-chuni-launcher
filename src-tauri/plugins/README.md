# 内置插件目录

本目录中的插件会随启动器一起打包，安装后位于程序资源目录的 `plugins/` 下，可在「插件」页中管理与启用。

## 目录结构

每个插件一个独立文件夹，文件夹内：

```
plugins/
  Duolinguo/
    Duolinguo.dll            ← 插件本体（注入到游戏进程的 DLL）
    Duolinguo.json           ← 默认清单（英文）：name/description/icon/version/author
    Duolinguo.zh-CN.json     ← 中文覆盖清单（每种语言一个文件）
    Duolinguo.png            ← 图标（可选，支持 png/jpg/jpeg/svg）
```

## 默认清单（Duolinguo.json 示例）

```json
{
  "name": "Duolinguo",
  "description": "Connects to nationwide battle servers with built-in anti-cheat.",
  "version": "1.0.0",
  "author": "your name"
}
```

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `name` | 否 | 插件名称，展示为只读；未提供时取 DLL 文件名 |
| `description` | 否 | 插件描述，只读，用户不可更改 |
| `icon` | 否 | 图标文件名（相对插件文件夹，支持 png/jpg/jpeg/svg）；未提供时使用与 DLL 同名的图片 |
| `version` | 否 | 版本号 |
| `author` | 否 | 作者 |

## 多语言覆盖（Duolinguo.zh-CN.json 示例）

每种语言一个覆盖文件，文件名 = `DLL名.语言ID.json`，只需提供 `name` / `description`：

```json
{
  "name": "全国对战",
  "description": "用于链接全国对战，内置反作弊."
}
```

**解析顺序**：当前语言覆盖文件 → 同语言前缀（`en-US` 匹配 `en-*`）→ 默认清单（英文）→ DLL 文件名。切换启动器语言后插件卡片即时跟随，无需重新扫描。

没有 JSON 清单的 DLL 也能被识别，此时名称取文件名、描述为空。

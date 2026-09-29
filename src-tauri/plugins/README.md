# 内置插件目录

本目录中的插件会随启动器一起打包，安装后位于程序资源目录的 `plugins/` 下，可在「设置 → 插件」中管理与启用。

## 插件格式

每个插件由一对**同名文件**组成（放在本目录即可）：

```
Duolinguo.dll   ← 插件本体（注入到游戏进程的 DLL）
Duolinguo.json  ← 插件清单：自定义名称、描述、图标等信息
```

## 清单字段（Duolinguo.json 示例）

`name` 与 `description` 支持两种写法，可混用：

```json
{
  "name": { "zh-CN": "全国对战", "en-US": "Nationwide Battle" },
  "description": {
    "zh-CN": "用于链接全国对战，内置反作弊.",
    "en-US": "Connects to nationwide battle servers with built-in anti-cheat."
  },
  "icon": "Duolinguo.png",
  "version": "1.0.0",
  "author": "your name"
}
```

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `name` | 否 | 插件名称，展示为只读；未提供时取 DLL 文件名 |
| `description` | 否 | 插件描述，只读，用户不可更改 |
| `icon` | 否 | 图标文件名（相对本目录，支持 png/jpg/svg/ico）；未提供时显示默认图标 |
| `version` | 否 | 版本号 |
| `author` | 否 | 作者 |

**i18n 规则**：`name`/`description` 既可以是语言中立的字符串（所有语言显示同一文案），也可以是 `{语言ID: 文案}` 映射。解析顺序：当前语言精确匹配 → 同语言前缀（`en-US` 匹配 `en-*`）→ 回退语言（`zh-CN`）→ 首个非空值。切换启动器语言后插件卡片即时跟随，无需重新扫描。

没有同名 JSON 的 DLL 也能被识别，此时名称取文件名、描述为空。

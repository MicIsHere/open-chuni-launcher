//! 内置插件：扫描随启动器打包的插件目录（每个插件一个独立文件夹），
//! 读取默认 JSON 清单（英文）与各语言覆盖清单，供前端展示与注入。
//!
//! 目录结构：
//! ```text
//! plugins/
//!   <plugin-folder>/
//!     <name>.dll              ← 插件本体
//!     <name>.json             ← 默认清单（英文）：name/description/icon/version/author
//!     <name>.<locale>.json    ← 语言覆盖（如 duolinguo.zh-CN.json）
//!     <name>.png|jpg|svg      ← 图标（或由清单 icon 字段指定）
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{fs};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

/// 插件默认清单（英文，全部字段可选）
#[derive(Deserialize, Default)]
#[serde(default)]
struct PluginManifest {
    name: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    version: Option<String>,
    author: Option<String>,
}

/// 语言覆盖清单（该语言的 name/description）
#[derive(Deserialize, Default)]
#[serde(default)]
struct PluginLocaleOverrides {
    name: Option<String>,
    description: Option<String>,
}

/// 返回给前端的插件信息
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    /// DLL 文件名（作为插件在设置中的唯一标识）
    pub file: String,
    /// 清单中的默认名称；无清单时取 DLL 文件名。
    /// 字符串（语言中立）或 {"en-US": …, "zh-CN": …} 多语言映射
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Value>,
    /// 清单中的描述（只读，用户不可更改），形态同 name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Value>,
    /// 版本号（来自默认清单）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// DLL 绝对路径（注入时直接使用）
    pub path: String,
    /// 图标的 data URL；无图标时为 null，前端显示默认图标
    pub icon_data_url: Option<String>,
    /// 是否存在默认清单：有则名称与描述来自清单、不可更改
    pub has_manifest: bool,
}

/// 解析插件根目录：打包后取资源目录下的 plugins，开发时回退到源码目录
pub fn plugins_dir(app: &AppHandle) -> Option<PathBuf> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let dir = resource_dir.join("plugins");
        if dir.is_dir() {
            return Some(dir);
        }
    }
    let dev_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("plugins");
    dev_dir.is_dir().then_some(dev_dir)
}

/// 扫描插件根目录：每个直接子文件夹是一个插件，按 DLL 文件名排序
pub fn scan_plugins(dir: &Path) -> Vec<PluginInfo> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut plugins: Vec<PluginInfo> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| read_plugin_folder(&entry.path()))
        .collect();

    plugins.sort_by(|a, b| a.file.to_lowercase().cmp(&b.file.to_lowercase()));
    plugins
}

/// 读取单个插件文件夹
fn read_plugin_folder(folder: &Path) -> Option<PluginInfo> {
    let dll_path = find_dll(folder)?;
    let dll_file = dll_path.file_name()?.to_string_lossy().to_string();
    let stem = dll_path.file_stem()?.to_string_lossy().to_string();

    let manifest_path = folder.join(format!("{stem}.json"));
    let manifest_text = fs::read_to_string(&manifest_path).ok();
    let manifest: PluginManifest = manifest_text
        .as_deref()
        .and_then(|text| serde_json::from_str(text).ok())
        .unwrap_or_default();

    let (locale_names, locale_descriptions) = read_locale_overrides(folder, &stem);

    let icon_path = manifest
        .icon
        .as_deref()
        .map(|icon| folder.join(icon))
        .or_else(|| find_default_icon(folder, &stem))
        .and_then(|icon_path| icon_to_data_url(&icon_path));

    Some(PluginInfo {
        file: dll_file,
        name: build_localized_value(manifest.name, &locale_names, Some(stem.clone())),
        description: build_localized_value(manifest.description, &locale_descriptions, None),
        version: manifest.version,
        author: manifest.author,
        path: dll_path.to_string_lossy().to_string(),
        icon_data_url: icon_path,
        has_manifest: manifest_text.is_some(),
    })
}

/// 定位插件文件夹中的 DLL：优先与文件夹同名的 DLL，否则取第一个（按文件名排序）
fn find_dll(folder: &Path) -> Option<PathBuf> {
    let mut dlls: Vec<PathBuf> = fs::read_dir(folder)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("dll"))
        })
        .collect();
    dlls.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let folder_name = folder.file_name()?.to_string_lossy().to_lowercase();
    let preferred = dlls.iter().position(|path| {
        path.file_stem()
            .is_some_and(|stem| stem.to_string_lossy().to_lowercase() == folder_name)
    });
    match preferred {
        Some(index) => Some(dlls.remove(index)),
        None => dlls.into_iter().next(),
    }
}

/// 读取语言覆盖清单：`<stem>.<locale>.json` → (locale → 覆盖字段)
fn read_locale_overrides(
    folder: &Path,
    stem: &str,
) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let mut names = BTreeMap::new();
    let mut descriptions = BTreeMap::new();

    let Ok(entries) = fs::read_dir(folder) else {
        return (names, descriptions);
    };
    let prefix = format!("{stem}.");
    for entry in entries.flatten() {
        let file = entry.file_name().to_string_lossy().to_string();
        let Some(suffix) = file.strip_prefix(&prefix).and_then(|rest| rest.strip_suffix(".json"))
        else {
            continue;
        };
        // 语言 ID 形如 zh-CN / en（字母开头，仅含字母、数字与连字符）
        let is_locale_id = suffix.starts_with(|c: char| c.is_ascii_alphabetic())
            && suffix
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-');
        if !is_locale_id {
            continue;
        }

        let Ok(text) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let Ok(overrides) = serde_json::from_str::<PluginLocaleOverrides>(&text) else {
            continue;
        };
        if let Some(name) = overrides.name {
            names.insert(suffix.to_string(), name);
        }
        if let Some(description) = overrides.description {
            descriptions.insert(suffix.to_string(), description);
        }
    }
    (names, descriptions)
}

/// 合并基础文案与语言覆盖为多语言值：
/// 无覆盖 → 字符串；有覆盖 → {"en-US": 基础英文, ...覆盖}；两者皆无 → None
/// （fallback 用于名称回落到 DLL 文件名，描述无清单时保持 None）
fn build_localized_value(
    base: Option<String>,
    overrides: &BTreeMap<String, String>,
    fallback: Option<String>,
) -> Option<Value> {
    let base = base.filter(|text| !text.trim().is_empty());
    if overrides.is_empty() {
        return base.or(fallback).map(Value::String);
    }

    let mut map = serde_json::Map::new();
    if let Some(text) = base {
        map.insert("en-US".to_string(), Value::String(text));
    }
    for (locale, text) in overrides {
        map.insert(locale.clone(), Value::String(text.clone()));
    }
    Some(Value::Object(map))
}

/// 图标回退：与 DLL 同名的 png/jpg/jpeg/svg
fn find_default_icon(folder: &Path, stem: &str) -> Option<PathBuf> {
    ["png", "jpg", "jpeg", "svg"]
        .iter()
        .map(|ext| folder.join(format!("{stem}.{ext}")))
        .find(|path| path.is_file())
}

/// 读取图标文件并编码为 data URL
fn icon_to_data_url(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let mime = match path.extension()?.to_string_lossy().to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        _ => return None,
    };
    Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("launcher-plugins-{tag}-{unique}"));
        fs::create_dir(&dir).unwrap();
        dir
    }

    #[test]
    fn scans_plugin_folders_and_merges_locale_overrides() {
        let dir = temp_dir("scan");

        // 插件 A：DLL + 英文默认清单 + 中文覆盖 + svg 图标
        let folder_a = dir.join("plugin-a");
        fs::create_dir(&folder_a).unwrap();
        fs::write(folder_a.join("plugin-a.dll"), b"").unwrap();
        fs::write(
            folder_a.join("plugin-a.json"),
            r#"{
                "name": "Plugin A",
                "description": "English description",
                "version": "1.0.0",
                "icon": "plugin-a.svg"
            }"#,
        )
        .unwrap();
        fs::write(
            folder_a.join("plugin-a.zh-CN.json"),
            r#"{"name": "插件A", "description": "中文描述"}"#,
        )
        .unwrap();
        fs::write(folder_a.join("plugin-a.svg"), "<svg/>").unwrap();

        // 插件 B：仅 DLL（无清单）
        let folder_b = dir.join("plugin-b");
        fs::create_dir(&folder_b).unwrap();
        fs::write(folder_b.join("plugin-b.dll"), b"").unwrap();

        // 干扰项：散落文件与无 DLL 的文件夹被忽略；含 DLL 的子文件夹是合法插件
        fs::write(dir.join("notes.txt"), b"").unwrap();
        fs::create_dir(dir.join("empty-folder")).unwrap();
        let nested = dir.join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("nested.dll"), b"").unwrap();

        let plugins = scan_plugins(&dir);
        assert_eq!(plugins.len(), 3);

        let nested_info = &plugins[0];
        assert_eq!(nested_info.file, "nested.dll");
        assert_eq!(
            nested_info.name.as_ref().unwrap(),
            &serde_json::json!("nested")
        );

        let a = &plugins[1];
        assert_eq!(a.file, "plugin-a.dll");
        assert_eq!(
            a.name.as_ref().unwrap(),
            &serde_json::json!({"en-US": "Plugin A", "zh-CN": "插件A"})
        );
        assert_eq!(
            a.description.as_ref().unwrap(),
            &serde_json::json!({"en-US": "English description", "zh-CN": "中文描述"})
        );
        assert_eq!(a.version.as_deref(), Some("1.0.0"));
        assert!(a.has_manifest);
        assert!(a
            .icon_data_url
            .as_deref()
            .unwrap_or("")
            .starts_with("data:image/svg+xml;base64,"));

        let b = &plugins[2];
        assert_eq!(b.file, "plugin-b.dll");
        assert_eq!(b.name.as_ref().unwrap(), &serde_json::json!("plugin-b"));
        assert_eq!(b.description, None);
        assert!(!b.has_manifest);
        assert_eq!(b.icon_data_url, None);

        // 不存在的目录 → 空列表
        assert!(scan_plugins(&dir.join("missing")).is_empty());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn prefers_dll_matching_the_folder_name() {
        let dir = temp_dir("prefer");
        let folder = dir.join("my-plugin");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("other.dll"), b"").unwrap();
        fs::write(folder.join("my-plugin.dll"), b"").unwrap();

        let info = read_plugin_folder(&folder).unwrap();
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(info.file, "my-plugin.dll");
        assert_eq!(info.name.as_ref().unwrap(), &serde_json::json!("my-plugin"));
    }
}

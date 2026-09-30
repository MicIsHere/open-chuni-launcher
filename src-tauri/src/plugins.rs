use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{fs};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

#[derive(Deserialize, Default)]
#[serde(default)]
struct PluginManifest {
    name: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    version: Option<String>,
    author: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct PluginLocaleOverrides {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub path: String,
    pub icon_data_url: Option<String>,
    pub has_manifest: bool,
}

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

fn find_default_icon(folder: &Path, stem: &str) -> Option<PathBuf> {
    ["png", "jpg", "jpeg", "svg"]
        .iter()
        .map(|ext| folder.join(format!("{stem}.{ext}")))
        .find(|path| path.is_file())
}

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

        let folder_b = dir.join("plugin-b");
        fs::create_dir(&folder_b).unwrap();
        fs::write(folder_b.join("plugin-b.dll"), b"").unwrap();

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

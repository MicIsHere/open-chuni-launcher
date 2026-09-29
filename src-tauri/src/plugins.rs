//! 内置插件：扫描随启动器打包的插件目录，读取同名 JSON 清单，
//! 供前端展示与注入。

use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

/// 插件清单（X.dll 对应的 X.json，全部字段可选）。
/// name/description 既可以是字符串，也可以是 {locale: text} 多语言映射，
/// 由前端按当前语言解析。
#[derive(Deserialize, Default)]
#[serde(default)]
struct PluginManifest {
    name: Option<Value>,
    description: Option<Value>,
    icon: Option<String>,
    version: Option<String>,
    author: Option<String>,
}

/// 返回给前端的插件信息
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    /// DLL 文件名（作为插件在设置中的唯一标识）
    pub file: String,
    /// 清单中的默认名称；无清单时取文件名（字符串或多语言映射）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Value>,
    /// 清单中的描述（只读，用户不可更改）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Value>,
    pub version: Option<String>,
    pub author: Option<String>,
    /// DLL 绝对路径（注入时直接使用）
    pub path: String,
    /// 图标的 data URL；无图标时为 null，前端显示默认图标
    pub icon_data_url: Option<String>,
    /// 是否存在同名 JSON 清单：有则名称与描述来自清单、不可更改
    pub has_manifest: bool,
}

/// 解析插件目录：打包后取资源目录下的 plugins，开发时回退到源码目录
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

/// 读取单个插件：X.dll + 同名 X.json
pub fn read_plugin_info(dir: &Path, dll_path: &Path) -> Option<PluginInfo> {
    let file = dll_path.file_name()?.to_string_lossy().to_string();
    let stem = dll_path.file_stem()?.to_string_lossy().to_string();

    let manifest_text = std::fs::read_to_string(dir.join(format!("{stem}.json"))).ok();
    let manifest: PluginManifest = manifest_text
        .as_deref()
        .and_then(|text| serde_json::from_str(text).ok())
        .unwrap_or_default();

    let icon_data_url = manifest
        .icon
        .as_deref()
        .map(|icon| dir.join(icon))
        .and_then(|icon_path| icon_to_data_url(&icon_path));

    Some(PluginInfo {
        file,
        name: manifest.name,
        description: manifest.description,
        version: manifest.version,
        author: manifest.author,
        path: dll_path.to_string_lossy().to_string(),
        icon_data_url,
        has_manifest: manifest_text.is_some(),
    })
}

/// 读取图标文件并编码为 data URL
fn icon_to_data_url(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
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
        std::fs::create_dir(&dir).unwrap();
        dir
    }

    #[test]
    fn reads_localized_manifest_fields() {
        let dir = temp_dir("manifest");
        std::fs::write(dir.join("sample.dll"), b"").unwrap();
        std::fs::write(
            dir.join("sample.json"),
            r#"{
                "name": {"zh-CN": "示例", "en-US": "Sample"},
                "description": {"zh-CN": "中文描述", "en-US": "English description"},
                "icon": "sample.svg"
            }"#,
        )
        .unwrap();
        std::fs::write(dir.join("sample.svg"), "<svg xmlns=\"http://www.w3.org/2000/svg\"/>")
            .unwrap();

        let info = read_plugin_info(&dir, &dir.join("sample.dll")).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(info.file, "sample.dll");
        assert_eq!(
            info.name.unwrap(),
            serde_json::json!({"zh-CN": "示例", "en-US": "Sample"})
        );
        assert_eq!(
            info.description.unwrap(),
            serde_json::json!({"zh-CN": "中文描述", "en-US": "English description"})
        );
        assert!(info.has_manifest);
        assert!(info
            .icon_data_url
            .as_deref()
            .unwrap_or("")
            .starts_with("data:image/svg+xml;base64,"));
    }

    #[test]
    fn plain_manifest_fields_pass_through() {
        let dir = temp_dir("plain");
        std::fs::write(dir.join("plain.dll"), b"").unwrap();
        std::fs::write(
            dir.join("plain.json"),
            r#"{"name": "Plain", "description": "plain description"}"#,
        )
        .unwrap();

        let info = read_plugin_info(&dir, &dir.join("plain.dll")).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(info.name.unwrap(), serde_json::json!("Plain"));
        assert_eq!(info.description.unwrap(), serde_json::json!("plain description"));
        assert!(info.has_manifest);
    }

    #[test]
    fn dll_without_manifest_has_no_metadata() {
        let dir = temp_dir("bare");
        std::fs::write(dir.join("bare.dll"), b"").unwrap();

        let info = read_plugin_info(&dir, &dir.join("bare.dll")).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(info.name, None);
        assert_eq!(info.description, None);
        assert!(!info.has_manifest);
        assert_eq!(info.icon_data_url, None);
    }
}

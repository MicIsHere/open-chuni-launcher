use std::path::Path;

use serde::Deserialize;

pub struct IniEntry {
    pub key: &'static str,
    pub value: String,
}

pub struct SectionPatch {
    pub section: &'static str,
    pub entries: Vec<IniEntry>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SegatoolsPatch {
    pub dns: Option<DnsPatch>,
    pub keychip: Option<KeychipPatch>,
    pub vfs: Option<VfsPatch>,
    pub gpio: Option<GpioPatch>,
    pub gfx: Option<GfxPatch>,
    pub aime: Option<AimePatch>,
    pub io3: Option<Io3Patch>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsPatch {
    pub default: String,
    pub aimedb: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeychipPatch {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VfsPatch {
    pub option: String,
    pub amfs: String,
    pub appdata: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpioPatch {
    pub dipsw1: u8,
    pub dipsw2: u8,
    pub dipsw3: u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GfxPatch {
    pub windowed: u8,
    pub framed: u8,
    pub monitor: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AimePatch {
    pub enable: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Io3Patch {
    pub test: String,
    pub service: String,
    pub coin: String,
}

impl SegatoolsPatch {
    pub fn sections(&self) -> Vec<SectionPatch> {
        let mut patches: Vec<SectionPatch> = Vec::new();

        if let Some(dns) = &self.dns {
            let mut entries = vec![IniEntry {
                key: "default",
                value: dns.default.clone(),
            }];
            if let Some(aimedb) = &dns.aimedb {
                entries.push(IniEntry {
                    key: "aimedb",
                    value: aimedb.clone(),
                });
            }
            patches.push(SectionPatch {
                section: "dns",
                entries,
            });
        }

        if let Some(keychip) = &self.keychip {
            patches.push(SectionPatch {
                section: "keychip",
                entries: vec![IniEntry {
                    key: "id",
                    value: keychip.id.clone(),
                }],
            });
        }

        if let Some(vfs) = &self.vfs {
            patches.push(SectionPatch {
                section: "vfs",
                entries: vec![
                    IniEntry { key: "option", value: vfs.option.clone() },
                    IniEntry { key: "amfs", value: vfs.amfs.clone() },
                    IniEntry { key: "appdata", value: vfs.appdata.clone() },
                ],
            });
        }

        if let Some(gpio) = &self.gpio {
            patches.push(SectionPatch {
                section: "gpio",
                entries: vec![
                    IniEntry { key: "dipsw1", value: gpio.dipsw1.to_string() },
                    IniEntry { key: "dipsw2", value: gpio.dipsw2.to_string() },
                    IniEntry { key: "dipsw3", value: gpio.dipsw3.to_string() },
                ],
            });
        }

        if let Some(gfx) = &self.gfx {
            patches.push(SectionPatch {
                section: "gfx",
                entries: vec![
                    IniEntry { key: "windowed", value: gfx.windowed.to_string() },
                    IniEntry { key: "framed", value: gfx.framed.to_string() },
                    IniEntry { key: "monitor", value: gfx.monitor.to_string() },
                ],
            });
        }

        if let Some(aime) = &self.aime {
            patches.push(SectionPatch {
                section: "aime",
                entries: vec![IniEntry {
                    key: "enable",
                    value: if aime.enable { "1".to_string() } else { "0".to_string() },
                }],
            });
        }

        if let Some(io3) = &self.io3 {
            patches.push(SectionPatch {
                section: "io3",
                entries: vec![
                    IniEntry { key: "test", value: io3.test.clone() },
                    IniEntry { key: "service", value: io3.service.clone() },
                    IniEntry { key: "coin", value: io3.coin.clone() },
                ],
            });
        }

        patches
    }
}

pub fn is_valid_keychip(keychip: &str) -> bool {
    let value = keychip.trim();
    let bytes = value.as_bytes();
    bytes.len() == 16
        && bytes[0] == b'A'
        && bytes[1].is_ascii_digit()
        && bytes[2].is_ascii_digit()
        && (bytes[3] == b'E' || bytes[3] == b'X')
        && bytes[4] == b'-'
        && matches!(&value[5..7], "01" | "20")
        && matches!(bytes[7], b'A' | b'B' | b'C' | b'D' | b'U')
        && bytes[8..16].iter().all(|c| c.is_ascii_digit())
}

pub fn patch_ini(bin_dir: &Path, patches: &[SectionPatch]) -> Result<(), String> {
    let ini_path = bin_dir.join("segatools.ini");
    let content = std::fs::read_to_string(&ini_path).unwrap_or_default();
    let patched = apply_patches(&content, patches);
    std::fs::write(&ini_path, patched)
        .map_err(|error| format!("写入 segatools.ini 失败：{error}"))
}

struct IniSection {
    name: Option<String>,
    body: Vec<String>,
}

fn parse_sections(content: &str) -> Vec<IniSection> {
    let mut sections = vec![IniSection {
        name: None,
        body: Vec::new(),
    }];
    for raw in content.lines() {
        let trimmed = raw.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            sections.push(IniSection {
                name: Some(trimmed[1..trimmed.len() - 1].trim().to_lowercase()),
                body: Vec::new(),
            });
        } else {
            sections
                .last_mut()
                .expect("至少存在一个节")
                .body
                .push(raw.to_string());
        }
    }
    sections
}

fn key_of(line: &str) -> Option<String> {
    let eq = line.find('=')?;
    Some(line[..eq].trim().to_lowercase())
}

fn apply_patches(content: &str, patches: &[SectionPatch]) -> String {
    let mut sections = parse_sections(content);

    for patch in patches {
        let managed_keys: Vec<String> =
            patch.entries.iter().map(|entry| entry.key.to_lowercase()).collect();
        let managed_lines: Vec<String> = patch
            .entries
            .iter()
            .map(|entry| format!("{} = {}", entry.key, entry.value))
            .collect();

        if let Some(section) = sections
            .iter_mut()
            .find(|section| section.name.as_deref() == Some(patch.section))
        {
            let kept: Vec<String> = section
                .body
                .iter()
                .filter(|line| key_of(line).map_or(true, |key| !managed_keys.contains(&key)))
                .cloned()
                .collect();
            let mut body = managed_lines.clone();
            body.extend(kept);
            section.body = body;
        } else {
            sections.push(IniSection {
                name: Some(patch.section.to_string()),
                body: managed_lines.clone(),
            });
        }
    }

    let mut output = String::new();
    for section in &sections {
        let mut body = section.body.clone();
        while body.last().map_or(false, |line| line.trim().is_empty()) {
            body.pop();
        }
        if let Some(name) = &section.name {
            output.push_str(&format!("[{name}]\n"));
        }
        for line in &body {
            output.push_str(line);
            output.push('\n');
        }
        if section.name.is_some() {
            output.push('\n');
        }
    }
    output.trim_end_matches('\n').to_string() + "\n"
}

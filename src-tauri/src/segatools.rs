use std::path::Path;


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

pub fn patch_server_config(
    bin_dir: &Path,
    dns_default: &str,
    dns_aimedb: Option<&str>,
    keychip: &str,
) -> Result<(), String> {
    let ini_path = bin_dir.join("segatools.ini");
    let content = std::fs::read_to_string(&ini_path).unwrap_or_default();
    let patched = apply_server_patch(&content, dns_default, dns_aimedb, keychip);
    std::fs::write(&ini_path, patched)
        .map_err(|error| format!("写入 segatools.ini 失败：{error}"))
}

fn managed_keys(section: &str) -> &'static [&'static str] {
    match section {
        "dns" => &["default", "aimedb"],
        "keychip" => &["keychip"],
        _ => &[],
    }
}

fn key_of(line: &str) -> Option<String> {
    let eq = line.find('=')?;
    Some(line[..eq].trim().to_lowercase())
}

fn apply_server_patch(
    content: &str,
    dns_default: &str,
    dns_aimedb: Option<&str>,
    keychip: &str,
) -> String {
    struct Section {
        name: Option<String>,
        body: Vec<String>,
    }

    let mut sections: Vec<Section> = vec![Section {
        name: None,
        body: Vec::new(),
    }];
    for raw in content.lines() {
        let trimmed = raw.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            sections.push(Section {
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

    let dns_lines = match dns_aimedb {
        Some(aimedb) => vec![
            format!("default = {dns_default}"),
            format!("aimedb = {aimedb}"),
        ],
        None => vec![format!("default = {dns_default}")],
    };
    let targets: Vec<(&str, Vec<String>)> = vec![
        ("dns", dns_lines),
        ("keychip", vec![format!("keychip = {keychip}")]),
    ];

    for (name, managed) in &targets {
        let keys = managed_keys(name);
        if let Some(section) = sections
            .iter_mut()
            .find(|section| section.name.as_deref() == Some(name))
        {
            let kept: Vec<String> = section
                .body
                .iter()
                .filter(|line| key_of(line).map_or(true, |key| !keys.contains(&key.as_str())))
                .cloned()
                .collect();
            let mut body: Vec<String> = managed.clone();
            body.extend(kept);
            section.body = body;
        } else {
            sections.push(Section {
                name: Some(name.to_string()),
                body: managed.clone(),
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
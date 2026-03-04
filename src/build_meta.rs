use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMeta {
    pub version: u32,
    pub script: String,
    pub action: String,
    pub target: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub llm_ms: Option<u128>,
    pub build_ms: Option<u128>,
    pub run_ms: Option<u128>,
    pub total_ms: u128,
    pub timestamp_unix_ms: u128,
    pub status: String,
    pub token_usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildMetaHistory {
    pub version: u32,
    pub entries: Vec<BuildMeta>,
}

pub fn dist_dir_for_input(input: &Path) -> PathBuf {
    let filename = input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("sculpt");
    if let Some(base) = filename.strip_suffix(".sculpt.json") {
        return Path::new("dist").join(base);
    }
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sculpt");
    Path::new("dist").join(stem)
}

pub fn meta_path(dist_dir: &Path) -> PathBuf {
    dist_dir.join("build.meta.json")
}

pub fn history_path(dist_dir: &Path) -> PathBuf {
    dist_dir.join("build.history.json")
}

pub fn write_build_meta(dist_dir: &Path, meta: &BuildMeta) -> Result<()> {
    fs::create_dir_all(dist_dir)?;
    let json = serde_json::to_string_pretty(meta)?;
    fs::write(meta_path(dist_dir), json)?;
    append_build_history(dist_dir, meta, 30)?;
    Ok(())
}

pub fn read_build_meta(dist_dir: &Path) -> Option<BuildMeta> {
    let data = fs::read_to_string(meta_path(dist_dir)).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn read_build_history(dist_dir: &Path) -> Vec<BuildMeta> {
    let data = match fs::read_to_string(history_path(dist_dir)) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let history = match serde_json::from_str::<BuildMetaHistory>(&data) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    history.entries
}

fn append_build_history(dist_dir: &Path, meta: &BuildMeta, max_entries: usize) -> Result<()> {
    let mut entries = read_build_history(dist_dir);
    entries.push(meta.clone());
    if entries.len() > max_entries {
        let start = entries.len().saturating_sub(max_entries);
        entries = entries[start..].to_vec();
    }
    let history = BuildMetaHistory {
        version: 1,
        entries,
    };
    let json = serde_json::to_string_pretty(&history)?;
    fs::write(history_path(dist_dir), json)?;
    Ok(())
}

pub fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn dist_dir_uses_file_stem() {
        let p = Path::new("examples/getting-started/hello_world.sculpt");
        assert_eq!(dist_dir_for_input(p), PathBuf::from("dist/hello_world"));
    }

    #[test]
    fn write_meta_appends_history() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("sculpt_meta_history_{}", stamp));
        fs::create_dir_all(&dir).expect("create temp dir");

        let meta1 = BuildMeta {
            version: 1,
            script: "a.sculpt".to_string(),
            action: "build".to_string(),
            target: "cli".to_string(),
            provider: Some("stub".to_string()),
            model: Some("stub".to_string()),
            llm_ms: Some(10),
            build_ms: Some(20),
            run_ms: None,
            total_ms: 30,
            timestamp_unix_ms: 1,
            status: "ok".to_string(),
            token_usage: None,
        };
        let mut meta2 = meta1.clone();
        meta2.action = "run".to_string();
        meta2.timestamp_unix_ms = 2;

        write_build_meta(&dir, &meta1).expect("write meta1");
        write_build_meta(&dir, &meta2).expect("write meta2");
        let history = read_build_history(&dir);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].action, "build");
        assert_eq!(history[1].action, "run");

        let _ = fs::remove_dir_all(&dir);
    }
}

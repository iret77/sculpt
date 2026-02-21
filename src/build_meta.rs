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

pub fn write_build_meta(dist_dir: &Path, meta: &BuildMeta) -> Result<()> {
    fs::create_dir_all(dist_dir)?;
    let json = serde_json::to_string_pretty(meta)?;
    fs::write(meta_path(dist_dir), json)?;
    Ok(())
}

pub fn read_build_meta(dist_dir: &Path) -> Option<BuildMeta> {
    let data = fs::read_to_string(meta_path(dist_dir)).ok()?;
    serde_json::from_str(&data).ok()
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

    #[test]
    fn dist_dir_uses_file_stem() {
        let p = Path::new("examples/getting-started/hello_world.sculpt");
        assert_eq!(dist_dir_for_input(p), PathBuf::from("dist/hello_world"));
    }
}

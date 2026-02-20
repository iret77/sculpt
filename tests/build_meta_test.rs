use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use sculpt::build_meta::{meta_path, read_build_meta, write_build_meta, BuildMeta, TokenUsage};

fn temp_dir() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("sculpt_meta_test_{}", stamp));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn writes_and_reads_build_meta_json() {
    let dir = temp_dir();
    let meta = BuildMeta {
        version: 1,
        script: "examples/getting-started/hello_world.sculpt".to_string(),
        action: "build".to_string(),
        target: "cli".to_string(),
        provider: Some("gemini".to_string()),
        model: Some("gemini-2.5-pro".to_string()),
        llm_ms: Some(1200),
        build_ms: Some(500),
        run_ms: None,
        total_ms: 1900,
        timestamp_unix_ms: 123456,
        status: "ok".to_string(),
        token_usage: Some(TokenUsage {
            input_tokens: Some(100),
            output_tokens: Some(200),
            total_tokens: Some(300),
        }),
    };

    write_build_meta(&dir, &meta).expect("write");
    assert!(meta_path(&dir).exists());

    let loaded = read_build_meta(&dir).expect("read");
    assert_eq!(loaded.script, "examples/getting-started/hello_world.sculpt");
    assert_eq!(loaded.action, "build");
    assert_eq!(loaded.target, "cli");
    assert_eq!(loaded.provider.as_deref(), Some("gemini"));
    assert_eq!(loaded.model.as_deref(), Some("gemini-2.5-pro"));
    assert_eq!(loaded.llm_ms, Some(1200));
    assert_eq!(loaded.build_ms, Some(500));
    assert_eq!(loaded.total_ms, 1900);

    let _ = fs::remove_dir_all(dir);
}

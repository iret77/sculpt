use std::fs;
use std::path::{Path, PathBuf};

fn collect_sculpt_files(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(root) {
        Ok(v) => v,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_sculpt_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("sculpt") {
            out.push(path);
        }
    }
}

fn has_unqualified_data_calls(src: &str) -> bool {
    const NAMES: &[&str] = &[
        "csvRead",
        "rowCount",
        "csvHasColumns",
        "csvMissingColumns",
        "schemaErrorMessage",
        "reconcileInvoices",
        "metric",
        "buildExceptions",
        "buildReportJson",
        "processingMs",
        "writeJson",
        "sortBy",
        "writeCsv",
        "summaryLine",
    ];
    NAMES.iter().any(|name| {
        let needle = format!("{}(", name);
        let mut pos = 0usize;
        while let Some(idx) = src[pos..].find(&needle) {
            let abs = pos + idx;
            let prev = src[..abs].chars().next_back();
            let is_qualified = prev == Some('.');
            if !is_qualified {
                return true;
            }
            pos = abs + needle.len();
        }
        false
    })
}

#[test]
fn examples_use_namespaced_data_calls() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = repo.join("examples");
    let mut files = Vec::new();
    collect_sculpt_files(&root, &mut files);
    files.sort();

    for file in files {
        let content = fs::read_to_string(&file).expect("read example");
        assert!(
            !has_unqualified_data_calls(&content),
            "example contains unqualified data call: {}",
            file.display()
        );
    }
}

#[test]
fn examples_keep_data_use_and_calls_in_sync() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = repo.join("examples");
    let mut files = Vec::new();
    collect_sculpt_files(&root, &mut files);
    files.sort();

    for file in files {
        let content = fs::read_to_string(&file).expect("read example");
        let has_data_use = content.contains("use(cli.data) as data")
            || content.contains("use(gui.data) as data")
            || content.contains("use(web.data) as data");
        let has_data_call = content.contains("data.");
        assert_eq!(
            has_data_use,
            has_data_call,
            "data namespace mismatch in example: {}",
            file.display()
        );
    }
}

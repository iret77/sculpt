use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn sculpt_bin() -> &'static str {
    env!("CARGO_BIN_EXE_sculpt")
}

fn temp_dir(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_millis();
    let dir = std::env::temp_dir().join(format!("sculpt_{name}_{stamp}"));
    fs::create_dir_all(&dir).expect("mkdir");
    dir
}

#[test]
fn target_stacks_lists_web_adapters() {
    let out = Command::new(sculpt_bin())
        .args(["target", "stacks", "--target", "web"])
        .output()
        .expect("run");
    assert!(out.status.success(), "stdout={} stderr={}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("builtin.web.standard@1"));
    assert!(s.contains("provider.web.next@1"));
    assert!(s.contains("provider.web.laravel@1"));
}

#[test]
fn build_resolves_recursive_imports() {
    let dir = temp_dir("recursive_import");
    let nested = dir.join("modules").join("nested");
    fs::create_dir_all(&nested).expect("mkdir nested");

    fs::write(
        dir.join("modules").join("shared.sculpt"),
        r#"module(Company.Shared):
  import("nested/deep.sculpt") as Deep
  state():
    root = 1
  end
end
"#,
    )
    .expect("write shared");

    fs::write(
        nested.join("deep.sculpt"),
        r#"module(Company.Deep):
  state():
    leaf = 1
  end
end
"#,
    )
    .expect("write deep");

    let main = dir.join("main.sculpt");
    fs::write(
        &main,
        r#"@meta target=cli
module(App.Main):
  import("modules/shared.sculpt") as Shared
  use(cli.ui)
  use(cli.input) as input
  flow(Main):
    start > A
    state(A):
      ui.text("ok", color: "white")
      value = Shared.root
      on input.key(Esc) > Exit
    end
    state(Exit):
      terminate
    end
  end
end
"#,
    )
    .expect("write main");

    let out = Command::new(sculpt_bin())
        .arg("build")
        .arg(&main)
        .args(["--target", "cli", "--provider", "stub"])
        .current_dir(&dir)
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

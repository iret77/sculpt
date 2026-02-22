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
    assert!(
        out.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
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
  import(Company.Deep) as Deep
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
  import(Company.Shared) as Shared
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

    let project = dir.join("recursive.sculpt.json");
    fs::write(
        &project,
        r#"{
  "name": "recursive",
  "entry": "App.Main",
  "modules": [
    "main.sculpt",
    "modules/shared.sculpt",
    "modules/nested/deep.sculpt"
  ]
}
"#,
    )
    .expect("write project");

    let out = Command::new(sculpt_bin())
        .arg("build")
        .arg(&project)
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

#[test]
fn standalone_script_with_import_fails() {
    let dir = temp_dir("standalone_import_fail");
    let main = dir.join("main.sculpt");
    fs::write(
        &main,
        r#"@meta target=cli
module(App.Main):
  import(Company.Shared) as Shared
  flow(Main):
    start > A
    state(A):
      on input.key(esc) > A
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
    assert!(!out.status.success(), "build unexpectedly succeeded");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Imports require a project file"));
}

#[test]
fn project_create_with_glob_creates_project_file() {
    let dir = temp_dir("project_create_glob");
    fs::write(
        dir.join("a.sculpt"),
        r#"module(App.A):
  state():
    x = 1
  end
end
"#,
    )
    .expect("write a");
    fs::write(
        dir.join("b.sculpt"),
        r#"module(App.B):
  state():
    y = 2
  end
end
"#,
    )
    .expect("write b");

    let out = Command::new(sculpt_bin())
        .args([
            "project",
            "create",
            "demo",
            "-p",
            dir.to_string_lossy().as_ref(),
            "-f",
            "*.sculpt",
        ])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let project_file = dir.join("demo.sculpt.json");
    assert!(project_file.exists(), "project file missing");
    let text = fs::read_to_string(project_file).expect("read project");
    assert!(text.contains("\"name\": \"demo\""));
    assert!(text.contains("\"entry\": \"App.A\""));
    assert!(text.contains("\"a.sculpt\""));
    assert!(text.contains("\"b.sculpt\""));
}

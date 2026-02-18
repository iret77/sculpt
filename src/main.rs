use anyhow::Result;

fn main() -> Result<()> {
  // (C) 2026 byte5 GmbH
  let args: Vec<String> = std::env::args().collect();
  if args.len() == 2 && args[1] == "help" {
    print_help_tui();
    return Ok(());
  }
  if args.len() == 1 {
    return sculpt::tui::run();
  }
  sculpt::cli::run()
}

fn print_help_tui() {
  let c = "\x1b[0m";
  let dim = "\x1b[38;2;150;160;170m";
  let accent = "\x1b[1;38;2;0;255;255m"; // byte5 cyan
  let accent2 = "\x1b[1;38;2;234;81;114m"; // byte5 pink

  println!();
  let left_plain = format!("SCULPT Compiler {}", env!("CARGO_PKG_VERSION"));
  let right_plain = "(C) 2026 byte5 GmbH";
  let content_width: usize = 64;
  let spacer = " ".repeat(content_width.saturating_sub(left_plain.len() + right_plain.len()));
  println!(
    "{accent2}SCULPT{c} {accent}Compiler {version}{c}{spacer}{dim}{right}{c}",
    version = env!("CARGO_PKG_VERSION"),
    right = right_plain
  );
  print_box(
    "Usage",
    &[" sculpt <command> [options]", " sculpt help <command>"],
    accent2,
    c,
  );
  print_box(
    "Commands",
    &[
      " examples   write curated examples into ./examples",
      " build      compile .sculpt to target output",
      " freeze     compile + lock deterministic output",
      " replay     build from sculpt.lock (no LLM)",
      " run        run last build output",
      " target     list/describe targets",
      " auth       provider auth check",
    ],
    accent2,
    c,
  );
  print_box(
    "Options",
    &[" -h, --help        show built-in help", " -V, --version     show version"],
    accent2,
    c,
  );
  print_box(
    "Quickstart",
    &[
      " sculpt examples",
      " sculpt build app.sculpt --target gui",
      " sculpt run app.sculpt",
    ],
    accent2,
    c,
  );
  print_box(
    "Tips",
    &[
      " TUI: run `sculpt` with no arguments",
      " Debug: add --debug=compact|raw|all|json",
    ],
    accent2,
    c,
  );
}

fn print_box(title: &str, lines: &[&str], accent2: &str, c: &str) {
  const WIDTH: usize = 62;
  println!("┌{}┐", "─".repeat(WIDTH));
  print_padded(&format!(" {accent2}{title}{c}"), WIDTH);
  for line in lines {
    print_padded(line, WIDTH);
  }
  println!("└{}┘", "─".repeat(WIDTH));
}

fn print_padded(line: &str, width: usize) {
  let len = visible_len(line);
  let pad = width.saturating_sub(len);
  println!("│{}{}│", line, " ".repeat(pad));
}

fn visible_len(s: &str) -> usize {
  let mut len = 0;
  let mut chars = s.chars().peekable();
  while let Some(ch) = chars.next() {
    if ch == '\x1b' {
      while let Some(next) = chars.next() {
        if next == 'm' {
          break;
        }
      }
      continue;
    }
    len += 1;
  }
  len
}

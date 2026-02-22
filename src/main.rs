use anyhow::Result;

fn main() -> Result<()> {
    // (C) 2026 byte5 GmbH
    let args: Vec<String> = std::env::args().collect();
    if maybe_print_custom_version(&args) {
        return Ok(());
    }
    if maybe_print_custom_help(&args) {
        return Ok(());
    }
    if args.len() == 1 {
        return sculpt::tui::run();
    }
    sculpt::cli::run()
}

fn maybe_print_custom_version(args: &[String]) -> bool {
    if args.len() == 2 && matches!(args[1].as_str(), "--version" | "-V") {
        println!("sculpt {}", env!("CARGO_PKG_VERSION"));
        println!("{}", sculpt::versioning::language_line());
        return true;
    }
    false
}

fn maybe_print_custom_help(args: &[String]) -> bool {
    if args.len() == 2 && matches!(args[1].as_str(), "help" | "--help" | "-h") {
        print_help_tui();
        return true;
    }
    if args.len() == 3 && matches!(args[2].as_str(), "--help" | "-h") {
        return print_subcommand_help(&args[1]);
    }
    if args.len() == 3 && args[1] == "help" {
        return print_subcommand_help(&args[2]);
    }
    false
}

fn print_help_tui() {
    print_header();
    let c = "\x1b[0m";
    let accent2 = "\x1b[1;38;2;234;81;114m"; // byte5 pink
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
            " project    create/manage .sculpt.json project files",
            " gate       evaluate release quality gates",
            " build      compile .sculpt or .sculpt.json to target output",
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
        &[
            " -h, --help        show built-in help",
            " -V, --version     show version",
            " --debug=...       build/freeze option (see command help)",
        ],
        accent2,
        c,
    );
    print_box(
        "Language",
        &[&format!(" {}", sculpt::versioning::language_line())],
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
            " Command details: sculpt <command> --help",
        ],
        accent2,
        c,
    );
}

fn print_subcommand_help(cmd: &str) -> bool {
    let c = "\x1b[0m";
    let accent2 = "\x1b[1;38;2;234;81;114m"; // byte5 pink
    match cmd {
        "build" => {
            print_header();
            print_box(
                "Usage",
                &[" sculpt build <input.sculpt|project.sculpt.json> [--target <cli|gui|web>] [options]"],
                accent2,
                c,
            );
            print_box(
                "Options",
                &[
                    " --target <name>         override target (meta target is preferred)",
                    " --nd-policy <strict>    override nd_policy meta",
                    " --provider <name>       llm provider (default from config)",
                    " --model <name>          model override",
                    " --strict-provider       fail if provider auth/config is missing",
                    " --debug[=compact|raw|all|json]",
                ],
                accent2,
                c,
            );
            print_box(
                "Examples",
                &[
                    " sculpt build examples/getting-started/hello_world.sculpt --target cli",
                    " sculpt build examples/getting-started/native_window.sculpt --target gui",
                ],
                accent2,
                c,
            );
            true
        }
        "project" => {
            print_header();
            print_box(
                "Usage",
                &[" sculpt project create <name> [-p <path>] [-f <files> ...]"],
                accent2,
                c,
            );
            print_box(
                "Options",
                &[
                    " -p, --path <dir>        base directory (default: current dir)",
                    " -f, --files <glob...>   module files or glob patterns",
                ],
                accent2,
                c,
            );
            print_box(
                "Examples",
                &[
                    " sculpt project create app -p . -f \"*.sculpt\"",
                    " sculpt project create billing -p examples/business -f \"modules/*.sculpt\" -f modular_invoice_app.sculpt",
                ],
                accent2,
                c,
            );
            true
        }
        "gate" => {
            print_header();
            print_box("Usage", &[" sculpt gate check <gate.json>"], accent2, c);
            print_box(
                "Behavior",
                &[
                    " Evaluates pre-registered quality criteria from JSON gate files.",
                    " Exits non-zero if one or more criteria fail.",
                ],
                accent2,
                c,
            );
            print_box(
                "Example",
                &[" sculpt gate check poc/gates/incident_triage_vibe_gate.json"],
                accent2,
                c,
            );
            true
        }
        "freeze" => {
            print_header();
            print_box(
                "Usage",
                &[" sculpt freeze <input.sculpt|project.sculpt.json> [--target <cli|gui|web>] [options]"],
                accent2,
                c,
            );
            print_box(
                "Options",
                &[
                    " --target <name>         override target",
                    " --nd-policy <strict>    override nd_policy meta",
                    " --provider <name>       llm provider (default from config)",
                    " --model <name>          model override",
                    " --strict-provider       fail if provider auth/config is missing",
                    " --debug[=compact|raw|all|json]",
                ],
                accent2,
                c,
            );
            print_box(
                "Behavior",
                &[
                    " Generates target IR and writes sculpt.lock for deterministic replay.",
                    " Build artifacts are written to dist/<script_name>/.",
                ],
                accent2,
                c,
            );
            true
        }
        "replay" => {
            print_header();
            print_box(
                "Usage",
                &[" sculpt replay <input.sculpt|project.sculpt.json> [--target <cli|gui|web>]"],
                accent2,
                c,
            );
            print_box(
                "Behavior",
                &[
                    " Rebuilds deterministically from sculpt.lock (no LLM call).",
                    " Fails if lock data is missing/incompatible.",
                ],
                accent2,
                c,
            );
            true
        }
        "run" => {
            print_header();
            print_box(
                "Usage",
                &[" sculpt run <input.sculpt|project.sculpt.json> [--target <cli|gui|web>]"],
                accent2,
                c,
            );
            print_box(
                "Behavior",
                &[
                    " Runs the last successful build for the selected script.",
                    " Reads artifacts from dist/<script_name>/.",
                ],
                accent2,
                c,
            );
            true
        }
        "target" => {
            print_header();
            print_box(
                "Usage",
                &[
                    " sculpt target list",
                    " sculpt target describe --target <name>",
                    " sculpt target packages --target <name>",
                    " sculpt target exports --target <name> --package <id>",
                    " sculpt target stacks --target <name>",
                ],
                accent2,
                c,
            );
            print_box(
                "Behavior",
                &[
                    " list     : available targets",
                    " describe : full target contract + schema",
                    " packages : provider packages and exposed namespaces",
                    " exports  : symbols exported by one package",
                    " stacks   : stack adapter profiles for the target",
                ],
                accent2,
                c,
            );
            true
        }
        "clean" => {
            print_header();
            print_box(
                "Usage",
                &[
                    " sculpt clean <input.sculpt|project.sculpt.json>",
                    " sculpt clean --all",
                ],
                accent2,
                c,
            );
            print_box(
                "Behavior",
                &[
                    " clean <input>: removes dist/<script_name>/",
                    " clean --all  : removes entire dist/ directory",
                ],
                accent2,
                c,
            );
            true
        }
        _ => false,
    }
}

fn print_header() {
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
    println!("{}{}{}", dim, sculpt::versioning::language_line(), c);
}

fn print_box(title: &str, lines: &[&str], accent2: &str, c: &str) {
    let title_len = visible_len(title) + 1;
    let max_line_len = lines
        .iter()
        .map(|line| visible_len(line))
        .max()
        .unwrap_or(0);
    let width = usize::max(62, usize::max(title_len, max_line_len));
    println!("┌{}┐", "─".repeat(width));
    print_padded(&format!(" {accent2}{title}{c}"), width);
    for line in lines {
        print_padded(line, width);
    }
    println!("└{}┘", "─".repeat(width));
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

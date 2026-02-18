use anyhow::Result;

fn main() -> Result<()> {
  // (C) 2026 byte5 GmbH
  if std::env::args().len() == 1 {
    return sculpt::tui::run();
  }
  sculpt::cli::run()
}

use std::cmp::min;
use std::fs;
use std::io::stdout;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Terminal;

use crate::targets::list_targets;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
  Files,
  Targets,
}

struct Entry {
  name: String,
  path: PathBuf,
  is_dir: bool,
  is_sculpt: bool,
}

struct Theme {
  panel_bg: Color,
  fg: Color,
  dim: Color,
  accent: Color,
  accent2: Color,
  highlight_bg: Color,
}

impl Theme {
  fn dark() -> Self {
    Self {
      panel_bg: Color::Rgb(24, 26, 31),
      fg: Color::Rgb(220, 224, 230),
      dim: Color::Rgb(140, 148, 160),
      accent: Color::Rgb(106, 164, 236),
      accent2: Color::Rgb(123, 192, 170),
      highlight_bg: Color::Rgb(36, 40, 48),
    }
  }
}

struct AppState {
  cwd: PathBuf,
  entries: Vec<Entry>,
  file_state: ListState,
  targets: Vec<String>,
  target_state: ListState,
  focus: Focus,
  log: Vec<String>,
  status: String,
  selected_file: Option<PathBuf>,
  meta_target: Option<String>,
  last_refresh: Instant,
  theme: Theme,
}

pub fn run() -> Result<()> {
  enable_raw_mode()?;
  let mut stdout = stdout();
  execute!(stdout, EnterAlternateScreen)?;
  terminal::enable_raw_mode()?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let mut state = AppState::new()?;

  let res = loop {
    terminal.draw(|f| ui(f, &mut state))?;

    if event::poll(Duration::from_millis(200))? {
      if let Event::Key(key) = event::read()? {
        if handle_key(&mut state, key)? {
          break Ok(());
        }
      }
    }

    if state.last_refresh.elapsed() > Duration::from_secs(2) {
      state.refresh_targets()?;
      state.last_refresh = Instant::now();
    }
  };

  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
  terminal.show_cursor()?;

  res
}

impl AppState {
  fn new() -> Result<Self> {
    let cwd = std::env::current_dir()?;
    let entries = read_entries(&cwd)?;
    let targets = list_targets().unwrap_or_else(|_| vec!["cli".to_string(), "gui".to_string(), "web".to_string()]);
    let mut file_state = ListState::default();
    file_state.select(Some(0));
    let mut target_state = ListState::default();
    target_state.select(Some(0));

    Ok(Self {
      cwd,
      entries,
      file_state,
      targets,
      target_state,
      focus: Focus::Files,
      log: vec!["SCULPT TUI ready".to_string()],
      status: "Ready".to_string(),
      selected_file: None,
      meta_target: None,
      last_refresh: Instant::now(),
      theme: Theme::dark(),
    })
  }

  fn refresh_entries(&mut self) -> Result<()> {
    self.entries = read_entries(&self.cwd)?;
    let idx = self.file_state.selected().unwrap_or(0);
    let idx = min(idx, self.entries.len().saturating_sub(1));
    self.file_state.select(Some(idx));
    Ok(())
  }

  fn refresh_targets(&mut self) -> Result<()> {
    let targets = list_targets().unwrap_or_else(|_| vec!["cli".to_string(), "gui".to_string(), "web".to_string()]);
    self.targets = targets;
    if let Some(meta) = &self.meta_target {
      if let Some(idx) = self.targets.iter().position(|t| t == meta) {
        self.target_state.select(Some(idx));
      }
    } else {
      let idx = self.target_state.selected().unwrap_or(0);
      self.target_state.select(Some(min(idx, self.targets.len().saturating_sub(1))));
    }
    Ok(())
  }

  fn set_selected_file(&mut self, path: PathBuf) -> Result<()> {
    self.selected_file = Some(path.clone());
    self.meta_target = extract_meta_target(&path)?;
    if let Some(meta) = &self.meta_target {
      if let Some(idx) = self.targets.iter().position(|t| t == meta) {
        self.target_state.select(Some(idx));
      }
    }
    Ok(())
  }

  fn active_target(&self) -> Option<String> {
    if let Some(meta) = &self.meta_target {
      return Some(meta.clone());
    }
    self.target_state.selected().and_then(|i| self.targets.get(i).cloned())
  }

  fn can_run(&self) -> bool {
    let Some(target) = self.active_target() else { return false; };
    match target.as_str() {
      "cli" => Path::new("dist/main.js").exists(),
      "web" => Path::new("dist/index.html").exists(),
      "gui" => Path::new("dist/gui/.build/release/SculptGui").exists(),
      _ => true,
    }
  }

  fn run_command(&mut self, cmd: &str, args: &[String]) -> Result<()> {
    self.status = format!("Running: {} {}", cmd, args.join(" "));
    let output = Command::new(cmd).args(args).output()?;
    self.log.push(format!("$ {} {}", cmd, args.join(" ")));
    if !output.stdout.is_empty() {
      let out = String::from_utf8_lossy(&output.stdout);
      for line in out.lines() {
        self.log.push(line.to_string());
      }
    }
    if !output.stderr.is_empty() {
      let out = String::from_utf8_lossy(&output.stderr);
      for line in out.lines() {
        self.log.push(line.to_string());
      }
    }
    if !output.status.success() {
      self.status = format!("Failed: status {:?}", output.status.code());
      bail!("Command failed");
    }
    self.status = "Ready".to_string();
    Ok(())
  }
}

fn handle_key(state: &mut AppState, key: KeyEvent) -> Result<bool> {
  match key.code {
    KeyCode::Char('q') => return Ok(true),
    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
    KeyCode::Tab => {
      state.focus = if state.focus == Focus::Files { Focus::Targets } else { Focus::Files };
    }
    KeyCode::Up => match state.focus {
      Focus::Files => move_selection(&mut state.file_state, state.entries.len(), -1),
      Focus::Targets => move_selection(&mut state.target_state, state.targets.len(), -1),
    },
    KeyCode::Down => match state.focus {
      Focus::Files => move_selection(&mut state.file_state, state.entries.len(), 1),
      Focus::Targets => move_selection(&mut state.target_state, state.targets.len(), 1),
    },
    KeyCode::Enter => {
      if state.focus == Focus::Files {
        if let Some(idx) = state.file_state.selected() {
          if let Some(entry) = state.entries.get(idx) {
            if entry.is_dir {
              state.cwd = entry.path.clone();
              state.refresh_entries()?;
            } else if entry.is_sculpt {
              state.set_selected_file(entry.path.clone())?;
            }
          }
        }
      }
    }
    KeyCode::Backspace => {
      if let Some(parent) = state.cwd.parent() {
        state.cwd = parent.to_path_buf();
        state.refresh_entries()?;
      }
    }
    KeyCode::Char('b') => {
      if let Some(file) = &state.selected_file {
        let target = state.active_target();
        let mut args = vec!["build".to_string(), file.to_string_lossy().to_string()];
        if let Some(t) = target {
          args.push("--target".to_string());
          args.push(t);
        }
        let _ = state.run_command("sculpt", &args);
      }
    }
    KeyCode::Char('f') => {
      if let Some(file) = &state.selected_file {
        let target = state.active_target();
        let mut args = vec!["freeze".to_string(), file.to_string_lossy().to_string()];
        if let Some(t) = target {
          args.push("--target".to_string());
          args.push(t);
        }
        let _ = state.run_command("sculpt", &args);
      }
    }
    KeyCode::Char('p') => {
      if let Some(file) = &state.selected_file {
        let target = state.active_target();
        let mut args = vec!["replay".to_string(), file.to_string_lossy().to_string()];
        if let Some(t) = target {
          args.push("--target".to_string());
          args.push(t);
        }
        let _ = state.run_command("sculpt", &args);
      }
    }
    KeyCode::Char('r') => {
      if state.can_run() {
        if let Some(file) = &state.selected_file {
          let target = state.active_target();
          let mut args = vec!["run".to_string(), file.to_string_lossy().to_string()];
          if let Some(t) = target {
            args.push("--target".to_string());
            args.push(t);
          }
          let _ = state.run_command("sculpt", &args);
        }
      }
    }
    _ => {}
  }
  Ok(false)
}

fn move_selection(state: &mut ListState, len: usize, delta: i32) {
  let len = len.saturating_sub(1);
  let idx = state.selected().unwrap_or(0) as i32;
  let next = (idx + delta).clamp(0, len as i32) as usize;
  state.select(Some(next));
}

fn ui(f: &mut ratatui::Frame, state: &mut AppState) {
  let size = f.size();
  let layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(5)].as_ref())
    .split(size);

  let header = Paragraph::new(vec![
    Line::from(vec![
      Span::styled("SCULPT", Style::default().fg(state.theme.accent).add_modifier(Modifier::BOLD)),
      Span::raw("  —  TUI"),
      Span::raw("   "),
      Span::styled(state.status.as_str(), Style::default().fg(state.theme.accent2)),
    ]),
    Line::from(vec![
      Span::raw("Dir: "),
      Span::styled(state.cwd.to_string_lossy(), Style::default().fg(state.theme.fg)),
    ]),
  ])
  .block(Block::default().borders(Borders::ALL).style(Style::default().bg(state.theme.panel_bg)));
  f.render_widget(header, layout[0]);

  let body = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(55), Constraint::Percentage(45)].as_ref())
    .split(layout[1]);

  let files = list_files(state);
  let mut file_state = state.file_state.clone();
  f.render_stateful_widget(files, body[0], &mut file_state);
  state.file_state = file_state;

  let right = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(9), Constraint::Min(5)].as_ref())
    .split(body[1]);

  let targets = list_targets_widget(state);
  let mut target_state = state.target_state.clone();
  f.render_stateful_widget(targets, right[0], &mut target_state);
  state.target_state = target_state;

  let log = Paragraph::new(state.log.iter().rev().take(12).rev().map(|l| Line::raw(l.clone())).collect::<Vec<_>>())
    .block(Block::default().borders(Borders::ALL).title("Log").style(Style::default().bg(state.theme.panel_bg)))
    .wrap(Wrap { trim: true });
  f.render_widget(log, right[1]);

  let footer = Paragraph::new(vec![
    Line::from(vec![
      Span::styled("Keys: ", Style::default().fg(state.theme.dim)),
      Span::raw("↑↓ navigate  Tab switch  Enter open/select  Backspace up  "),
      Span::styled("B", Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD)),
      Span::raw(" build  "),
      Span::styled("R", Style::default().fg(if state.can_run() { state.theme.accent2 } else { state.theme.dim }).add_modifier(Modifier::BOLD)),
      Span::raw(" run  "),
      Span::styled("F", Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD)),
      Span::raw(" freeze  "),
      Span::styled("P", Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD)),
      Span::raw(" replay  Q quit"),
    ]),
  ])
  .block(Block::default().borders(Borders::ALL).style(Style::default().bg(state.theme.panel_bg)));
  f.render_widget(footer, layout[2]);
}

fn list_files(state: &AppState) -> List<'_> {
  let items: Vec<ListItem> = state
    .entries
    .iter()
    .map(|e| {
      let icon = if e.is_dir { "[D]" } else if e.is_sculpt { "[S]" } else { "   " };
      let name = format!("{} {}", icon, e.name);
      ListItem::new(Line::from(Span::raw(name)))
    })
    .collect();

  let mut list = List::new(items)
    .block(Block::default().borders(Borders::ALL).title("Files").style(Style::default().bg(state.theme.panel_bg)))
    .highlight_style(Style::default().bg(state.theme.highlight_bg).fg(state.theme.fg).add_modifier(Modifier::BOLD));
  if state.focus != Focus::Files {
    list = list.highlight_style(Style::default().bg(state.theme.panel_bg).fg(state.theme.dim));
  }
  list
}

fn list_targets_widget(state: &AppState) -> List<'_> {
  let items: Vec<ListItem> = state
    .targets
    .iter()
    .map(|t| {
      let tag = if Some(t) == state.meta_target.as_ref() { " (meta)" } else { "" };
      ListItem::new(Line::from(Span::raw(format!("{}{}", t, tag))))
    })
    .collect();

  let title = if state.meta_target.is_some() { "Targets (locked)" } else { "Targets" };
  let mut list = List::new(items)
    .block(Block::default().borders(Borders::ALL).title(title).style(Style::default().bg(state.theme.panel_bg)))
    .highlight_style(Style::default().bg(state.theme.highlight_bg).fg(state.theme.fg).add_modifier(Modifier::BOLD));
  if state.focus != Focus::Targets {
    list = list.highlight_style(Style::default().bg(state.theme.panel_bg).fg(state.theme.dim));
  }
  list
}

fn read_entries(dir: &Path) -> Result<Vec<Entry>> {
  let mut entries = Vec::new();
  entries.push(Entry {
    name: "..".to_string(),
    path: dir.parent().unwrap_or(dir).to_path_buf(),
    is_dir: true,
    is_sculpt: false,
  });
  for entry in fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    let name = entry.file_name().to_string_lossy().to_string();
    let is_dir = path.is_dir();
    let is_sculpt = path.extension().and_then(|s| s.to_str()) == Some("sculpt");
    entries.push(Entry { name, path, is_dir, is_sculpt });
  }
  entries.sort_by(|a, b| {
    if a.is_dir != b.is_dir {
      return b.is_dir.cmp(&a.is_dir);
    }
    a.name.to_lowercase().cmp(&b.name.to_lowercase())
  });
  Ok(entries)
}

fn extract_meta_target(path: &Path) -> Result<Option<String>> {
  let content = fs::read_to_string(path)?;
  for line in content.lines() {
    let line = line.trim();
    if !line.starts_with("@meta") {
      continue;
    }
    let rest = line.trim_start_matches("@meta").trim();
    for part in rest.split_whitespace() {
      if let Some(eq) = part.find('=') {
        let (k, v) = part.split_at(eq);
        if k == "target" {
          let val = v.trim_start_matches('=').trim_matches('"');
          return Ok(Some(val.to_string()));
        }
      }
    }
  }
  Ok(None)
}

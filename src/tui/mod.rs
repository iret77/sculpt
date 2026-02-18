use std::cmp::min;
use std::collections::BTreeMap;
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
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap};
use ratatui::Terminal;

use crate::build_meta::read_build_meta;
use crate::targets::list_targets;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
  Files,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModalFocus {
  Targets,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingAction {
  BuildOnly,
  RunOnly,
  BuildRun,
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
      panel_bg: Color::Rgb(18, 20, 22),  // neutral anthracite
      fg: Color::Rgb(255, 255, 255),     // white
      dim: Color::Rgb(150, 160, 170),
      accent: Color::Rgb(0, 255, 255),   // byte5 cyan
      accent2: Color::Rgb(234, 81, 114), // byte5 pink (accent)
      highlight_bg: Color::Rgb(28, 30, 34),
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
  preview_meta: BTreeMap<String, String>,
  preview_lines: usize,
  preview_size: u64,
  preview_intro: Vec<String>,
  last_run: Option<LastRun>,
  last_refresh: Instant,
  theme: Theme,
  modal_open: bool,
  modal_focus: ModalFocus,
  info_modal: Option<String>,
  pending_action: PendingAction,
}

struct LastRun {
  action: String,
  target: Option<String>,
  duration_ms: u128,
  when: Instant,
  provider: Option<String>,
  model: Option<String>,
  ok: bool,
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
      preview_meta: BTreeMap::new(),
      preview_lines: 0,
      preview_size: 0,
      preview_intro: Vec::new(),
      last_run: None,
      last_refresh: Instant::now(),
      theme: Theme::dark(),
      modal_open: false,
      modal_focus: ModalFocus::Targets,
      info_modal: None,
      pending_action: PendingAction::BuildRun,
    })
  }

  fn refresh_entries(&mut self) -> Result<()> {
    self.entries = read_entries(&self.cwd)?;
    let idx = self.file_state.selected().unwrap_or(0);
    let idx = min(idx, self.entries.len().saturating_sub(1));
    self.file_state.select(Some(idx));
    self.update_preview_from_selection();
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
    let meta = extract_meta(&path)?;
    self.meta_target = meta.get("target").cloned();
    self.preview_meta = meta;
    self.preview_lines = count_lines(&path).unwrap_or(0);
    self.preview_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    self.preview_intro = extract_intro_comment(&path).unwrap_or_default();
    if let Some(meta) = &self.meta_target {
      if let Some(idx) = self.targets.iter().position(|t| t == meta) {
        self.target_state.select(Some(idx));
      }
    }
    Ok(())
  }

  fn update_preview_from_selection(&mut self) {
    let Some(idx) = self.file_state.selected() else { return; };
    let Some(entry) = self.entries.get(idx) else { return; };
    if !entry.is_sculpt {
      self.clear_preview();
      return;
    }
    let _ = self.set_selected_file(entry.path.clone());
  }

  fn clear_preview(&mut self) {
    self.selected_file = None;
    self.meta_target = None;
    self.preview_meta.clear();
    self.preview_lines = 0;
    self.preview_size = 0;
    self.preview_intro.clear();
  }

  fn active_target(&self) -> Option<String> {
    if let Some(meta) = &self.meta_target {
      return Some(meta.clone());
    }
    self.target_state.selected().and_then(|i| self.targets.get(i).cloned())
  }

  fn can_run(&self) -> bool {
    can_run_for_selected(self)
  }

  fn run_command(&mut self, cmd: &str, args: &[String]) -> Result<()> {
    self.status = format!("Running: {} {}", cmd, args.join(" "));
    let started = Instant::now();
    let output = Command::new(cmd).args(args).output()?;
    let duration = started.elapsed();
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
    let ok = output.status.success();
    let (provider, model) = extract_provider_model(&output);
    let target = extract_target(args);
    let action = args.get(0).cloned().unwrap_or_else(|| "run".to_string());
    self.last_run = Some(LastRun {
      action,
      target,
      duration_ms: duration.as_millis(),
      when: Instant::now(),
      provider,
      model,
      ok,
    });
    if !ok {
      self.status = format!("Failed: status {:?}", output.status.code());
      bail!("Command failed");
    }
    self.status = "Ready".to_string();
    Ok(())
  }
}

fn handle_key(state: &mut AppState, key: KeyEvent) -> Result<bool> {
  if state.info_modal.is_some() {
    match key.code {
      KeyCode::Esc | KeyCode::Enter => {
        state.info_modal = None;
      }
      _ => {}
    }
    return Ok(false);
  }
  if state.modal_open {
    return handle_modal_key(state, key);
  }
  match key.code {
    KeyCode::Char('q') => return Ok(true),
    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
    KeyCode::Esc => return Ok(true),
    KeyCode::Up => {
      move_selection(&mut state.file_state, state.entries.len(), -1);
      state.update_preview_from_selection();
    }
    KeyCode::Down => {
      move_selection(&mut state.file_state, state.entries.len(), 1);
      state.update_preview_from_selection();
    }
    KeyCode::Enter => {
      if let Some(idx) = state.file_state.selected() {
        if let Some(entry) = state.entries.get(idx) {
          if entry.is_dir {
            state.cwd = entry.path.clone();
            state.refresh_entries()?;
          } else if entry.is_sculpt {
            state.set_selected_file(entry.path.clone())?;
            state.pending_action = PendingAction::BuildRun;
            if state.meta_target.is_some() {
              execute_pending_action(state)?;
            } else {
              state.modal_open = true;
              state.modal_focus = ModalFocus::Targets;
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
      if state.selected_file.is_some() {
        state.pending_action = PendingAction::BuildOnly;
        if state.meta_target.is_some() {
          execute_pending_action(state)?;
        } else {
          state.modal_open = true;
          state.modal_focus = ModalFocus::Targets;
        }
      }
    }
    KeyCode::Char('r') => {
      if state.selected_file.is_some() {
        state.pending_action = PendingAction::RunOnly;
        if state.meta_target.is_some() {
          execute_pending_action(state)?;
        } else {
          state.modal_open = true;
          state.modal_focus = ModalFocus::Targets;
        }
      }
    }
    KeyCode::Char('f') => {
      if state.selected_file.is_some() {
        let _ = state.run_command("sculpt", &build_args(state, "freeze"));
      }
    }
    KeyCode::Char('p') => {
      if state.selected_file.is_some() {
        let _ = state.run_command("sculpt", &build_args(state, "replay"));
      }
    }
    KeyCode::Char('c') => {
      if state.selected_file.is_some() {
        let _ = state.run_command("sculpt", &build_args(state, "clean"));
        state.update_preview_from_selection();
      } else {
        state.info_modal = Some("Select a .sculpt file first.".to_string());
      }
    }
    _ => {}
  }
  Ok(false)
}

fn handle_modal_key(state: &mut AppState, key: KeyEvent) -> Result<bool> {
  match key.code {
    KeyCode::Esc => {
      state.modal_open = false;
      return Ok(false);
    }
    KeyCode::Up => match state.modal_focus {
      ModalFocus::Targets => move_selection(&mut state.target_state, state.targets.len(), -1),
    },
    KeyCode::Down => match state.modal_focus {
      ModalFocus::Targets => move_selection(&mut state.target_state, state.targets.len(), 1),
    },
    KeyCode::Enter => {
      execute_pending_action(state)?;
    }
    _ => {}
  }
  Ok(false)
}

fn execute_pending_action(state: &mut AppState) -> Result<()> {
  match state.pending_action {
    PendingAction::BuildOnly => {
      let _ = state.run_command("sculpt", &build_args(state, "build"));
    }
    PendingAction::RunOnly => {
      if can_run_for_selected(state) {
        let _ = state.run_command("sculpt", &build_args(state, "run"));
      } else {
        state.info_modal = Some("Run not available. Build first.".to_string());
      }
    }
    PendingAction::BuildRun => {
      if can_run_for_selected(state) {
        let _ = state.run_command("sculpt", &build_args(state, "run"));
      } else {
        let _ = state.run_command("sculpt", &build_args(state, "build"));
        if can_run_for_selected(state) {
          let _ = state.run_command("sculpt", &build_args(state, "run"));
        }
      }
    }
  }
  state.modal_open = false;
  Ok(())
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
    .constraints([Constraint::Length(3), Constraint::Min(15), Constraint::Min(8), Constraint::Length(3)].as_ref())
    .split(size);

  let header_line = header_line(state, layout[0].width);
  let header = Paragraph::new(vec![header_line])
  .block(
    Block::default()
      .borders(Borders::ALL)
      .padding(Padding { left: 1, right: 1, top: 0, bottom: 0 })
      .style(Style::default().bg(state.theme.panel_bg)),
  );
  f.render_widget(header, layout[0]);

  let body = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(55), Constraint::Percentage(45)].as_ref())
    .split(layout[1]);

  let files = list_files(state);
  let mut file_state = state.file_state.clone();
  f.render_stateful_widget(files, body[0], &mut file_state);
  state.file_state = file_state;

  let details = Paragraph::new(render_details(state))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("[Details]", Style::default().fg(state.theme.fg)))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(state.theme.panel_bg)),
    )
    .wrap(Wrap { trim: true });
  f.render_widget(details, body[1]);

  let log = Paragraph::new(render_log_lines(state))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("[Log]", Style::default().fg(state.theme.fg)))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(state.theme.panel_bg)),
    )
    .wrap(Wrap { trim: true });
  f.render_widget(log, layout[2]);

  let footer = Paragraph::new(vec![
    Line::from(vec![
      Span::styled("↑↓", Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD)),
      Span::raw(" move  "),
      Span::styled("Enter", Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD)),
      Span::raw(" run/build  "),
      Span::styled("B", Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD)),
      Span::raw(" build  "),
      Span::styled("R", Style::default().fg(if state.can_run() { state.theme.accent2 } else { state.theme.dim }).add_modifier(Modifier::BOLD)),
      Span::raw(" run  "),
      Span::styled("Esc", Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD)),
      Span::raw(" exit  "),
      Span::styled("F", Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD)),
      Span::raw(" freeze  "),
      Span::styled("P", Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD)),
      Span::raw(" replay  "),
      Span::styled("C", Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD)),
      Span::raw(" clean"),
    ]),
  ])
  .block(
    Block::default()
      .borders(Borders::ALL)
      .padding(Padding::horizontal(1))
      .style(Style::default().bg(state.theme.panel_bg)),
  );
  f.render_widget(footer, layout[3]);

  if state.modal_open {
    render_modal(f, state);
  }
  if let Some(msg) = state.info_modal.clone() {
    render_info_modal(f, state, &msg);
  }
}

fn list_files(state: &AppState) -> List<'_> {
  let items: Vec<ListItem> = state
    .entries
    .iter()
    .map(|e| {
      let icon = if e.is_dir { "[D]" } else if e.is_sculpt { "[S]" } else { "   " };
      let icon_span = if e.is_sculpt {
        Span::styled(icon, Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD))
      } else {
        Span::styled(icon, Style::default().fg(state.theme.dim))
      };
      let name_span = Span::raw(format!(" {}", e.name));
      ListItem::new(Line::from(vec![icon_span, name_span]))
    })
    .collect();

  let mut list = List::new(items)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("[Files]", Style::default().fg(state.theme.fg)))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(state.theme.panel_bg)),
    )
    .highlight_style(Style::default().bg(state.theme.highlight_bg).fg(state.theme.fg).add_modifier(Modifier::BOLD));
  if state.focus != Focus::Files {
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

fn extract_meta(path: &Path) -> Result<BTreeMap<String, String>> {
  let content = fs::read_to_string(path)?;
  let mut meta = BTreeMap::new();
  for line in content.lines() {
    let line = line.trim();
    if !line.starts_with("@meta") {
      continue;
    }
    let rest = line.trim_start_matches("@meta").trim();
    for part in rest.split_whitespace() {
      if let Some(eq) = part.find('=') {
        let (k, v) = part.split_at(eq);
        let val = v.trim_start_matches('=').trim_matches('"');
        meta.insert(k.to_string(), val.to_string());
      }
    }
  }
  Ok(meta)
}

fn extract_intro_comment(path: &Path) -> Result<Vec<String>> {
  let content = fs::read_to_string(path)?;
  let mut lines = Vec::new();
  for raw in content.lines() {
    let line = raw.trim();
    if line.is_empty() {
      continue;
    }
    if line.starts_with("@meta") {
      continue;
    }
    if line.starts_with('#') || line.starts_with(';') || line.starts_with("//") {
      let cleaned = line.trim_start_matches("//").trim_start_matches('#').trim_start_matches(';').trim();
      lines.push(cleaned.to_string());
      continue;
    }
    break;
  }
  Ok(lines)
}

fn count_lines(path: &Path) -> Result<usize> {
  let content = fs::read_to_string(path)?;
  Ok(content.lines().count())
}

fn render_details(state: &AppState) -> Vec<Line<'_>> {
  let mut lines = Vec::new();
  if let Some(path) = &state.selected_file {
    lines.push(Line::from(vec![Span::styled(
      path.file_name().unwrap_or_default().to_string_lossy(),
      Style::default().fg(state.theme.fg).add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![
      Span::styled("Lines: ", Style::default().fg(state.theme.dim)),
      Span::styled(state.preview_lines.to_string(), Style::default().fg(state.theme.fg)),
      Span::raw("  "),
      Span::styled("Size: ", Style::default().fg(state.theme.dim)),
      Span::styled(format!("{} bytes", state.preview_size), Style::default().fg(state.theme.fg)),
    ]));
    let target = state
      .preview_meta
      .get("target")
      .cloned()
      .unwrap_or_else(|| "auto".to_string());
    let layout = state
      .preview_meta
      .get("layout")
      .cloned()
      .unwrap_or_else(|| "default".to_string());
    let dist_dir = dist_dir_for(path);
    let has_cli = dist_dir.join("main.js").exists();
    let has_web = dist_dir.join("index.html").exists();
    let has_gui = dist_dir.join("gui/.build/release/SculptGui").exists();
    let has_lock = Path::new("sculpt.lock").exists();
    let build_ok = match target.as_str() {
      "cli" => has_cli,
      "web" => has_web,
      "gui" => has_gui,
      _ => has_cli || has_web || has_gui,
    };
    let build_box = if build_ok { "[X]" } else { "[ ]" };
    let lock_box = if has_lock { "[X]" } else { "[ ]" };
    lines.push(Line::from(vec![
      Span::styled(format!("{} build", build_box), Style::default().fg(state.theme.fg)),
      Span::raw("   "),
      Span::styled(format!("{} lock", lock_box), Style::default().fg(state.theme.fg)),
      Span::raw("    "),
      Span::styled("target ", Style::default().fg(state.theme.dim)),
      Span::styled(target, Style::default().fg(state.theme.fg)),
      Span::raw("   "),
      Span::styled("layout ", Style::default().fg(state.theme.dim)),
      Span::styled(layout, Style::default().fg(state.theme.fg)),
    ]));
    if !state.preview_intro.is_empty() {
      lines.push(Line::from(""));
      lines.push(Line::from(Span::styled("Intro:", Style::default().fg(state.theme.dim))));
      for line in &state.preview_intro {
        lines.push(Line::from(Span::styled(format!("  {}", line), Style::default().fg(state.theme.fg))));
      }
    }
    if let Some(meta) = read_build_meta(&dist_dir) {
      lines.push(Line::from(""));
      lines.push(Line::from(Span::styled("Last Build:", Style::default().fg(state.theme.dim))));
      lines.push(Line::from(vec![
        Span::styled("  action ", Style::default().fg(state.theme.dim)),
        Span::styled(meta.action, Style::default().fg(state.theme.fg)),
        Span::raw("  "),
        Span::styled("target ", Style::default().fg(state.theme.dim)),
        Span::styled(meta.target, Style::default().fg(state.theme.fg)),
      ]));
      lines.push(Line::from(vec![
        Span::styled("  provider ", Style::default().fg(state.theme.dim)),
        Span::styled(meta.provider.unwrap_or_else(|| "unknown".to_string()), Style::default().fg(state.theme.fg)),
        Span::raw("  "),
        Span::styled("model ", Style::default().fg(state.theme.dim)),
        Span::styled(meta.model.unwrap_or_else(|| "unknown".to_string()), Style::default().fg(state.theme.fg)),
      ]));
      lines.push(Line::from(vec![
        Span::styled("  llm_ms ", Style::default().fg(state.theme.dim)),
        Span::styled(meta.llm_ms.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string()), Style::default().fg(state.theme.fg)),
        Span::raw("  "),
        Span::styled("build_ms ", Style::default().fg(state.theme.dim)),
        Span::styled(meta.build_ms.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string()), Style::default().fg(state.theme.fg)),
      ]));
      lines.push(Line::from(vec![
        Span::styled("  total_ms ", Style::default().fg(state.theme.dim)),
        Span::styled(meta.total_ms.to_string(), Style::default().fg(state.theme.fg)),
        Span::raw("  "),
        Span::styled("ts ", Style::default().fg(state.theme.dim)),
        Span::styled(meta.timestamp_unix_ms.to_string(), Style::default().fg(state.theme.fg)),
      ]));
    }
    if let Some(last) = &state.last_run {
      lines.push(Line::from(""));
      lines.push(Line::from(Span::styled("Last Run:", Style::default().fg(state.theme.dim))));
      let target = last.target.clone().unwrap_or_else(|| "auto".to_string());
      let age = last.when.elapsed().as_secs();
      let provider = last.provider.clone().unwrap_or_else(|| "unknown".to_string());
      let model = last.model.clone().unwrap_or_else(|| "unknown".to_string());
      let status = if last.ok { "ok" } else { "failed" };
      lines.push(Line::from(vec![
        Span::styled("  action ", Style::default().fg(state.theme.dim)),
        Span::styled(last.action.clone(), Style::default().fg(state.theme.fg)),
        Span::raw("  "),
        Span::styled("target ", Style::default().fg(state.theme.dim)),
        Span::styled(target, Style::default().fg(state.theme.fg)),
      ]));
      lines.push(Line::from(vec![
        Span::styled("  provider ", Style::default().fg(state.theme.dim)),
        Span::styled(provider, Style::default().fg(state.theme.fg)),
        Span::raw("  "),
        Span::styled("model ", Style::default().fg(state.theme.dim)),
        Span::styled(model, Style::default().fg(state.theme.fg)),
      ]));
      lines.push(Line::from(vec![
        Span::styled("  duration ", Style::default().fg(state.theme.dim)),
        Span::styled(format!("{} ms", last.duration_ms), Style::default().fg(state.theme.fg)),
        Span::raw("  "),
        Span::styled("age ", Style::default().fg(state.theme.dim)),
        Span::styled(format!("{}s", age), Style::default().fg(state.theme.fg)),
        Span::raw("  "),
        Span::styled("status ", Style::default().fg(state.theme.dim)),
        Span::styled(status, Style::default().fg(if last.ok { state.theme.accent } else { state.theme.accent2 })),
      ]));
    }
  } else {
    lines.push(Line::from(vec![Span::styled(
      "Select a .sculpt file to see details",
      Style::default().fg(state.theme.dim),
    )]));
  }
  lines
}

fn render_log_lines(state: &AppState) -> Vec<Line<'_>> {
  state
    .log
    .iter()
    .rev()
    .take(12)
    .rev()
    .map(|l| {
      let lower = l.to_lowercase();
      let style = if lower.contains("error") || lower.contains("failed") {
        Style::default().fg(state.theme.accent2)
      } else if lower.contains("build complete") || lower.contains(" ok") {
        Style::default().fg(state.theme.accent)
      } else if l.starts_with("$ ") {
        Style::default().fg(state.theme.dim)
      } else {
        Style::default().fg(state.theme.fg)
      };
      Line::from(Span::styled(l.clone(), style))
    })
    .collect()
}

fn extract_target(args: &[String]) -> Option<String> {
  let mut iter = args.iter();
  while let Some(arg) = iter.next() {
    if arg == "--target" {
      return iter.next().cloned();
    }
  }
  None
}

fn extract_provider_model(output: &std::process::Output) -> (Option<String>, Option<String>) {
  let mut provider = None;
  let mut model = None;
  let text = format!(
    "{}\n{}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr)
  );
  for part in text.split_whitespace() {
    if part.starts_with("provider=") {
      provider = Some(part.trim_start_matches("provider=").trim_matches(',').to_string());
    }
    if part.starts_with("model=") {
      model = Some(part.trim_start_matches("model=").trim_matches(',').to_string());
    }
  }
  (provider, model)
}

fn build_args(state: &AppState, action: &str) -> Vec<String> {
  let Some(file) = &state.selected_file else { return vec![action.to_string()]; };
  let target = if let Some(meta) = &state.meta_target {
    Some(meta.clone())
  } else {
    state.target_state.selected().and_then(|i| state.targets.get(i).cloned())
  };
  let mut args = vec![action.to_string(), file.to_string_lossy().to_string()];
  if let Some(t) = target {
    args.push("--target".to_string());
    args.push(t);
  }
  args
}

fn can_run_for_selected(state: &AppState) -> bool {
  let Some(target) = state.active_target() else { return false; };
  let Some(file) = &state.selected_file else { return false; };
  let dist_dir = dist_dir_for(file);
  match target.as_str() {
    "cli" => dist_dir.join("main.js").exists(),
    "web" => dist_dir.join("index.html").exists(),
    "gui" => dist_dir.join("gui/.build/release/SculptGui").exists(),
    _ => true,
  }
}

fn dist_dir_for(path: &Path) -> PathBuf {
  let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("sculpt");
  Path::new("dist").join(stem)
}

fn header_line(state: &AppState, width: u16) -> Line<'static> {
  let left_plain = format!("SCULPT Compiler {}", env!("CARGO_PKG_VERSION"));
  let right_plain = "(C) 2026 byte5 GmbH";
  let content_width = width.saturating_sub(4) as usize;
  let left_len = left_plain.len();
  let right_len = right_plain.len();
  let include_right = content_width >= left_len + 1 + right_len;
  let spacer = if include_right {
    " ".repeat(content_width.saturating_sub(left_len + right_len))
  } else {
    String::new()
  };
  let mut spans = vec![
    Span::styled("SCULPT ", Style::default().fg(state.theme.accent2).add_modifier(Modifier::BOLD)),
    Span::styled("Compiler ", Style::default().fg(state.theme.accent).add_modifier(Modifier::BOLD)),
    Span::styled(env!("CARGO_PKG_VERSION"), Style::default().fg(state.theme.accent)),
  ];
  if include_right {
    spans.push(Span::raw(spacer));
    spans.push(Span::styled(right_plain, Style::default().fg(state.theme.dim)));
  }
  Line::from(spans)
}

fn render_modal(f: &mut ratatui::Frame, state: &mut AppState) {
  let area = centered_rect(32, 24, f.size());
  f.render_widget(Clear, area);
  f.render_widget(Block::default().style(Style::default().bg(state.theme.panel_bg)), area);
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(4)].as_ref())
    .split(area);

  let title = Paragraph::new(Line::from(vec![
    Span::styled("Select Target", Style::default().fg(state.theme.accent).add_modifier(Modifier::BOLD)),
  ]))
  .block(
    Block::default()
      .borders(Borders::ALL)
      .padding(Padding::horizontal(1))
      .style(Style::default().bg(state.theme.panel_bg)),
  );
  f.render_widget(title, chunks[0]);

  let body = chunks[1];

  let targets = if state.meta_target.is_some() {
    vec![ListItem::new(Line::from(Span::raw(
      format!("{} (locked)", state.meta_target.as_ref().unwrap()),
    )))]
  } else {
    state
      .targets
      .iter()
      .map(|t| ListItem::new(Line::from(Span::raw(t.clone()))))
      .collect()
  };
  let mut tlist = List::new(targets)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("[Target]", Style::default().fg(state.theme.fg)))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(state.theme.panel_bg)),
    )
    .highlight_style(Style::default().bg(state.theme.highlight_bg).fg(state.theme.fg).add_modifier(Modifier::BOLD));
  if state.modal_focus != ModalFocus::Targets || state.meta_target.is_some() {
    tlist = tlist.highlight_style(Style::default().bg(state.theme.panel_bg).fg(state.theme.dim));
  }
  let mut tstate = state.target_state.clone();
  f.render_stateful_widget(tlist, body, &mut tstate);
  state.target_state = tstate;
}

fn render_info_modal(f: &mut ratatui::Frame, state: &AppState, msg: &str) {
  let area = centered_rect(36, 18, f.size());
  f.render_widget(Clear, area);
  let block = Block::default()
    .borders(Borders::ALL)
    .padding(Padding::horizontal(1))
    .style(Style::default().bg(state.theme.panel_bg));
  let text = Paragraph::new(Line::from(vec![
    Span::styled(msg, Style::default().fg(state.theme.accent)),
  ]))
  .block(block);
  f.render_widget(text, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
      [
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
      ]
      .as_ref(),
    )
    .split(r);
  Layout::default()
    .direction(Direction::Horizontal)
    .constraints(
      [
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
      ]
      .as_ref(),
    )
    .split(popup_layout[1])[1]
}

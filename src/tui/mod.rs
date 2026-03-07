use std::cmp::min;
use std::collections::BTreeMap;
use std::fs;
use std::io::stdout;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap,
};
use ratatui::Terminal;
use serde::{Deserialize, Serialize};

use crate::build_meta::{
    dist_dir_for_input, now_unix_ms, read_build_history, read_build_meta, BuildMeta,
};
use crate::targets::list_targets;
use crate::versioning::LANGUAGE_DEFAULT;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Files,
    Details,
    Log,
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
    is_project: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectedKind {
    Script,
    Project,
}

#[derive(Default, Clone)]
struct ProjectPreview {
    name: String,
    entry: String,
    modules: Vec<String>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct TuiConfig {
    provider: Option<String>,
    openai: Option<TuiProviderConfig>,
    anthropic: Option<TuiProviderConfig>,
    gemini: Option<TuiProviderConfig>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct TuiProviderConfig {
    api_key: Option<String>,
    model: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveModal {
    Target,
    Config,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfigField {
    Provider,
    OpenAiKey,
    OpenAiModel,
    AnthropicKey,
    AnthropicModel,
    GeminiKey,
    GeminiModel,
    Save,
    Test,
    Cancel,
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
            panel_bg: Color::Rgb(18, 20, 22), // neutral anthracite
            fg: Color::Rgb(255, 255, 255),    // white
            dim: Color::Rgb(150, 160, 170),
            accent: Color::Rgb(0, 255, 255),   // byte5 cyan
            accent2: Color::Rgb(234, 81, 114), // byte5 pink (accent)
            highlight_bg: Color::Rgb(28, 30, 34),
        }
    }
}

struct AppState {
    cwd: PathBuf,
    sculpt_cmd: PathBuf,
    entries: Vec<Entry>,
    file_state: ListState,
    targets: Vec<String>,
    target_state: ListState,
    focus: Focus,
    details_scroll: usize,
    log_scroll: usize,
    log: Vec<String>,
    history: Vec<String>,
    status: String,
    selected_file: Option<PathBuf>,
    selected_kind: Option<SelectedKind>,
    meta_target: Option<String>,
    preview_meta: BTreeMap<String, String>,
    preview_lines: usize,
    preview_size: u64,
    preview_intro: Vec<String>,
    project_preview: Option<ProjectPreview>,
    last_run: Option<LastRun>,
    last_refresh: Instant,
    theme: Theme,
    modal_open: bool,
    active_modal: ActiveModal,
    modal_focus: ModalFocus,
    info_modal: Option<String>,
    pending_action: PendingAction,
    config: TuiConfig,
    config_path: PathBuf,
    config_field: ConfigField,
    config_editing: bool,
    config_input: String,
    needs_full_redraw: bool,
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
        if state.needs_full_redraw {
            terminal.clear()?;
            state.needs_full_redraw = false;
        }
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
        let sculpt_cmd = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("sculpt"));
        let config_path = cwd.join("sculpt.config.json");
        let config = load_tui_config(&config_path);
        let entries = read_entries(&cwd)?;
        let targets = list_targets()
            .unwrap_or_else(|_| vec!["cli".to_string(), "gui".to_string(), "web".to_string()]);
        let mut file_state = ListState::default();
        file_state.select(Some(0));
        let mut target_state = ListState::default();
        target_state.select(Some(0));

        Ok(Self {
            cwd,
            sculpt_cmd,
            entries,
            file_state,
            targets,
            target_state,
            focus: Focus::Files,
            details_scroll: 0,
            log_scroll: 0,
            log: vec![
                "SCULPT TUI ready".to_string(),
                format!("Language {}", LANGUAGE_DEFAULT),
            ],
            history: Vec::new(),
            status: "Ready".to_string(),
            selected_file: None,
            selected_kind: None,
            meta_target: None,
            preview_meta: BTreeMap::new(),
            preview_lines: 0,
            preview_size: 0,
            preview_intro: Vec::new(),
            project_preview: None,
            last_run: None,
            last_refresh: Instant::now(),
            theme: Theme::dark(),
            modal_open: false,
            active_modal: ActiveModal::Target,
            modal_focus: ModalFocus::Targets,
            info_modal: None,
            pending_action: PendingAction::BuildRun,
            config,
            config_path,
            config_field: ConfigField::Provider,
            config_editing: false,
            config_input: String::new(),
            needs_full_redraw: false,
        })
    }

    fn refresh_entries(&mut self) -> Result<()> {
        self.config_path = self.cwd.join("sculpt.config.json");
        self.config = load_tui_config(&self.config_path);
        self.entries = read_entries(&self.cwd)?;
        let idx = self.file_state.selected().unwrap_or(0);
        let idx = min(idx, self.entries.len().saturating_sub(1));
        self.file_state.select(Some(idx));
        self.update_preview_from_selection();
        self.details_scroll = 0;
        self.log_scroll = 0;
        Ok(())
    }

    fn refresh_targets(&mut self) -> Result<()> {
        let targets = list_targets()
            .unwrap_or_else(|_| vec!["cli".to_string(), "gui".to_string(), "web".to_string()]);
        self.targets = targets;
        if let Some(meta) = &self.meta_target {
            if let Some(idx) = self.targets.iter().position(|t| t == meta) {
                self.target_state.select(Some(idx));
            }
        } else {
            let idx = self.target_state.selected().unwrap_or(0);
            self.target_state
                .select(Some(min(idx, self.targets.len().saturating_sub(1))));
        }
        Ok(())
    }

    fn set_selected_input(&mut self, path: PathBuf, kind: SelectedKind) -> Result<()> {
        self.selected_file = Some(path.clone());
        self.selected_kind = Some(kind);
        let (meta, project_preview) = match kind {
            SelectedKind::Script => (extract_meta(&path)?, None),
            SelectedKind::Project => extract_project_preview_and_meta(&path)?,
        };
        self.meta_target = meta.get("target").cloned();
        self.preview_meta = meta;
        self.preview_lines = count_lines(&path).unwrap_or(0);
        self.preview_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        self.preview_intro = extract_intro_comment(&path).unwrap_or_default();
        self.project_preview = project_preview;
        if let Some(meta) = &self.meta_target {
            if let Some(idx) = self.targets.iter().position(|t| t == meta) {
                self.target_state.select(Some(idx));
            }
        }
        Ok(())
    }

    fn update_preview_from_selection(&mut self) {
        let Some(idx) = self.file_state.selected() else {
            self.clear_preview();
            return;
        };
        let Some(entry) = self.entries.get(idx) else {
            self.clear_preview();
            return;
        };
        if !entry.is_sculpt && !entry.is_project {
            self.clear_preview();
            return;
        }
        let kind = if entry.is_project {
            SelectedKind::Project
        } else {
            SelectedKind::Script
        };
        let _ = self.set_selected_input(entry.path.clone(), kind);
        self.details_scroll = 0;
    }

    fn clear_preview(&mut self) {
        self.selected_file = None;
        self.selected_kind = None;
        self.meta_target = None;
        self.preview_meta.clear();
        self.preview_lines = 0;
        self.preview_size = 0;
        self.preview_intro.clear();
        self.project_preview = None;
        self.details_scroll = 0;
    }

    fn active_target(&self) -> Option<String> {
        if let Some(meta) = &self.meta_target {
            return Some(meta.clone());
        }
        self.target_state
            .selected()
            .and_then(|i| self.targets.get(i).cloned())
    }

    fn can_run(&self) -> bool {
        can_run_for_selected(self)
    }

    fn sync_selected_from_cursor(&mut self) -> Result<()> {
        let Some(idx) = self.file_state.selected() else {
            self.clear_preview();
            return Ok(());
        };
        let Some(entry) = self.entries.get(idx) else {
            self.clear_preview();
            return Ok(());
        };
        if entry.is_sculpt {
            self.set_selected_input(entry.path.clone(), SelectedKind::Script)?;
        } else if entry.is_project {
            self.set_selected_input(entry.path.clone(), SelectedKind::Project)?;
        } else {
            self.clear_preview();
        }
        Ok(())
    }

    fn run_sculpt(&mut self, args: &[String]) -> Result<()> {
        let cmd_display = self.sculpt_cmd.display().to_string();
        self.status = format!("Running: {} {}", cmd_display, args.join(" "));
        self.history.push(args.join(" "));
        if self.history.len() > 10 {
            self.history.remove(0);
        }
        let started = Instant::now();
        let output = Command::new(&self.sculpt_cmd).args(args).output()?;
        let duration = started.elapsed();
        self.log
            .push(format!("$ {} {}", cmd_display, args.join(" ")));
        self.log.extend(normalize_log_output(&output.stdout));
        self.log.extend(normalize_log_output(&output.stderr));
        self.log_scroll = 0;
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

    fn run_sculpt_interactive(&mut self, args: &[String]) -> Result<()> {
        let cmd_display = self.sculpt_cmd.display().to_string();
        self.status = format!("Running: {} {}", cmd_display, args.join(" "));
        self.history.push(args.join(" "));
        if self.history.len() > 10 {
            self.history.remove(0);
        }
        self.log
            .push(format!("$ {} {}", cmd_display, args.join(" ")));
        self.log.push("Launching interactive run...".to_string());
        self.log_scroll = 0;

        let started = Instant::now();
        let mut out = stdout();
        let _ = disable_raw_mode();
        let _ = execute!(out, LeaveAlternateScreen);
        let status_res = Command::new(&self.sculpt_cmd).args(args).status();
        let _ = execute!(
            out,
            EnterAlternateScreen,
            crossterm::terminal::Clear(ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        );
        let _ = enable_raw_mode();
        self.needs_full_redraw = true;
        let duration = started.elapsed();

        let action = args.get(0).cloned().unwrap_or_else(|| "run".to_string());
        let target = extract_target(args);

        match status_res {
            Ok(status) => {
                let ok = status.success();
                self.last_run = Some(LastRun {
                    action,
                    target,
                    duration_ms: duration.as_millis(),
                    when: Instant::now(),
                    provider: None,
                    model: None,
                    ok,
                });
                if ok {
                    self.log.push("Interactive run finished.".to_string());
                    self.status = "Ready".to_string();
                    Ok(())
                } else {
                    self.log.push(format!(
                        "Interactive run failed (exit={:?}).",
                        status.code()
                    ));
                    self.status = format!("Failed: status {:?}", status.code());
                    bail!("Command failed");
                }
            }
            Err(err) => {
                self.last_run = Some(LastRun {
                    action,
                    target,
                    duration_ms: duration.as_millis(),
                    when: Instant::now(),
                    provider: None,
                    model: None,
                    ok: false,
                });
                self.log
                    .push(format!("Interactive run failed to start: {}", err));
                self.status = "Failed: could not launch interactive run".to_string();
                Err(err.into())
            }
        }
    }

    fn selected_entry_path(&self) -> Option<PathBuf> {
        let idx = self.file_state.selected()?;
        self.entries.get(idx).map(|e| e.path.clone())
    }

    fn run_editor_for_selection(&mut self) -> Result<()> {
        let Some(path) = self.selected_entry_path() else {
            self.info_modal = Some("Select a file first.".to_string());
            return Ok(());
        };
        if path.is_dir() {
            self.info_modal = Some("Select a file (not a directory).".to_string());
            return Ok(());
        }

        let mut editor_cmd: Option<(String, Vec<String>)> = None;
        if let Ok(ed) = std::env::var("EDITOR") {
            let trimmed = ed.trim();
            if !trimmed.is_empty() {
                let mut parts = trimmed.split_whitespace();
                if let Some(bin) = parts.next() {
                    let mut args: Vec<String> = parts.map(|s| s.to_string()).collect();
                    args.push(path.to_string_lossy().to_string());
                    editor_cmd = Some((bin.to_string(), args));
                }
            }
        }
        if editor_cmd.is_none() {
            if command_exists("code") {
                editor_cmd = Some((
                    "code".to_string(),
                    vec!["-w".to_string(), path.to_string_lossy().to_string()],
                ));
            } else if command_exists("nano") {
                editor_cmd = Some(("nano".to_string(), vec![path.to_string_lossy().to_string()]));
            } else {
                editor_cmd = Some(("vi".to_string(), vec![path.to_string_lossy().to_string()]));
            }
        }

        let (bin, args) = editor_cmd.expect("editor command exists");
        self.log.push(format!("$ {} {}", bin, args.join(" ")));
        self.status = format!("Editor: {}", path.display());

        let mut out = stdout();
        let _ = disable_raw_mode();
        let _ = execute!(out, LeaveAlternateScreen);
        let result = Command::new(&bin)
            .args(&args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
        let _ = execute!(
            out,
            EnterAlternateScreen,
            crossterm::terminal::Clear(ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        );
        let _ = enable_raw_mode();
        self.needs_full_redraw = true;

        match result {
            Ok(status) if status.success() => {
                self.status = "Ready".to_string();
                self.log.push("Editor closed.".to_string());
                self.refresh_entries()?;
            }
            Ok(status) => {
                self.status = format!("Editor exit {:?}", status.code());
                self.log
                    .push(format!("Editor exited with status {:?}.", status.code()));
            }
            Err(err) => {
                self.status = "Editor launch failed".to_string();
                self.log.push(format!("Editor launch failed: {}", err));
            }
        }
        Ok(())
    }
}

fn command_exists(bin: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| {
            let p = dir.join(bin);
            p.exists() && p.is_file()
        })
    })
}

fn cycle_focus(current: Focus, reverse: bool) -> Focus {
    match (current, reverse) {
        (Focus::Files, false) => Focus::Details,
        (Focus::Details, false) => Focus::Log,
        (Focus::Log, false) => Focus::Files,
        (Focus::Files, true) => Focus::Log,
        (Focus::Details, true) => Focus::Files,
        (Focus::Log, true) => Focus::Details,
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
        KeyCode::Tab => {
            state.focus = cycle_focus(state.focus, false);
        }
        KeyCode::BackTab => {
            state.focus = cycle_focus(state.focus, true);
        }
        KeyCode::Up => match state.focus {
            Focus::Files => {
                move_selection(&mut state.file_state, state.entries.len(), -1);
                state.update_preview_from_selection();
            }
            Focus::Details => {
                state.details_scroll = state.details_scroll.saturating_sub(1);
            }
            Focus::Log => {
                state.log_scroll = state.log_scroll.saturating_sub(1);
            }
        },
        KeyCode::Down => match state.focus {
            Focus::Files => {
                move_selection(&mut state.file_state, state.entries.len(), 1);
                state.update_preview_from_selection();
            }
            Focus::Details => {
                state.details_scroll = state.details_scroll.saturating_add(1);
            }
            Focus::Log => {
                state.log_scroll = state.log_scroll.saturating_add(1);
            }
        },
        KeyCode::Enter => {
            if state.focus == Focus::Files {
                if let Some(idx) = state.file_state.selected() {
                    if let Some(entry) = state.entries.get(idx) {
                        if entry.is_dir {
                            state.cwd = entry.path.clone();
                            state.refresh_entries()?;
                        } else if entry.is_sculpt || entry.is_project {
                            let kind = if entry.is_project {
                                SelectedKind::Project
                            } else {
                                SelectedKind::Script
                            };
                            state.set_selected_input(entry.path.clone(), kind)?;
                            state.pending_action = PendingAction::BuildRun;
                            if state.meta_target.is_some() {
                                execute_pending_action(state)?;
                            } else {
                                state.modal_open = true;
                                state.active_modal = ActiveModal::Target;
                                state.modal_focus = ModalFocus::Targets;
                            }
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
            state.sync_selected_from_cursor()?;
            if state.selected_file.is_some() {
                state.pending_action = PendingAction::BuildOnly;
                if state.meta_target.is_some() {
                    execute_pending_action(state)?;
                } else {
                    state.modal_open = true;
                    state.active_modal = ActiveModal::Target;
                    state.modal_focus = ModalFocus::Targets;
                }
            } else {
                state.info_modal = Some("Select a .sculpt or .sculpt.json file first.".to_string());
            }
        }
        KeyCode::Char('r') => {
            state.sync_selected_from_cursor()?;
            if state.selected_file.is_some() {
                state.pending_action = PendingAction::RunOnly;
                if state.meta_target.is_some() {
                    execute_pending_action(state)?;
                } else {
                    state.modal_open = true;
                    state.active_modal = ActiveModal::Target;
                    state.modal_focus = ModalFocus::Targets;
                }
            } else {
                state.info_modal = Some("Select a .sculpt or .sculpt.json file first.".to_string());
            }
        }
        KeyCode::Char('f') => {
            state.sync_selected_from_cursor()?;
            if state.selected_file.is_some() {
                let _ = state.run_sculpt(&build_args(state, "freeze"));
            } else {
                state.info_modal = Some("Select a .sculpt or .sculpt.json file first.".to_string());
            }
        }
        KeyCode::Char('p') => {
            state.sync_selected_from_cursor()?;
            if state.selected_file.is_some() {
                let _ = state.run_sculpt(&build_args(state, "replay"));
            } else {
                state.info_modal = Some("Select a .sculpt or .sculpt.json file first.".to_string());
            }
        }
        KeyCode::Char('c') => {
            state.sync_selected_from_cursor()?;
            if state.selected_file.is_some() {
                let _ = state.run_sculpt(&build_args(state, "clean"));
                state.update_preview_from_selection();
            } else {
                state.info_modal = Some("Select a .sculpt or .sculpt.json file first.".to_string());
            }
        }
        KeyCode::Char('g') => {
            state.config_editing = false;
            state.config_input.clear();
            state.config_field = ConfigField::Provider;
            state.modal_open = true;
            state.active_modal = ActiveModal::Config;
        }
        KeyCode::Char('e') => {
            let _ = state.run_editor_for_selection();
            state.update_preview_from_selection();
        }
        KeyCode::Char('.') => {
            if let Some(last) = state.history.last().cloned() {
                let args: Vec<String> = last.split_whitespace().map(|s| s.to_string()).collect();
                let _ = state.run_sculpt(&args);
            } else {
                state.info_modal = Some("No action history yet.".to_string());
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_modal_key(state: &mut AppState, key: KeyEvent) -> Result<bool> {
    if state.active_modal == ActiveModal::Config {
        return handle_config_modal_key(state, key);
    }
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

fn handle_config_modal_key(state: &mut AppState, key: KeyEvent) -> Result<bool> {
    if state.config_editing {
        match key.code {
            KeyCode::Esc => {
                state.config_editing = false;
            }
            KeyCode::Enter => {
                apply_config_input(state);
                state.config_editing = false;
            }
            KeyCode::Backspace => {
                state.config_input.pop();
            }
            KeyCode::Char(c) => {
                state.config_input.push(c);
            }
            _ => {}
        }
        return Ok(false);
    }

    match key.code {
        KeyCode::Esc => state.modal_open = false,
        KeyCode::Up => config_move_field(state, -1),
        KeyCode::Down => config_move_field(state, 1),
        KeyCode::Left => config_select_provider(state, -1),
        KeyCode::Right => config_select_provider(state, 1),
        KeyCode::Enter => match state.config_field {
            ConfigField::Provider => config_select_provider(state, 1),
            ConfigField::OpenAiKey
            | ConfigField::OpenAiModel
            | ConfigField::AnthropicKey
            | ConfigField::AnthropicModel
            | ConfigField::GeminiKey
            | ConfigField::GeminiModel => {
                state.config_input = current_config_field_value(state);
                state.config_editing = true;
            }
            ConfigField::Save => {
                save_tui_config(&state.config_path, &state.config)?;
                state
                    .log
                    .push(format!("Saved {}", state.config_path.display()));
                state.modal_open = false;
            }
            ConfigField::Test => {
                let provider = state
                    .config
                    .provider
                    .clone()
                    .unwrap_or_else(|| "stub".to_string());
                let args = vec![
                    "auth".to_string(),
                    "check".to_string(),
                    "--provider".to_string(),
                    provider,
                    "--verify".to_string(),
                ];
                let _ = state.run_sculpt(&args);
            }
            ConfigField::Cancel => state.modal_open = false,
        },
        _ => {}
    }
    Ok(false)
}

fn execute_pending_action(state: &mut AppState) -> Result<()> {
    if let Some(msg) = preflight_issue(state) {
        state.info_modal = Some(msg);
        state.modal_open = false;
        return Ok(());
    }
    match state.pending_action {
        PendingAction::BuildOnly => {
            let _ = state.run_sculpt(&build_args(state, "build"));
        }
        PendingAction::RunOnly => {
            if can_run_for_selected(state) {
                let _ = state.run_sculpt_interactive(&build_args(state, "run"));
            } else {
                state.info_modal = Some("Run not available. Build first.".to_string());
            }
        }
        PendingAction::BuildRun => {
            if can_run_for_selected(state) && !selected_output_stale(state) {
                let _ = state.run_sculpt_interactive(&build_args(state, "run"));
            } else {
                let _ = state.run_sculpt(&build_args(state, "build"));
                if can_run_for_selected(state) {
                    let _ = state.run_sculpt_interactive(&build_args(state, "run"));
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
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(15),
                Constraint::Min(8),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(size);

    let header_line = header_line(state, layout[0].width);
    let header = Paragraph::new(vec![header_line]).block(
        Block::default()
            .borders(Borders::ALL)
            .padding(Padding {
                left: 1,
                right: 1,
                top: 0,
                bottom: 0,
            })
            .style(Style::default().bg(state.theme.panel_bg)),
    );
    f.render_widget(header, layout[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)].as_ref())
        .split(layout[1]);

    let files_active = state.focus == Focus::Files;
    let details_active = state.focus == Focus::Details;
    let log_active = state.focus == Focus::Log;
    let active_border = |active: bool| {
        if active {
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(state.theme.fg)
        }
    };
    let active_title = |active: bool| {
        if active {
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(state.theme.fg)
        }
    };

    let files = list_files(state);
    let mut file_state = state.file_state.clone();
    f.render_stateful_widget(files, body[0], &mut file_state);
    state.file_state = file_state;

    let details_lines = render_details(state);
    let details_inner_h = body[1].height.saturating_sub(2) as usize;
    let details_max_scroll = details_lines.len().saturating_sub(details_inner_h);
    let details_scroll = min(state.details_scroll, details_max_scroll);
    let details = Paragraph::new(details_lines.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("[Details]", active_title(details_active)))
                .border_type(if details_active {
                    BorderType::Thick
                } else {
                    BorderType::Plain
                })
                .padding(Padding::horizontal(1))
                .border_style(active_border(details_active))
                .style(Style::default().bg(state.theme.panel_bg)),
        )
        .scroll((details_scroll as u16, 0))
        .wrap(Wrap { trim: true });
    f.render_widget(details, body[1]);

    let log_lines = render_log_lines(state);
    let log_inner_h = layout[2].height.saturating_sub(2) as usize;
    let log_max_scroll = log_lines.len().saturating_sub(log_inner_h);
    let log_scroll = min(state.log_scroll, log_max_scroll);
    let log = Paragraph::new(log_lines.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("[Log]", active_title(log_active)))
                .border_type(if log_active {
                    BorderType::Thick
                } else {
                    BorderType::Plain
                })
                .padding(Padding::horizontal(1))
                .border_style(active_border(log_active))
                .style(Style::default().bg(state.theme.panel_bg)),
        )
        .scroll((log_scroll as u16, 0))
        .wrap(Wrap { trim: true });
    f.render_widget(log, layout[2]);
    let files_up = state.file_state.selected().unwrap_or(0) > 0;
    let files_down = state.file_state.selected().unwrap_or(0) + 1 < state.entries.len();
    render_scroll_markers(f, body[0], files_up, files_down, files_active, state);
    render_scroll_markers(
        f,
        body[1],
        details_scroll > 0,
        details_scroll < details_max_scroll,
        details_active,
        state,
    );
    render_scroll_markers(
        f,
        layout[2],
        log_scroll > 0,
        log_scroll < log_max_scroll,
        log_active,
        state,
    );

    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "↑↓",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" move  "),
        Span::styled(
            "Tab",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" pane  "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" run/build  "),
        Span::styled(
            "B",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" build  "),
        Span::styled(
            "R",
            Style::default()
                .fg(if state.can_run() {
                    state.theme.accent2
                } else {
                    state.theme.dim
                })
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" run  "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" exit  "),
        Span::styled(
            "F",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" freeze  "),
        Span::styled(
            "P",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" replay  "),
        Span::styled(
            "C",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" clean  "),
        Span::styled(
            "G",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" config  "),
        Span::styled(
            "E",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" edit  "),
        Span::styled(
            ".",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" rerun  "),
        Span::styled("↑↓", Style::default().fg(state.theme.dim)),
        Span::raw(" move/scroll  "),
        Span::raw("  "),
        Span::styled("active:", Style::default().fg(state.theme.dim)),
        Span::raw(" "),
        Span::styled(
            if files_active {
                "files"
            } else if details_active {
                "details"
            } else {
                "log"
            },
            Style::default().fg(state.theme.accent),
        ),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1))
            .style(Style::default().bg(state.theme.panel_bg)),
    );
    f.render_widget(footer, layout[3]);

    if state.modal_open {
        match state.active_modal {
            ActiveModal::Target => render_target_modal(f, state),
            ActiveModal::Config => render_config_modal(f, state),
        }
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
            let icon = if e.is_dir {
                "[D]"
            } else if e.is_project {
                "[P]"
            } else if e.is_sculpt {
                "[S]"
            } else {
                "   "
            };
            let icon_span = if e.is_sculpt {
                Span::styled(
                    icon,
                    Style::default()
                        .fg(state.theme.accent2)
                        .add_modifier(Modifier::BOLD),
                )
            } else if e.is_project {
                Span::styled(
                    icon,
                    Style::default()
                        .fg(state.theme.accent)
                        .add_modifier(Modifier::BOLD),
                )
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
                .title(Span::styled(
                    "[Files]",
                    Style::default().fg(if state.focus == Focus::Files {
                        state.theme.accent2
                    } else {
                        state.theme.fg
                    }),
                ))
                .border_type(if state.focus == Focus::Files {
                    BorderType::Thick
                } else {
                    BorderType::Plain
                })
                .padding(Padding::horizontal(1))
                .border_style(if state.focus == Focus::Files {
                    Style::default()
                        .fg(state.theme.accent2)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(state.theme.fg)
                })
                .style(Style::default().bg(state.theme.panel_bg)),
        )
        .highlight_style(
            Style::default()
                .bg(state.theme.highlight_bg)
                .fg(state.theme.fg)
                .add_modifier(Modifier::BOLD),
        );
    if state.focus != Focus::Files {
        list = list.highlight_style(
            Style::default()
                .bg(state.theme.panel_bg)
                .fg(state.theme.dim),
        );
    }
    list
}

fn render_scroll_markers(
    f: &mut ratatui::Frame,
    rect: Rect,
    can_up: bool,
    can_down: bool,
    active: bool,
    state: &AppState,
) {
    if rect.width < 3 || rect.height < 3 {
        return;
    }
    let color = if active {
        state.theme.accent2
    } else {
        state.theme.dim
    };
    let style = Style::default().fg(color).add_modifier(Modifier::BOLD);
    let x = rect.x + rect.width - 1;
    if can_up {
        f.render_widget(
            Paragraph::new("▲").style(style),
            Rect::new(x, rect.y + 1, 1, 1),
        );
    }
    if can_down {
        f.render_widget(
            Paragraph::new("▼").style(style),
            Rect::new(x, rect.y + rect.height - 2, 1, 1),
        );
    }
}

fn read_entries(dir: &Path) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();
    entries.push(Entry {
        name: "..".to_string(),
        path: dir.parent().unwrap_or(dir).to_path_buf(),
        is_dir: true,
        is_sculpt: false,
        is_project: false,
    });
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = path.is_dir();
        let is_sculpt = path.extension().and_then(|s| s.to_str()) == Some("sculpt");
        let is_project = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.ends_with(".sculpt.json"))
            .unwrap_or(false);
        entries.push(Entry {
            name,
            path,
            is_dir,
            is_sculpt,
            is_project,
        });
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
            let cleaned = line
                .trim_start_matches("//")
                .trim_start_matches('#')
                .trim_start_matches(';')
                .trim();
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
            Style::default()
                .fg(state.theme.fg)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![
            Span::styled("Lines: ", Style::default().fg(state.theme.dim)),
            Span::styled(
                state.preview_lines.to_string(),
                Style::default().fg(state.theme.fg),
            ),
            Span::raw("  "),
            Span::styled("Size: ", Style::default().fg(state.theme.dim)),
            Span::styled(
                format!("{} bytes", state.preview_size),
                Style::default().fg(state.theme.fg),
            ),
        ]));
        if let Some(project) = &state.project_preview {
            lines.push(Line::from(vec![
                Span::styled("Project: ", Style::default().fg(state.theme.dim)),
                Span::styled(
                    project.name.clone(),
                    Style::default().fg(state.theme.accent),
                ),
                Span::raw("  "),
                Span::styled("Entry: ", Style::default().fg(state.theme.dim)),
                Span::styled(project.entry.clone(), Style::default().fg(state.theme.fg)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Modules: ", Style::default().fg(state.theme.dim)),
                Span::styled(
                    project.modules.len().to_string(),
                    Style::default().fg(state.theme.fg),
                ),
            ]));
        }
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
        let has_gui = dist_dir.join("gui/.build/release/SculptGui").exists()
            || dist_dir.join("gui/main.py").exists();
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
            Span::styled(
                format!("{} build", build_box),
                Style::default().fg(state.theme.fg),
            ),
            Span::raw("   "),
            Span::styled(
                format!("{} lock", lock_box),
                Style::default().fg(state.theme.fg),
            ),
            Span::raw("    "),
            Span::styled("target ", Style::default().fg(state.theme.dim)),
            Span::styled(target, Style::default().fg(state.theme.fg)),
            Span::raw("   "),
            Span::styled("layout ", Style::default().fg(state.theme.dim)),
            Span::styled(layout, Style::default().fg(state.theme.fg)),
        ]));
        if !state.preview_intro.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Intro:",
                Style::default().fg(state.theme.dim),
            )));
            for line in &state.preview_intro {
                lines.push(Line::from(Span::styled(
                    format!("  {}", line),
                    Style::default().fg(state.theme.fg),
                )));
            }
        }
        if let Some(meta) = read_build_meta(&dist_dir) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Last Build:",
                Style::default().fg(state.theme.dim),
            )));
            lines.push(Line::from(vec![
                Span::styled("  action ", Style::default().fg(state.theme.dim)),
                Span::styled(meta.action, Style::default().fg(state.theme.fg)),
                Span::raw("  "),
                Span::styled("target ", Style::default().fg(state.theme.dim)),
                Span::styled(meta.target, Style::default().fg(state.theme.fg)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  provider ", Style::default().fg(state.theme.dim)),
                Span::styled(
                    meta.provider.unwrap_or_else(|| "unknown".to_string()),
                    Style::default().fg(state.theme.fg),
                ),
                Span::raw("  "),
                Span::styled("model ", Style::default().fg(state.theme.dim)),
                Span::styled(
                    meta.model.unwrap_or_else(|| "unknown".to_string()),
                    Style::default().fg(state.theme.fg),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  llm_ms ", Style::default().fg(state.theme.dim)),
                Span::styled(fmt_opt_ms(meta.llm_ms), Style::default().fg(state.theme.fg)),
                Span::raw("  "),
                Span::styled("build_ms ", Style::default().fg(state.theme.dim)),
                Span::styled(
                    fmt_opt_ms(meta.build_ms),
                    Style::default().fg(state.theme.fg),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  run_ms ", Style::default().fg(state.theme.dim)),
                Span::styled(fmt_opt_ms(meta.run_ms), Style::default().fg(state.theme.fg)),
                Span::raw("  "),
                Span::styled("total ", Style::default().fg(state.theme.dim)),
                Span::styled(fmt_ms(meta.total_ms), Style::default().fg(state.theme.fg)),
            ]));
            if let Some(tokens) = meta.token_usage.clone() {
                lines.push(Line::from(vec![
                    Span::styled(
                        "  tokens in/out/total ",
                        Style::default().fg(state.theme.dim),
                    ),
                    Span::styled(
                        tokens
                            .input_tokens
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "unknown".to_string()),
                        Style::default().fg(state.theme.fg),
                    ),
                    Span::raw("/"),
                    Span::styled(
                        tokens
                            .output_tokens
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "unknown".to_string()),
                        Style::default().fg(state.theme.fg),
                    ),
                    Span::raw("/"),
                    Span::styled(
                        tokens
                            .total_tokens
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "unknown".to_string()),
                        Style::default().fg(state.theme.fg),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("  tokens ", Style::default().fg(state.theme.dim)),
                    Span::styled("unavailable", Style::default().fg(state.theme.dim)),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled("  age ", Style::default().fg(state.theme.dim)),
                Span::styled(
                    fmt_age_ms(now_unix_ms().saturating_sub(meta.timestamp_unix_ms)),
                    Style::default().fg(state.theme.fg),
                ),
                Span::raw("  "),
                Span::styled("status ", Style::default().fg(state.theme.dim)),
                {
                    let status_text = meta.status.clone();
                    Span::styled(
                        status_text.clone(),
                        Style::default().fg(if status_text == "ok" {
                            state.theme.accent
                        } else {
                            state.theme.accent2
                        }),
                    )
                },
            ]));
        }
        let history = read_build_history(&dist_dir);
        if !history.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Recent Trend (last 5):",
                Style::default().fg(state.theme.dim),
            )));
            for entry in history.iter().rev().take(5) {
                lines.push(render_trend_line(state, entry));
            }
        }
        if let Some(last) = &state.last_run {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Last Run:",
                Style::default().fg(state.theme.dim),
            )));
            let target = last.target.clone().unwrap_or_else(|| "auto".to_string());
            let age = last.when.elapsed().as_secs();
            let provider = last
                .provider
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
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
                Span::styled(
                    format!("{} ms", last.duration_ms),
                    Style::default().fg(state.theme.fg),
                ),
                Span::raw("  "),
                Span::styled("age ", Style::default().fg(state.theme.dim)),
                Span::styled(format!("{}s", age), Style::default().fg(state.theme.fg)),
                Span::raw("  "),
                Span::styled("status ", Style::default().fg(state.theme.dim)),
                Span::styled(
                    status,
                    Style::default().fg(if last.ok {
                        state.theme.accent
                    } else {
                        state.theme.accent2
                    }),
                ),
            ]));
        }
        let diagnostics = extract_recent_diagnostics(&state.log);
        if !diagnostics.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Diagnostics:",
                Style::default().fg(state.theme.dim),
            )));
            for d in diagnostics.iter().take(6) {
                lines.push(Line::from(Span::styled(
                    format!("  {}", d),
                    Style::default().fg(state.theme.accent2),
                )));
            }
        }
    } else {
        lines.push(Line::from(vec![Span::styled(
            "Select a .sculpt or .sculpt.json file to see details",
            Style::default().fg(state.theme.dim),
        )]));
    }
    lines
}

fn render_trend_line(state: &AppState, entry: &BuildMeta) -> Line<'static> {
    let action = match entry.action.as_str() {
        "build" => "B",
        "run" => "R",
        "freeze" => "F",
        "replay" => "P",
        _ => "?",
    };
    let status_color = if entry.status == "ok" {
        state.theme.accent
    } else {
        state.theme.accent2
    };
    let age = fmt_age_ms(now_unix_ms().saturating_sub(entry.timestamp_unix_ms));
    let total_tokens = entry
        .token_usage
        .as_ref()
        .and_then(|t| t.total_tokens)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());

    Line::from(vec![
        Span::styled("  ", Style::default().fg(state.theme.dim)),
        Span::styled(
            action,
            Style::default()
                .fg(state.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default().fg(state.theme.dim)),
        Span::styled(entry.target.clone(), Style::default().fg(state.theme.fg)),
        Span::styled(" ", Style::default().fg(state.theme.dim)),
        Span::styled(entry.status.clone(), Style::default().fg(status_color)),
        Span::styled(" ", Style::default().fg(state.theme.dim)),
        Span::styled("t=", Style::default().fg(state.theme.dim)),
        Span::styled(fmt_ms(entry.total_ms), Style::default().fg(state.theme.fg)),
        Span::styled(" ", Style::default().fg(state.theme.dim)),
        Span::styled("tok=", Style::default().fg(state.theme.dim)),
        Span::styled(total_tokens, Style::default().fg(state.theme.fg)),
        Span::styled(" ", Style::default().fg(state.theme.dim)),
        Span::styled("age=", Style::default().fg(state.theme.dim)),
        Span::styled(age, Style::default().fg(state.theme.fg)),
    ])
}

fn fmt_opt_ms(v: Option<u128>) -> String {
    v.map(fmt_ms).unwrap_or_else(|| "-".to_string())
}

fn fmt_ms(ms: u128) -> String {
    if ms >= 10_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}

fn fmt_age_ms(delta_ms: u128) -> String {
    let secs = (delta_ms / 1000) as u64;
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86_400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86_400)
    }
}

fn extract_recent_diagnostics(log: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for line in log.iter().rev() {
        let trimmed = line.trim();
        if looks_like_diagnostic_line(trimmed) {
            out.push(trimmed.to_string());
        }
        if out.len() >= 12 {
            break;
        }
    }
    out.reverse();
    out
}

fn looks_like_diagnostic_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.starts_with("Error:") {
        return true;
    }
    // Accept compiler-style diagnostic codes such as:
    // N309: ...
    // M701: ...
    // NS504: ...
    if let Some((head, _rest)) = trimmed.split_once(':') {
        if head.len() >= 4
            && head
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            && head.chars().any(|c| c.is_ascii_digit())
            && head.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        {
            return true;
        }
    }
    false
}

fn render_log_lines(state: &AppState) -> Vec<Line<'_>> {
    state
        .log
        .iter()
        .map(|l| {
            if let Some((idx, label, status)) = parse_pipeline_log_line(l) {
                return render_pipeline_log_line(state, idx, &label, &status);
            }
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

fn parse_pipeline_log_line(line: &str) -> Option<(usize, String, String)> {
    let s = line.trim_start();
    let (idx_raw, tail) = s.split_once('.')?;
    let idx = idx_raw.trim().parse::<usize>().ok()?;
    if idx == 0 || idx > 3 {
        return None;
    }
    let body = tail.trim_start();

    if let Some(label) = body.strip_suffix(" ok") {
        return Some((idx, label.trim_end().to_string(), "ok".to_string()));
    }
    if let Some(label) = body.strip_suffix(" failed") {
        return Some((idx, label.trim_end().to_string(), "failed".to_string()));
    }
    if let Some(pos) = body.rfind(" running ") {
        let label = body[..pos].trim_end().to_string();
        let status = body[pos + 1..].to_string();
        return Some((idx, label, status));
    }
    None
}

fn render_pipeline_log_line(
    state: &AppState,
    idx: usize,
    label: &str,
    status: &str,
) -> Line<'static> {
    const WIDTH: usize = 12;

    let filled = if status == "ok" || status == "failed" {
        WIDTH
    } else if status.starts_with("running") {
        WIDTH / 2
    } else {
        0
    };
    let running = status.starts_with("running");

    let filled_s: String = "█".repeat(filled);
    let empty_s: String = "░".repeat(WIDTH.saturating_sub(filled));
    let empty_tail: String = if running && !empty_s.is_empty() {
        empty_s.chars().skip(1).collect()
    } else {
        empty_s.clone()
    };

    let mut spans = vec![
        Span::styled(format!("{}.", idx), Style::default().fg(state.theme.dim)),
        Span::raw(" "),
        Span::styled("[", Style::default().fg(state.theme.dim)),
        Span::styled(
            filled_s,
            Style::default()
                .fg(state.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if running && !empty_s.is_empty() {
        spans.push(Span::styled(
            "▌".to_string(),
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            empty_tail,
            Style::default().fg(state.theme.dim),
        ));
    } else {
        spans.push(Span::styled(empty_s, Style::default().fg(state.theme.dim)));
    }

    spans.push(Span::styled("]", Style::default().fg(state.theme.dim)));
    spans.push(Span::raw(" "));
    let percent = if WIDTH == 0 {
        0
    } else {
        (filled * 100) / WIDTH
    };
    spans.push(Span::styled(
        format!("{:>3}%", percent),
        Style::default().fg(state.theme.dim),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        label.to_string(),
        Style::default().fg(state.theme.fg),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        status.to_string(),
        Style::default()
            .fg(if status == "ok" {
                state.theme.accent
            } else if status == "failed" {
                state.theme.accent2
            } else {
                state.theme.accent2
            })
            .add_modifier(Modifier::BOLD),
    ));
    Line::from(spans)
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
            provider = Some(
                part.trim_start_matches("provider=")
                    .trim_matches(',')
                    .to_string(),
            );
        }
        if part.starts_with("model=") {
            model = Some(
                part.trim_start_matches("model=")
                    .trim_matches(',')
                    .to_string(),
            );
        }
    }
    (provider, model)
}

fn build_args(state: &AppState, action: &str) -> Vec<String> {
    let Some(file) = &state.selected_file else {
        return vec![action.to_string()];
    };
    let target = if let Some(meta) = &state.meta_target {
        Some(meta.clone())
    } else {
        state
            .target_state
            .selected()
            .and_then(|i| state.targets.get(i).cloned())
    };
    let mut args = vec![action.to_string(), file.to_string_lossy().to_string()];
    if action != "clean" {
        if let Some(t) = target {
            args.push("--target".to_string());
            args.push(t);
        }
    }
    if action == "build" || action == "freeze" {
        if let Some(policy) = state.preview_meta.get("nd_policy") {
            args.push("--nd-policy".to_string());
            args.push(policy.clone());
        }
    }
    args
}

fn can_run_for_selected(state: &AppState) -> bool {
    let Some(target) = state.active_target() else {
        return false;
    };
    let Some(file) = &state.selected_file else {
        return false;
    };
    let dist_dir = dist_dir_for(file);
    match target.as_str() {
        "cli" => dist_dir.join("main.js").exists(),
        "web" => dist_dir.join("index.html").exists(),
        "gui" => {
            dist_dir.join("gui/.build/release/SculptGui").exists()
                || dist_dir.join("gui/main.py").exists()
        }
        _ => true,
    }
}

fn selected_output_stale(state: &AppState) -> bool {
    let Some(target) = state.active_target() else {
        return false;
    };
    let Some(file) = &state.selected_file else {
        return false;
    };
    if state.selected_kind != Some(SelectedKind::Script) {
        return false;
    }
    let src_mtime = match fs::metadata(file).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let dist_dir = dist_dir_for(file);
    let artifact = match target.as_str() {
        "cli" => dist_dir.join("main.js"),
        "web" => dist_dir.join("index.html"),
        "gui" => {
            let native = dist_dir.join("gui/.build/release/SculptGui");
            if native.exists() {
                native
            } else {
                dist_dir.join("gui/main.py")
            }
        }
        _ => return false,
    };
    let out_mtime = match fs::metadata(&artifact).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true,
    };
    src_mtime > out_mtime
}

fn preflight_issue(state: &AppState) -> Option<String> {
    let Some(target) = state.active_target() else {
        return Some("No target selected.".to_string());
    };
    let Some(file) = &state.selected_file else {
        return Some("No file selected.".to_string());
    };
    if state.selected_kind == Some(SelectedKind::Script)
        && file.extension().and_then(|s| s.to_str()) != Some("sculpt")
    {
        return Some("Script mode requires a .sculpt file.".to_string());
    }
    if target == "gui" && !state.preview_meta.get("target").is_some_and(|t| t == "gui") {
        return None;
    }
    let provider = state
        .config
        .provider
        .clone()
        .unwrap_or_else(|| "stub".to_string());
    if matches!(
        state.pending_action,
        PendingAction::BuildOnly | PendingAction::BuildRun
    ) && !provider_has_auth(state, &provider)
    {
        return Some(format!(
            "Preflight failed: provider '{}' has no API key configured (press G to open config).",
            provider
        ));
    }
    None
}

fn provider_has_auth(state: &AppState, provider: &str) -> bool {
    match provider {
        "openai" => {
            std::env::var("OPENAI_API_KEY").is_ok()
                || state
                    .config
                    .openai
                    .as_ref()
                    .and_then(|c| c.api_key.clone())
                    .is_some()
        }
        "anthropic" => {
            std::env::var("ANTHROPIC_API_KEY").is_ok()
                || state
                    .config
                    .anthropic
                    .as_ref()
                    .and_then(|c| c.api_key.clone())
                    .is_some()
        }
        "gemini" => {
            std::env::var("GEMINI_API_KEY").is_ok()
                || state
                    .config
                    .gemini
                    .as_ref()
                    .and_then(|c| c.api_key.clone())
                    .is_some()
        }
        _ => true,
    }
}

fn dist_dir_for(path: &Path) -> PathBuf {
    dist_dir_for_input(path)
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
        Span::styled(
            "SCULPT ",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Compiler ",
            Style::default()
                .fg(state.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            env!("CARGO_PKG_VERSION"),
            Style::default().fg(state.theme.accent),
        ),
    ];
    if include_right {
        spans.push(Span::raw(spacer));
        spans.push(Span::styled(
            right_plain,
            Style::default().fg(state.theme.dim),
        ));
    }
    Line::from(spans)
}

fn normalize_log_output(bytes: &[u8]) -> Vec<String> {
    if bytes.is_empty() {
        return Vec::new();
    }
    String::from_utf8_lossy(bytes)
        .lines()
        .map(strip_ansi_and_controls)
        .filter(|line| !line.trim().is_empty())
        .collect()
}

fn strip_ansi_and_controls(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if let Some('[') = chars.peek().copied() {
                let _ = chars.next();
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
            continue;
        }
        if ch.is_control() {
            continue;
        }
        out.push(ch);
    }
    out
}

fn render_target_modal(f: &mut ratatui::Frame, state: &mut AppState) {
    let area = centered_rect(32, 24, f.size());
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default().style(Style::default().bg(state.theme.panel_bg)),
        area,
    );
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4)].as_ref())
        .split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "Select Target",
        Style::default()
            .fg(state.theme.accent)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1))
            .style(Style::default().bg(state.theme.panel_bg)),
    );
    f.render_widget(title, chunks[0]);

    let body = chunks[1];

    let targets = if state.meta_target.is_some() {
        vec![ListItem::new(Line::from(Span::raw(format!(
            "{} (locked)",
            state.meta_target.as_ref().unwrap()
        ))))]
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
                .title(Span::styled(
                    "[Target]",
                    Style::default().fg(state.theme.fg),
                ))
                .padding(Padding::horizontal(1))
                .style(Style::default().bg(state.theme.panel_bg)),
        )
        .highlight_style(
            Style::default()
                .bg(state.theme.highlight_bg)
                .fg(state.theme.fg)
                .add_modifier(Modifier::BOLD),
        );
    if state.modal_focus != ModalFocus::Targets || state.meta_target.is_some() {
        tlist = tlist.highlight_style(
            Style::default()
                .bg(state.theme.panel_bg)
                .fg(state.theme.dim),
        );
    }
    let mut tstate = state.target_state.clone();
    f.render_stateful_widget(tlist, body, &mut tstate);
    state.target_state = tstate;
}

fn render_config_modal(f: &mut ratatui::Frame, state: &mut AppState) {
    let area = centered_rect(70, 70, f.size());
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default().style(Style::default().bg(state.theme.panel_bg)),
        area,
    );
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "Configuration",
        Style::default()
            .fg(state.theme.accent)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1))
            .style(Style::default().bg(state.theme.panel_bg)),
    );
    f.render_widget(title, chunks[0]);

    let lines = render_config_lines(state);
    let body = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1))
                .style(Style::default().bg(state.theme.panel_bg)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(body, chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            "↑↓",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" field  "),
        Span::styled(
            "←→",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" provider  "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" edit/apply inline  "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(state.theme.accent2)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" close"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1))
            .style(Style::default().bg(state.theme.panel_bg)),
    );
    f.render_widget(footer, chunks[2]);
}

fn render_info_modal(f: &mut ratatui::Frame, state: &AppState, msg: &str) {
    let area = centered_rect(36, 18, f.size());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(state.theme.panel_bg));
    let text = Paragraph::new(Line::from(vec![Span::styled(
        msg,
        Style::default().fg(state.theme.accent),
    )]))
    .block(block);
    f.render_widget(text, area);
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
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

fn config_fields() -> [ConfigField; 10] {
    [
        ConfigField::Provider,
        ConfigField::OpenAiKey,
        ConfigField::OpenAiModel,
        ConfigField::AnthropicKey,
        ConfigField::AnthropicModel,
        ConfigField::GeminiKey,
        ConfigField::GeminiModel,
        ConfigField::Save,
        ConfigField::Test,
        ConfigField::Cancel,
    ]
}

fn is_editable_config_field(field: ConfigField) -> bool {
    matches!(
        field,
        ConfigField::OpenAiKey
            | ConfigField::OpenAiModel
            | ConfigField::AnthropicKey
            | ConfigField::AnthropicModel
            | ConfigField::GeminiKey
            | ConfigField::GeminiModel
    )
}

fn config_move_field(state: &mut AppState, delta: i32) {
    let fields = config_fields();
    let mut idx = fields
        .iter()
        .position(|f| *f == state.config_field)
        .unwrap_or(0) as i32;
    idx = (idx + delta).clamp(0, (fields.len() - 1) as i32);
    state.config_field = fields[idx as usize];
}

fn config_select_provider(state: &mut AppState, delta: i32) {
    if state.config_field != ConfigField::Provider {
        return;
    }
    let providers = ["openai", "anthropic", "gemini", "stub"];
    let current = state
        .config
        .provider
        .clone()
        .unwrap_or_else(|| "stub".to_string());
    let mut idx = providers.iter().position(|p| *p == current).unwrap_or(3) as i32;
    idx = (idx + delta).rem_euclid(providers.len() as i32);
    state.config.provider = Some(providers[idx as usize].to_string());
}

fn current_config_field_value(state: &AppState) -> String {
    match state.config_field {
        ConfigField::OpenAiKey => state
            .config
            .openai
            .as_ref()
            .and_then(|c| c.api_key.clone())
            .unwrap_or_default(),
        ConfigField::OpenAiModel => state
            .config
            .openai
            .as_ref()
            .and_then(|c| c.model.clone())
            .unwrap_or_default(),
        ConfigField::AnthropicKey => state
            .config
            .anthropic
            .as_ref()
            .and_then(|c| c.api_key.clone())
            .unwrap_or_default(),
        ConfigField::AnthropicModel => state
            .config
            .anthropic
            .as_ref()
            .and_then(|c| c.model.clone())
            .unwrap_or_default(),
        ConfigField::GeminiKey => state
            .config
            .gemini
            .as_ref()
            .and_then(|c| c.api_key.clone())
            .unwrap_or_default(),
        ConfigField::GeminiModel => state
            .config
            .gemini
            .as_ref()
            .and_then(|c| c.model.clone())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn apply_config_input(state: &mut AppState) {
    let v = state.config_input.clone();
    match state.config_field {
        ConfigField::OpenAiKey => {
            let mut cfg = state.config.openai.clone().unwrap_or_default();
            cfg.api_key = if v.is_empty() { None } else { Some(v) };
            state.config.openai = Some(cfg);
        }
        ConfigField::OpenAiModel => {
            let mut cfg = state.config.openai.clone().unwrap_or_default();
            cfg.model = if v.is_empty() { None } else { Some(v) };
            state.config.openai = Some(cfg);
        }
        ConfigField::AnthropicKey => {
            let mut cfg = state.config.anthropic.clone().unwrap_or_default();
            cfg.api_key = if v.is_empty() { None } else { Some(v) };
            state.config.anthropic = Some(cfg);
        }
        ConfigField::AnthropicModel => {
            let mut cfg = state.config.anthropic.clone().unwrap_or_default();
            cfg.model = if v.is_empty() { None } else { Some(v) };
            state.config.anthropic = Some(cfg);
        }
        ConfigField::GeminiKey => {
            let mut cfg = state.config.gemini.clone().unwrap_or_default();
            cfg.api_key = if v.is_empty() { None } else { Some(v) };
            state.config.gemini = Some(cfg);
        }
        ConfigField::GeminiModel => {
            let mut cfg = state.config.gemini.clone().unwrap_or_default();
            cfg.model = if v.is_empty() { None } else { Some(v) };
            state.config.gemini = Some(cfg);
        }
        _ => {}
    }
}

fn render_config_lines(state: &AppState) -> Vec<Line<'static>> {
    let selected = state.config_field;
    let mut lines = Vec::new();
    let provider = state
        .config
        .provider
        .clone()
        .unwrap_or_else(|| "stub".to_string());
    lines.push(config_line(
        state,
        ConfigField::Provider,
        "Default Provider".to_string(),
        format!("< {} >", provider),
        selected,
        false,
    ));
    lines.push(config_line(
        state,
        ConfigField::OpenAiKey,
        "OpenAI API Key".to_string(),
        masked(
            &state
                .config
                .openai
                .as_ref()
                .and_then(|c| c.api_key.clone())
                .unwrap_or_default(),
        ),
        selected,
        true,
    ));
    lines.push(config_line(
        state,
        ConfigField::OpenAiModel,
        "OpenAI Model".to_string(),
        state
            .config
            .openai
            .as_ref()
            .and_then(|c| c.model.clone())
            .unwrap_or_else(|| "gpt-4.1".to_string()),
        selected,
        true,
    ));
    lines.push(config_line(
        state,
        ConfigField::AnthropicKey,
        "Anthropic API Key".to_string(),
        masked(
            &state
                .config
                .anthropic
                .as_ref()
                .and_then(|c| c.api_key.clone())
                .unwrap_or_default(),
        ),
        selected,
        true,
    ));
    lines.push(config_line(
        state,
        ConfigField::AnthropicModel,
        "Anthropic Model".to_string(),
        state
            .config
            .anthropic
            .as_ref()
            .and_then(|c| c.model.clone())
            .unwrap_or_else(|| "claude-3-7-sonnet-latest".to_string()),
        selected,
        true,
    ));
    lines.push(config_line(
        state,
        ConfigField::GeminiKey,
        "Gemini API Key".to_string(),
        masked(
            &state
                .config
                .gemini
                .as_ref()
                .and_then(|c| c.api_key.clone())
                .unwrap_or_default(),
        ),
        selected,
        true,
    ));
    lines.push(config_line(
        state,
        ConfigField::GeminiModel,
        "Gemini Model".to_string(),
        state
            .config
            .gemini
            .as_ref()
            .and_then(|c| c.model.clone())
            .unwrap_or_else(|| "gemini-2.5-pro".to_string()),
        selected,
        true,
    ));
    lines.push(Line::from(""));
    lines.push(config_line(
        state,
        ConfigField::Save,
        "[Save]".to_string(),
        "".to_string(),
        selected,
        false,
    ));
    lines.push(config_line(
        state,
        ConfigField::Test,
        "[Test Provider Auth]".to_string(),
        "".to_string(),
        selected,
        false,
    ));
    lines.push(config_line(
        state,
        ConfigField::Cancel,
        "[Cancel]".to_string(),
        "".to_string(),
        selected,
        false,
    ));
    lines
}

fn config_line(
    state: &AppState,
    field: ConfigField,
    label: String,
    value: String,
    selected: ConfigField,
    has_value_box: bool,
) -> Line<'static> {
    let is_selected = field == selected;
    let marker = if is_selected { ">" } else { " " };
    let style = if is_selected {
        Style::default()
            .fg(state.theme.accent2)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(state.theme.fg)
    };
    if value.is_empty() || !has_value_box {
        Line::from(vec![Span::styled(format!("{} {}", marker, label), style)])
    } else {
        let editing_here = is_selected && state.config_editing && is_editable_config_field(field);
        let display_value = if editing_here {
            state.config_input.clone()
        } else {
            value
        };
        const BOX_WIDTH: usize = 40;
        let clipped: String = display_value
            .chars()
            .take(BOX_WIDTH.saturating_sub(1))
            .collect();
        let mut box_value = clipped.clone();
        if editing_here {
            box_value.push('▌');
        }
        let current_len = box_value.chars().count();
        if current_len < BOX_WIDTH {
            box_value.push_str(&" ".repeat(BOX_WIDTH - current_len));
        }

        let value_style = if editing_here {
            Style::default()
                .fg(state.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(state.theme.fg)
        } else {
            Style::default().fg(state.theme.dim)
        };

        Line::from(vec![
            Span::styled(format!("{} {:18}", marker, label), style),
            Span::styled("│", Style::default().fg(state.theme.accent2)),
            Span::styled(box_value, value_style),
            Span::styled("│", Style::default().fg(state.theme.accent2)),
        ])
    }
}

fn masked(value: &str) -> String {
    if value.is_empty() {
        return "<empty>".to_string();
    }
    if value.len() <= 8 {
        return "*".repeat(value.len());
    }
    let head = &value[..4];
    let tail = &value[value.len() - 4..];
    format!("{}{}{}", head, "*".repeat(8), tail)
}

fn load_tui_config(path: &Path) -> TuiConfig {
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(cfg) = serde_json::from_str::<TuiConfig>(&data) {
            return cfg;
        }
    }
    TuiConfig::default()
}

fn save_tui_config(path: &Path, cfg: &TuiConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(cfg)?;
    fs::write(path, json)?;
    Ok(())
}

#[derive(Default, Deserialize)]
struct TuiProjectFile {
    name: Option<String>,
    entry: Option<String>,
    modules: Vec<String>,
}

fn extract_project_preview_and_meta(
    path: &Path,
) -> Result<(BTreeMap<String, String>, Option<ProjectPreview>)> {
    let content = fs::read_to_string(path)?;
    let parsed: TuiProjectFile = serde_json::from_str(&content)?;
    let name = parsed
        .name
        .clone()
        .or_else(|| {
            path.file_name()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_suffix(".sculpt.json"))
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "project".to_string());
    let entry = parsed.entry.clone().unwrap_or_default();
    let preview = ProjectPreview {
        name,
        entry: entry.clone(),
        modules: parsed.modules.clone(),
    };

    let mut meta = BTreeMap::new();
    if !entry.is_empty() {
        let base = path.parent().unwrap_or(Path::new("."));
        for rel in &parsed.modules {
            let p = base.join(rel);
            let src = fs::read_to_string(&p).unwrap_or_default();
            if let Ok(module_json) = extract_module_name_and_meta(&src) {
                if module_json.0 == entry {
                    meta = module_json.1;
                    break;
                }
            }
        }
    }
    Ok((meta, Some(preview)))
}

fn extract_module_name_and_meta(content: &str) -> Result<(String, BTreeMap<String, String>)> {
    let mut meta = BTreeMap::new();
    let mut module_name = String::new();
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("@meta") {
            let rest = t.trim_start_matches("@meta").trim();
            for part in rest.split_whitespace() {
                if let Some(eq) = part.find('=') {
                    let (k, v) = part.split_at(eq);
                    let val = v.trim_start_matches('=').trim_matches('"');
                    meta.insert(k.to_string(), val.to_string());
                }
            }
            continue;
        }
        if t.starts_with("module(") {
            if let Some(end) = t.find(')') {
                module_name = t[7..end].to_string();
            }
            break;
        }
    }
    Ok((module_name, meta))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn temp_case_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("sculpt_tui_{name}_{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn script_file(dir: &Path, name: &str) -> PathBuf {
        let p = dir.join(name);
        fs::write(&p, "module(Test)\nend\n").unwrap();
        p
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn test_state(dir: &Path, selected: &Path) -> AppState {
        let mut file_state = ListState::default();
        file_state.select(Some(0));
        let mut target_state = ListState::default();
        target_state.select(Some(0));

        AppState {
            cwd: dir.to_path_buf(),
            sculpt_cmd: PathBuf::from("/usr/bin/true"),
            entries: vec![Entry {
                name: selected
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("app.sculpt")
                    .to_string(),
                path: selected.to_path_buf(),
                is_dir: false,
                is_sculpt: true,
                is_project: false,
            }],
            file_state,
            targets: vec!["cli".to_string(), "gui".to_string(), "web".to_string()],
            target_state,
            focus: Focus::Files,
            details_scroll: 0,
            log_scroll: 0,
            log: vec![],
            history: vec![],
            status: "Ready".to_string(),
            selected_file: None,
            selected_kind: None,
            meta_target: None,
            preview_meta: BTreeMap::new(),
            preview_lines: 0,
            preview_size: 0,
            preview_intro: vec![],
            project_preview: None,
            last_run: None,
            last_refresh: Instant::now(),
            theme: Theme::dark(),
            modal_open: false,
            active_modal: ActiveModal::Target,
            modal_focus: ModalFocus::Targets,
            info_modal: None,
            pending_action: PendingAction::BuildRun,
            config: TuiConfig::default(),
            config_path: dir.join("sculpt.config.json"),
            config_field: ConfigField::Provider,
            config_editing: false,
            config_input: String::new(),
            needs_full_redraw: false,
        }
    }

    #[test]
    fn strip_ansi_sequences() {
        let input = "\u{1b}[38;2;234;81;114mError:\u{1b}[0m failed";
        assert_eq!(strip_ansi_and_controls(input), "Error: failed");
    }

    #[test]
    fn normalize_output_sanitizes_lines() {
        let bytes = b"\x1b[31mError\x1b[0m\nok\n";
        assert_eq!(
            normalize_log_output(bytes),
            vec!["Error".to_string(), "ok".to_string()]
        );
    }

    #[test]
    fn enter_on_selected_script_opens_target_modal() {
        let dir = temp_case_dir("enter_modal");
        let file = script_file(&dir, "enter_modal.sculpt");
        let mut state = test_state(&dir, &file);

        let should_quit = handle_key(&mut state, key(KeyCode::Enter)).unwrap();

        assert!(!should_quit);
        assert!(state.modal_open);
        assert!(matches!(state.active_modal, ActiveModal::Target));
        assert!(matches!(state.pending_action, PendingAction::BuildRun));
        assert_eq!(state.selected_file.as_deref(), Some(file.as_path()));
    }

    #[test]
    fn b_key_opens_target_modal_for_build_only() {
        let dir = temp_case_dir("build_only_modal");
        let file = script_file(&dir, "build_only_modal.sculpt");
        let mut state = test_state(&dir, &file);

        handle_key(&mut state, key(KeyCode::Char('b'))).unwrap();

        assert!(state.modal_open);
        assert!(matches!(state.pending_action, PendingAction::BuildOnly));
        assert!(matches!(state.active_modal, ActiveModal::Target));
    }

    #[test]
    fn r_key_shows_info_modal_when_run_not_available() {
        let dir = temp_case_dir("run_missing_artifact");
        let file = script_file(&dir, "run_missing_artifact.sculpt");
        let mut state = test_state(&dir, &file);

        handle_key(&mut state, key(KeyCode::Char('r'))).unwrap();
        assert!(state.modal_open);

        handle_key(&mut state, key(KeyCode::Enter)).unwrap();
        assert!(!state.modal_open);
        assert_eq!(
            state.info_modal.as_deref(),
            Some("Run not available. Build first.")
        );
    }

    #[test]
    fn f_p_c_keys_execute_commands_for_selected_script() {
        let dir = temp_case_dir("fpc_commands");
        let file = script_file(&dir, "fpc_commands.sculpt");
        let mut state = test_state(&dir, &file);

        handle_key(&mut state, key(KeyCode::Char('f'))).unwrap();
        handle_key(&mut state, key(KeyCode::Char('p'))).unwrap();
        handle_key(&mut state, key(KeyCode::Char('c'))).unwrap();

        assert_eq!(state.history.len(), 3);
        assert!(state.history[0].starts_with("freeze "));
        assert!(state.history[1].starts_with("replay "));
        assert!(state.history[2].starts_with("clean "));
    }

    #[test]
    fn esc_closes_target_modal_without_quitting() {
        let dir = temp_case_dir("modal_esc");
        let file = script_file(&dir, "modal_esc.sculpt");
        let mut state = test_state(&dir, &file);
        state.modal_open = true;
        state.active_modal = ActiveModal::Target;

        let should_quit = handle_key(&mut state, key(KeyCode::Esc)).unwrap();

        assert!(!should_quit);
        assert!(!state.modal_open);
    }

    #[test]
    fn enter_with_meta_target_buildrun_executes_without_modal() {
        let dir = temp_case_dir("meta_target_enter");
        let file = script_file(&dir, "meta_target_enter.sculpt");
        fs::write(&file, "@meta target=cli\nmodule(Test)\nend\n").unwrap();
        let dist = dist_dir_for(&file);
        fs::create_dir_all(&dist).unwrap();
        fs::write(dist.join("main.js"), "console.log('ok')").unwrap();

        let mut state = test_state(&dir, &file);
        state.meta_target = Some("cli".to_string());

        handle_key(&mut state, key(KeyCode::Enter)).unwrap();

        assert!(!state.modal_open);
        assert!(state.history.iter().any(|h| h.starts_with("run ")));
    }
}

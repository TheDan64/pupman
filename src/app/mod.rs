use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::mpsc::{self, Sender};
use std::thread;

use color_eyre::eyre::{OptionExt, eyre};
use compact_str::CompactString;
use crossterm::event::Event as CrosstermEvent;
use log::{error, warn};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) mod event;
mod state;
pub(crate) mod ui;

use event::{AppEvent, Event, EventHandler, FileSystemChangeKind};
use state::State;
use tui_logger::TuiWidgetEvent;
use ui::{Finding, FindingKind, IdMapEntry};

use crate::fs;
use crate::fs::monitor::{MonitorHandler, is_valid_file};
use crate::fs::subid::{ETC_SUBGID, ETC_SUBUID, SubID};
use crate::lxc::config::Config;
use crate::lxc::rootfs_value_to_path;
use crate::metadata::Metadata;

pub struct App {
    metadata: Metadata,
    // infra: Infrastructure,
    monitor: MonitorHandler,
    event_handler: EventHandler,
    fs_reader_tx: Sender<PathBuf>,
    state: State,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(metadata: Metadata) -> Self {
        let event_handler = EventHandler::new();
        let (fs_tx, fs_rx) = mpsc::channel();
        let app_tx = event_handler.sender();

        thread::spawn(|| fs::reader::start(fs_rx, app_tx));

        Self {
            fs_reader_tx: fs_tx.clone(),
            monitor: MonitorHandler::new(event_handler.sender(), fs_tx, &metadata.lxc_config_dir).expect("Fixme"),
            metadata,
            event_handler,
            state: State::default(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.initialize()?;

        while self.state.is_running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.event_handler.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => {
                if let CrosstermEvent::Key(key_event) = event {
                    self.handle_key_event(key_event)?;
                }
            },
            Event::App(app_event) => match app_event {
                AppEvent::FileSystemChanged(change_kind) => {
                    match change_kind {
                        FileSystemChangeKind::Remove(path) => self.unload_container_id_map(&path)?,
                        FileSystemChangeKind::Update(path, content) => {
                            if path.starts_with(&self.metadata.lxc_config_dir) {
                                self.load_container_id_map(&path, &content)?;
                            } else if path == Path::new(ETC_SUBUID) {
                                self.load_subid(&content, SubID::UID)?;
                            } else if path == Path::new(ETC_SUBGID) {
                                self.load_subid(&content, SubID::GID)?;
                            }
                        },
                    };

                    self.state.evaluate_findings();
                },
                AppEvent::Quit => self.quit(),
            },
        }
        Ok(())
    }

    fn load_container_id_map(&mut self, path: &Path, content: &str) -> color_eyre::Result<()> {
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| eyre!("Invalid file name"))?;
        let config = Config::from_str(content)?;
        let section = config.section(None);

        if let Some(rootfs_value) = section.get_rootfs() {
            match rootfs_value_to_path(rootfs_value) {
                Ok(path) => self.monitor.watch_rootfs(&path)?,
                Err(err) => {
                    error!("Failed to convert rootfs value {rootfs_value} to path for load: {err:?}");
                },
            };
        }

        self.state.lxc_configs.insert(CompactString::new(filename), config);
        self.state.lxc_configs.sort_unstable_keys();

        Ok(())
    }

    fn unload_container_id_map(&mut self, path: &Path) -> color_eyre::Result<()> {
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| eyre!("Invalid file name"))?;
        let Some(config) = self.state.lxc_configs.shift_remove(filename) else {
            warn!("Attempted to unload container ID map for non-existent file: {filename}");
            return Ok(());
        };
        let section = config.section(None);

        if let Some(rootfs) = section.get_rootfs() {
            if self.state.rootfs_info.shift_remove(rootfs).is_none() {
                warn!("Attempted to unload rootfs info for non-existent file: {filename}");
                return Ok(());
            };
        }

        Ok(())
    }

    fn load_subid(&mut self, content: &str, subid: SubID) -> color_eyre::Result<()> {
        let id_map = parse_subid_map(content)?;

        match subid {
            SubID::UID => self.state.host_mapping.subuid = id_map,
            SubID::GID => self.state.host_mapping.subgid = id_map,
        }

        Ok(())
    }

    fn initialize(&mut self) -> color_eyre::Result<()> {
        self.fs_reader_tx.send(PathBuf::from(ETC_SUBUID))?;
        self.fs_reader_tx.send(PathBuf::from(ETC_SUBGID))?;

        for entry in read_dir(&self.metadata.lxc_config_dir)? {
            let path = entry?.path();

            if is_valid_file(&path) {
                self.fs_reader_tx.send(path)?;
            }
        }

        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        // If the fix popup is shown, handle the key events for the fix popup.
        if self.state.show_fix_popup {
            match key_event.code {
                KeyCode::Esc => self.state.show_fix_popup = false,
                _ => {},
            }

            return Ok(());
        }

        // If the settings page is shown, handle the key events for the settings page.
        if self.state.show_settings_page {
            match key_event.code {
                KeyCode::Esc => self.state.show_settings_page = false,
                _ => {},
            }

            return Ok(());
        }

        // If the logs page is shown, handle the key events for the logger page.
        if self.state.show_logs_page {
            let state = &self.state.logger_page_state;

            match key_event.code {
                KeyCode::Esc => self.state.show_logs_page = false,
                KeyCode::Char(' ') => state.transition(TuiWidgetEvent::SpaceKey),
                KeyCode::Char('q') => state.transition(TuiWidgetEvent::EscapeKey),
                KeyCode::PageUp => state.transition(TuiWidgetEvent::PrevPageKey),
                KeyCode::PageDown => state.transition(TuiWidgetEvent::NextPageKey),
                KeyCode::Up => state.transition(TuiWidgetEvent::UpKey),
                KeyCode::Down => state.transition(TuiWidgetEvent::DownKey),
                KeyCode::Left => state.transition(TuiWidgetEvent::LeftKey),
                KeyCode::Right => state.transition(TuiWidgetEvent::RightKey),
                KeyCode::Char('+') => state.transition(TuiWidgetEvent::PlusKey),
                KeyCode::Char('-') => state.transition(TuiWidgetEvent::MinusKey),
                KeyCode::Char('h') => state.transition(TuiWidgetEvent::HideKey),
                KeyCode::Char('f') => state.transition(TuiWidgetEvent::FocusKey),
                _ => {},
            }

            return Ok(());
        }

        // Handle the key events for the main application.
        match key_event.code {
            // TODO: Prompt for confirmation before quitting. Esc should cancel the prompt for consistency.
            // Enter or y to confirm quitting.
            KeyCode::Esc => self.event_handler.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.event_handler.send(AppEvent::Quit)
            },
            KeyCode::Char('f') if !self.state.show_fix_popup => {
                if let Some(finding) = self.selected_finding() {
                    if finding.kind == FindingKind::Bad {
                        self.state.show_fix_popup = true;
                    }
                }
            },
            KeyCode::Char('l') => {
                self.state.show_logs_page = true;
            },
            KeyCode::Char('s') => {
                self.state.show_settings_page = true;
            },
            KeyCode::Up => {
                if self.state.findings.is_empty() {
                    return Ok(());
                }

                if let Some(index) = self.state.selected_finding {
                    if index > 0 {
                        self.state.selected_finding = Some(index - 1);
                    } else {
                        self.state.selected_finding = None;
                    }
                } else {
                    self.state.selected_finding = Some(self.state.findings.len() - 1);
                }
            },
            KeyCode::Down => {
                if self.state.findings.is_empty() {
                    return Ok(());
                }

                if let Some(index) = self.state.selected_finding {
                    if index < self.state.findings.len() - 1 {
                        self.state.selected_finding = Some(index + 1);
                    } else {
                        self.state.selected_finding = None;
                    }
                } else {
                    self.state.selected_finding = Some(0);
                }
            },
            KeyCode::PageUp => {
                if self.state.findings.is_empty() {
                    return Ok(());
                }

                self.state.selected_finding = Some(0);
            },
            KeyCode::PageDown => {
                if self.state.findings.is_empty() {
                    return Ok(());
                }

                self.state.selected_finding = Some(self.state.findings.len() - 1);
            },
            _ => {},
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.state.is_running = false;
    }

    fn selected_finding(&self) -> Option<&Finding> {
        self.state
            .selected_finding
            .and_then(|index| self.state.findings.get(index))
    }
}

fn parse_subid_map(content: &str) -> color_eyre::Result<Vec<IdMapEntry>> {
    let mut id_map = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let mut iter = trimmed.split(':');
        let host_user_id = CompactString::new(iter.next().ok_or_eyre("user id not found")?);
        let host_sub_id: u32 = iter.next().ok_or_eyre("host sub id not found")?.parse()?;
        let host_sub_id_count: u32 = iter
            .next()
            .ok_or_eyre("host sub id host_sub_id_count not found")?
            .parse()?;

        id_map.push(IdMapEntry {
            host_user_id,
            host_sub_id,
            host_sub_id_count,
        });
    }

    Ok(id_map)
}

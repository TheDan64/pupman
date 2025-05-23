use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::mpsc::{self, Sender};
use std::thread;

use color_eyre::eyre::{OptionExt, eyre};
use crossterm::event::Event as CrosstermEvent;
use indexmap::IndexMap;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) mod event;
pub(crate) mod ui;

use event::{AppEvent, Event, EventHandler, FileSystemChangeKind};
use ui::{Finding, HostMapping, IdMapEntry};

use crate::fs;
use crate::fs::monitor::{MonitorHandler, is_valid_file};
use crate::fs::subid::{ETC_SUBGID, ETC_SUBUID, SubID};
use crate::proxmox::lxc::{self, Config};

#[derive(Debug)]
pub struct App {
    is_running: bool,
    lxc_config_dir: PathBuf,
    _monitor: MonitorHandler,
    event_handler: EventHandler,
    findings: Vec<Finding>,
    selected_finding: Option<usize>,
    host_mapping: HostMapping,
    fs_reader_tx: Sender<PathBuf>,
    lxc_configs: IndexMap<String, Config>,
}

impl Default for App {
    fn default() -> Self {
        Self::new(Path::new(lxc::CONF_DIR))
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(lxc_config_dir: &Path) -> Self {
        let event_handler = EventHandler::new();

        let (fs_tx, fs_rx) = mpsc::channel();
        let app_tx = event_handler.sender();

        thread::spawn(|| fs::reader::start(fs_rx, app_tx));

        Self {
            is_running: true,
            fs_reader_tx: fs_tx.clone(),
            _monitor: MonitorHandler::new(event_handler.sender(), fs_tx, lxc_config_dir).expect("Fixme"),
            lxc_config_dir: lxc_config_dir.to_path_buf(),
            event_handler,
            findings: Vec::new(),
            selected_finding: None,
            host_mapping: HostMapping {
                subuid: Vec::new(),
                subgid: Vec::new(),
            },
            lxc_configs: IndexMap::new(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.initialize()?;

        while self.is_running {
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
                            if path.starts_with(&self.lxc_config_dir) {
                                self.load_container_id_map(&path, &content)?;
                            } else if path == Path::new(ETC_SUBUID) {
                                self.load_subid(&content, SubID::SubUID)?;
                            } else if path == Path::new(ETC_SUBGID) {
                                self.load_subid(&content, SubID::SubGID)?;
                            }
                        },
                    };

                    self.evaluate_findings();
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
            .ok_or_else(|| eyre!("Invalid file name"))?
            .to_string();

        let config = Config::from_str(content)?;

        self.lxc_configs.insert(filename.clone(), config.clone());
        // self.lxc_configs.sort_unstable_keys();

        Ok(())
    }

    fn unload_container_id_map(&mut self, path: &Path) -> color_eyre::Result<()> {
        Err(eyre!("TODO: Unload container id map from path: {path:?}"))
    }

    fn load_subid(&mut self, content: &str, subid: SubID) -> color_eyre::Result<()> {
        let id_map = parse_subid_map(&content)?;

        match subid {
            SubID::SubUID => self.host_mapping.subuid = id_map,
            SubID::SubGID => self.host_mapping.subgid = id_map,
        }

        Ok(())
    }

    fn initialize(&mut self) -> color_eyre::Result<()> {
        self.fs_reader_tx.send(PathBuf::from(ETC_SUBUID))?;
        self.fs_reader_tx.send(PathBuf::from(ETC_SUBGID))?;

        for entry in read_dir(&self.lxc_config_dir)? {
            let path = entry?.path();

            if is_valid_file(&path) {
                self.fs_reader_tx.send(path)?;
            }
        }

        Ok(())
    }

    /// Findings are re-evaluated based on latest update
    fn evaluate_findings(&mut self) {
        // TODO: Implement the logic to evaluate findings based on the current state of the application.
        // This is where you would compare the host mapping with the container mappings and
        // determine if there are any discrepancies or issues.
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            // TODO: Esc should back out of popups and such rather than quitting
            KeyCode::Esc | KeyCode::Char('q') => self.event_handler.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.event_handler.send(AppEvent::Quit)
            },
            KeyCode::Up => {
                if self.findings.is_empty() {
                    return Ok(());
                }

                if let Some(index) = self.selected_finding {
                    if index > 0 {
                        self.selected_finding = Some(index - 1);
                    } else {
                        self.selected_finding = None;
                    }
                } else {
                    self.selected_finding = Some(self.findings.len() - 1);
                }
            },
            KeyCode::Down => {
                if self.findings.is_empty() {
                    return Ok(());
                }

                if let Some(index) = self.selected_finding {
                    if index < self.findings.len() - 1 {
                        self.selected_finding = Some(index + 1);
                    } else {
                        self.selected_finding = None;
                    }
                } else {
                    self.selected_finding = Some(0);
                }
            },
            KeyCode::PageUp => {
                if self.findings.is_empty() {
                    return Ok(());
                }

                self.selected_finding = Some(0);
            },
            KeyCode::PageDown => {
                if self.findings.is_empty() {
                    return Ok(());
                }

                self.selected_finding = Some(self.findings.len() - 1);
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
        self.is_running = false;
    }

    fn selected_finding(&self) -> Option<&Finding> {
        self.selected_finding.and_then(|index| self.findings.get(index))
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
        let host_user_id = iter.next().ok_or_eyre("user id not found")?.to_owned();
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

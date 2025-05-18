use std::path::Path;

use crossterm::event::Event as CrosstermEvent;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) mod event;
pub(crate) mod ui;

use event::{AppEvent, Event, EventHandler};
use ui::{ContainerIdMaps, Finding, FindingKind, HostMapping, IdMapEntry};

use crate::fs::monitor::MonitorHandler;
use crate::fs::{ETC_SUBGID, ETC_SUBUID};
use crate::proxmox::lxc;

#[derive(Debug)]
pub struct App {
    is_running: bool,
    _monitor: MonitorHandler,
    event_handler: EventHandler,
    findings: Vec<Finding>,
    selected_finding: Option<usize>,
    host_mapping: HostMapping,
    container_mappings: Vec<ContainerIdMaps>,
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

        Self {
            is_running: true,
            _monitor: MonitorHandler::new(event_handler.sender(), lxc_config_dir).expect("Fixme"),
            event_handler,
            findings: Vec::new(),
            selected_finding: None,
            host_mapping: HostMapping {
                subuid: vec![
                    IdMapEntry {
                        kind: "UID".to_string(),
                        container_id: 0,
                        host_id: 100000,
                        size: 65536,
                    },
                    IdMapEntry {
                        kind: "UID".to_string(),
                        container_id: 65536,
                        host_id: 100000 + 65536,
                        size: 4294967295 - 65536,
                    },
                ],
                subgid: vec![
                    IdMapEntry {
                        kind: "GID".to_string(),
                        container_id: 0,
                        host_id: 100000,
                        size: 65536,
                    },
                    IdMapEntry {
                        kind: "GID".to_string(),
                        container_id: 65536,
                        host_id: 100000 + 65536,
                        size: 4294967295 - 65536,
                    },
                ],
            },
            container_mappings: vec![ContainerIdMaps {
                filename: "100.conf".to_string(),
                uid_maps: vec![
                    IdMapEntry {
                        kind: "UID".to_string(),
                        container_id: 0,
                        host_id: 100000,
                        size: 65536,
                    },
                    IdMapEntry {
                        kind: "UID".to_string(),
                        container_id: 65536,
                        host_id: 100000 + 65536,
                        size: 4294967295 - 65536,
                    },
                ],
                gid_maps: vec![
                    IdMapEntry {
                        kind: "GID".to_string(),
                        container_id: 0,
                        host_id: 100000,
                        size: 65536,
                    },
                    IdMapEntry {
                        kind: "GID".to_string(),
                        container_id: 65536,
                        host_id: 100000 + 65536,
                        size: 4294967295 - 65536,
                    },
                ],
            }],
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.is_running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.event_handler.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                CrosstermEvent::Key(key_event) => self.handle_key_event(key_event)?,
                _ => {},
            },
            Event::App(app_event) => match app_event {
                AppEvent::FileSystemChanged(change_kind, path) => {
                    match path.to_str() {
                        Some(ETC_SUBUID) => {},
                        Some(ETC_SUBGID) => {},
                        Some(conf) => match change_kind {
                            event::FileSystemChangeKind::Remove => todo!(),
                            event::FileSystemChangeKind::Update => todo!(),
                        },
                        // TODO: Log as warning?
                        None => return Ok(()),
                    };

                    self.evaluate_findings();
                },
                AppEvent::Quit => self.quit(),
            },
        }
        Ok(())
    }

    /// Findings are re-evaluated based on latest update
    fn evaluate_findings(&mut self) {
        self.findings = vec![
            Finding {
                kind: FindingKind::Good,
                host_mapping_highlights: vec![0, 3],
                container_id_mapping_highlights: vec![1],
            },
            Finding {
                kind: FindingKind::Bad,
                host_mapping_highlights: vec![1, 3],
                container_id_mapping_highlights: vec![0],
            },
            Finding {
                kind: FindingKind::Good,
                host_mapping_highlights: vec![1],
                container_id_mapping_highlights: vec![0, 1],
            },
        ];
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
            // Other handlers you could add here.
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

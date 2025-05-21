use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify::{
    Config, Event as NotifyEvent, EventHandler, EventKind, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher,
};

use super::subid::{ETC_SUBGID, ETC_SUBUID};
use crate::app::event::{AppEvent, Event, FileSystemChangeKind};

pub fn is_valid_file(path: &Path) -> bool {
    if path == Path::new(ETC_SUBGID) || path == Path::new(ETC_SUBUID) {
        return true;
    }

    match path.file_name().and_then(|f| f.to_str()) {
        Some(filename) if filename.ends_with(".conf") => {
            let prefix = &filename[..filename.len() - 5];
            !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit())
        },
        _ => false,
    }
}

pub struct FileEventHandler {
    app_tx: Sender<Event>,
    fs_tx: Sender<PathBuf>,
}

impl FileEventHandler {
    pub fn new(app_tx: Sender<Event>, fs_tx: Sender<PathBuf>) -> Self {
        Self { app_tx, fs_tx }
    }
}

impl EventHandler for FileEventHandler {
    fn handle_event(&mut self, event: Result<NotifyEvent, notify::Error>) {
        if let Ok(event) = event {
            for path in event.paths {
                if !is_valid_file(&path) {
                    continue;
                }

                match event.kind {
                    EventKind::Create(CreateKind::File) | EventKind::Modify(ModifyKind::Data(_)) => {
                        self.fs_tx.send(path).expect("fixme");
                    },
                    // REVIEW: Not sure if (re)name is correct:
                    EventKind::Modify(ModifyKind::Name(_)) | EventKind::Remove(RemoveKind::File) => {
                        self.app_tx
                            .send(Event::App(AppEvent::FileSystemChanged(FileSystemChangeKind::Remove(
                                path,
                            ))))
                            .expect("fixme");
                    },
                    _ => continue,
                };
            }
        }
    }
}

#[derive(Debug)]
pub struct MonitorHandler {
    _watcher: INotifyWatcher,
}

impl MonitorHandler {
    pub fn new(app_tx: Sender<Event>, fs_tx: Sender<PathBuf>, lxc_config_dir: &Path) -> notify::Result<Self> {
        let event_handler = FileEventHandler { app_tx, fs_tx };
        let mut watcher = RecommendedWatcher::new(event_handler, Config::default())?;

        watcher.watch(Path::new(ETC_SUBGID), RecursiveMode::NonRecursive)?;
        watcher.watch(Path::new(ETC_SUBUID), RecursiveMode::NonRecursive)?;
        watcher.watch(lxc_config_dir, RecursiveMode::Recursive)?;

        Ok(Self { _watcher: watcher })
    }
}

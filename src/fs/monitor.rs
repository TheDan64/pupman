use std::path::Path;
use std::sync::mpsc::Sender;

use notify::{
    Config, Event as NotifyEvent, EventHandler, EventKind, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher,
    event::{CreateKind, ModifyKind, RemoveKind},
};

use crate::app::event::{AppEvent, Event, FileSystemChangeKind};

const ETC_SUBGID: &str = "/etc/subgid";
const ETC_SUBUID: &str = "/etc/subuid";

fn is_valid_file(path: &Path) -> bool {
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
    tx: Sender<Event>,
}

impl FileEventHandler {
    pub fn new(tx: Sender<Event>) -> Self {
        Self { tx }
    }
}

impl EventHandler for FileEventHandler {
    fn handle_event(&mut self, event: Result<NotifyEvent, notify::Error>) {
        if let Ok(event) = event {
            for path in event.paths {
                if !is_valid_file(&path) {
                    continue;
                }

                let change_kind = match event.kind {
                    EventKind::Create(CreateKind::File) => FileSystemChangeKind::Update,
                    EventKind::Modify(ModifyKind::Data(_)) => FileSystemChangeKind::Update,
                    // REVIEW: Not sure if this one is correct:
                    EventKind::Modify(ModifyKind::Name(_)) => FileSystemChangeKind::Remove,
                    EventKind::Remove(RemoveKind::File) => FileSystemChangeKind::Remove,
                    _ => continue,
                };

                // TODO: Log on error
                self.tx
                    .send(Event::App(AppEvent::FileSystemChanged(change_kind, path)))
                    .expect("not to fail");
            }
        }
    }
}

#[derive(Debug)]
pub struct MonitorHandler {
    _watcher: INotifyWatcher,
}

impl MonitorHandler {
    pub fn new(tx: Sender<Event>, lxc_config_dir: &Path) -> notify::Result<Self> {
        let event_handler = FileEventHandler { tx };
        let mut watcher = RecommendedWatcher::new(event_handler, Config::default())?;

        watcher.watch(Path::new(ETC_SUBGID), RecursiveMode::NonRecursive)?;
        watcher.watch(Path::new(ETC_SUBUID), RecursiveMode::NonRecursive)?;
        watcher.watch(lxc_config_dir, RecursiveMode::Recursive)?;

        Ok(Self { _watcher: watcher })
    }
}

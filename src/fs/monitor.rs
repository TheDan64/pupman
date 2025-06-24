use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::thread::sleep;
use std::time::Duration;
use std::{fs, thread};

use log::{debug, error};
use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify::{
    Config, Event as NotifyEvent, EventHandler, EventKind, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher,
};

use super::subid::{ETC_SUBGID, ETC_SUBUID};
use crate::app::event::{AppEvent, Event, FileSystemChangeKind};
use crate::lxc::rootfs_value_to_path;

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
    file_tx: Sender<PathBuf>,
}

impl FileEventHandler {
    pub fn new(app_tx: Sender<Event>, file_tx: Sender<PathBuf>) -> Self {
        Self { app_tx, file_tx }
    }
}

impl EventHandler for FileEventHandler {
    fn handle_event(&mut self, event: Result<NotifyEvent, notify::Error>) {
        if let Ok(event) = event {
            for path in &event.paths {
                if !is_valid_file(path) {
                    continue;
                }

                match &event.kind {
                    EventKind::Create(CreateKind::File) | EventKind::Modify(ModifyKind::Data(_)) => {
                        if self.file_tx.send(path.clone()).is_err() {
                            error!("Failed to send file system change event {:?} for {path:?}", event.kind);
                        }
                    },
                    // REVIEW: Not sure if (re)name is correct:
                    EventKind::Modify(ModifyKind::Name(_)) | EventKind::Remove(RemoveKind::File) => {
                        if self
                            .app_tx
                            .send(Event::App(AppEvent::FileSystemChanged(
                                FileSystemChangeKind::RemoveFile(path.clone()),
                            )))
                            .is_err()
                        {
                            error!("Failed to send file system change event {:?} for {path:?}", event.kind);
                        }
                    },
                    _ => {
                        debug!("Unsupported file system change kind: {event:?}");

                        continue;
                    },
                };
            }
        }
    }
}

/// The handler for the file system monitor.
// It turns out that Linux and INotify don't support notifications when owner / group
// changes, so we need a secondary poller to detect that change.
#[derive(Debug)]
pub struct MonitorHandler {
    /// Watches all files: `/etc/subuid`, `/etc/subgid`, and the LXC config directory.
    _file_watcher: INotifyWatcher,
    /// Sender to watch all rootfs owner/group changes.
    dir_watcher_tx: Sender<String>,
}

impl MonitorHandler {
    pub fn new(app_tx: Sender<Event>, file_tx: Sender<PathBuf>, lxc_config_dir: &Path) -> notify::Result<Self> {
        let event_handler = FileEventHandler {
            app_tx: app_tx.clone(),
            file_tx,
        };
        let mut file_watcher = RecommendedWatcher::new(event_handler, Config::default())?;

        file_watcher.watch(Path::new(ETC_SUBGID), RecursiveMode::NonRecursive)?;
        file_watcher.watch(Path::new(ETC_SUBUID), RecursiveMode::NonRecursive)?;
        file_watcher.watch(lxc_config_dir, RecursiveMode::Recursive)?;

        let (dir_watcher_tx, dir_watcher_rx) = mpsc::channel::<String>();

        thread::spawn(move || {
            let mut paths = HashMap::new();

            loop {
                match dir_watcher_rx.try_recv() {
                    Ok(rootfs_value) => {
                        let path = match rootfs_value_to_path(&rootfs_value) {
                            Ok(path) => path,
                            Err(err) => {
                                error!("Failed to convert rootfs value {rootfs_value} to path for load: {err:?}");
                                continue;
                            },
                        };
                        let md = match fs::metadata(&path) {
                            Ok(md) => md,
                            Err(err) => {
                                error!("Failed to monitor metadata for {}: {err:?}", path.display());
                                continue;
                            },
                        };

                        paths.insert(path.clone(), (rootfs_value.clone(), md.clone()));
                        if app_tx
                            .send(Event::App(AppEvent::FileSystemChanged(
                                FileSystemChangeKind::UpdateDir(rootfs_value, path, md),
                            )))
                            .is_err()
                        {
                            error!("Failed to send initial UpdateDir event");
                        }
                    },
                    Err(TryRecvError::Empty) => (),
                    Err(TryRecvError::Disconnected) => panic!("RootFS ownership watcher died unexpectedly!"),
                };

                sleep(Duration::from_secs(5));

                for (path, (rootfs_value, old_md)) in &mut paths {
                    let md = match fs::metadata(path) {
                        Ok(md) => md,
                        Err(err) => {
                            error!("Failed to monitor metadata in loop for {}: {err:?}", path.display());
                            continue;
                        },
                    };

                    if md.gid() != old_md.gid() || md.uid() != old_md.uid() {
                        if app_tx
                            .send(Event::App(AppEvent::FileSystemChanged(
                                FileSystemChangeKind::UpdateDir(rootfs_value.clone(), path.clone(), md.clone()),
                            )))
                            .is_err()
                        {
                            error!("Failed to send UpdateDir event on change");
                        }
                        *old_md = md;
                    }
                }
            }
        });

        Ok(Self {
            _file_watcher: file_watcher,
            dir_watcher_tx,
        })
    }

    pub fn watch_rootfs(&mut self, rootfs_value: &str) -> notify::Result<()> {
        self.dir_watcher_tx.send(rootfs_value.to_owned())?;
        Ok(())
    }
}

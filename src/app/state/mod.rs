use std::collections::{HashMap, hash_map::Entry};
use std::fs::{self, Metadata};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use ahash::RandomState;
use compact_str::CompactString;
use indexmap::IndexMap;
use log::error;
use tui_logger::TuiWidgetState;

use super::ui::{Finding, FindingKind, HostMapping};
use crate::fs::subid::SubID;
use crate::linux::{groupname_to_id, username_to_id};
use crate::lxc::config::Config;
use crate::lxc::rootfs_value_to_path;

#[cfg(test)]
mod tests;

pub struct State {
    pub is_running: bool,
    pub findings: Vec<Finding>,
    pub selected_finding: Option<usize>,
    pub host_mapping: HostMapping,
    pub lxc_configs: IndexMap<CompactString, Config, RandomState>,
    pub rootfs_info: IndexMap<String, (PathBuf, Metadata), RandomState>,
    pub show_fix_popup: bool,
    pub show_settings_page: bool,
    pub show_logs_page: bool,
    pub logger_page_state: TuiWidgetState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            is_running: true,
            findings: Vec::new(),
            selected_finding: None,
            host_mapping: HostMapping {
                subuid: Vec::new(),
                subgid: Vec::new(),
            },
            lxc_configs: IndexMap::with_hasher(RandomState::new()),
            rootfs_info: IndexMap::with_hasher(RandomState::new()),
            show_fix_popup: false,
            show_settings_page: false,
            show_logs_page: false,
            logger_page_state: TuiWidgetState::default(),
        }
    }
}

impl State {
    /// Findings are re-evaluated based on latest update
    // TODO: Check for overlaps between configs
    pub fn evaluate_findings(&mut self) {
        self.findings.clear();

        let mut username_to_id_map = HashMap::with_hasher(RandomState::new());
        let mut groupname_to_id_map = HashMap::with_hasher(RandomState::new());
        let mut usernames: HashMap<_, (&CompactString, SubID), _> = HashMap::with_hasher(RandomState::new());
        let mut groupnames: HashMap<_, (&CompactString, SubID), _> = HashMap::with_hasher(RandomState::new());

        for mapping in &self.host_mapping.subuid {
            match usernames.entry(&mapping.host_user_id) {
                Entry::Occupied(occupancy) => {
                    let (user_id, sub_id) = *occupancy.get();

                    self.findings.push(Finding {
                        kind: FindingKind::Bad,
                        message: "Cannot have multiple entries for the same user",
                        host_mapping_highlights: vec![(user_id.clone(), sub_id)],
                        lxc_config_mapping_highlights: Vec::new(),
                        rootfs_highlights: Vec::new(),
                    });
                },
                Entry::Vacant(vacancy) => {
                    vacancy.insert((&mapping.host_user_id, SubID::UID));
                },
            };
        }

        for mapping in &self.host_mapping.subgid {
            match groupnames.entry(&mapping.host_user_id) {
                Entry::Occupied(occupancy) => {
                    let (user_id, sub_id) = *occupancy.get();

                    self.findings.push(Finding {
                        kind: FindingKind::Bad,
                        message: "Cannot have multiple entries for the same group",
                        host_mapping_highlights: vec![(user_id.clone(), sub_id)],
                        lxc_config_mapping_highlights: Vec::new(),
                        rootfs_highlights: Vec::new(),
                    });
                },
                Entry::Vacant(vacancy) => {
                    vacancy.insert((&mapping.host_user_id, SubID::GID));
                },
            };
        }

        if !self
            .findings
            .iter()
            .any(|f| f.message.starts_with("Cannot have multiple entries for the same"))
        {
            self.findings.push(Finding {
                kind: FindingKind::Good,
                message: "No duplicate ids found in subuid/subgid mappings",
                // TODO: Highlight all entries?
                host_mapping_highlights: Vec::new(),
                lxc_config_mapping_highlights: Vec::new(),
                rootfs_highlights: Vec::new(),
            });
        }

        for (filename, config) in &self.lxc_configs {
            let section = config.section(None);

            if section.get_unprivileged() != Some("1") {
                continue;
            }

            let rootfs = section.get_rootfs().and_then(|rootfs_value| {
                let path = match rootfs_value_to_path(rootfs_value) {
                    Ok(path) => path,
                    Err(err) => {
                        error!("Failed to convert rootfs value {rootfs_value} to path: {err}");
                        return None;
                    },
                };
                match fs::metadata(&path) {
                    Ok(metadata) => Some((rootfs_value, metadata)),
                    Err(err) => {
                        error!("Failed to get metadata for path {path:?}: {err}");
                        None
                    },
                }
            });

            let mut has_user_idmap = false;
            let mut has_group_idmap = false;

            for idmap in section.get_lxc_idmaps() {
                let mut idmap = idmap.trim().split(' ');
                let Some(kind) = idmap.next() else {
                    unreachable!("Invalid ID map entry kind");
                };
                let Some(host_id) = idmap.next() else {
                    unreachable!("Invalid ID map entry host user id");
                };
                let parsed_host_id = host_id.parse::<u32>().unwrap();
                let Some(host_sub_id) = idmap.next() else {
                    unreachable!("Invalid ID map entry host sub id");
                };
                let parsed_host_sub_id = host_sub_id.parse::<u32>().unwrap();
                let Some(host_sub_id_size) = idmap.next() else {
                    unreachable!("Invalid ID map entry host sub id count");
                };
                let parsed_host_sub_id_size = host_sub_id_size.parse::<u32>().unwrap();
                let (idmap, mappings, to_id) = if kind == "u" {
                    has_user_idmap = true;

                    (
                        &mut username_to_id_map,
                        &*self.host_mapping.subuid,
                        username_to_id as fn(&str) -> color_eyre::Result<u32>,
                    )
                } else if kind == "g" {
                    has_group_idmap = true;

                    (
                        &mut groupname_to_id_map,
                        &*self.host_mapping.subgid,
                        groupname_to_id as _,
                    )
                } else {
                    unreachable!("Invalid sub id kind")
                };

                if let Some((value, metadata)) = &rootfs {
                    if kind == "u" && metadata.uid() != parsed_host_sub_id {
                        self.findings.push(Finding {
                            kind: FindingKind::Bad,
                            message: "Rootfs uid does not match host mapping",
                            host_mapping_highlights: Vec::new(),
                            lxc_config_mapping_highlights: vec![(filename.clone(), SubID::UID)],
                            rootfs_highlights: vec![value.to_string()],
                        });
                    }

                    if kind == "g" && metadata.gid() != parsed_host_sub_id {
                        self.findings.push(Finding {
                            kind: FindingKind::Bad,
                            message: "Rootfs gid does not match host mapping",
                            host_mapping_highlights: Vec::new(),
                            lxc_config_mapping_highlights: vec![(filename.clone(), SubID::GID)],
                            rootfs_highlights: vec![value.to_string()],
                        });
                    }
                }

                for mapping in mappings {
                    let host_id = match idmap.entry(&mapping.host_user_id) {
                        Entry::Occupied(id) => *id.get(),
                        Entry::Vacant(vacancy) => {
                            let id = match to_id(&mapping.host_user_id) {
                                Ok(id) => id,
                                Err(err) => {
                                    error!("Failed to parse id for {kind} {}: {err:?}", mapping.host_user_id);
                                    continue;
                                },
                            };
                            *vacancy.insert(id)
                        },
                    };

                    if host_id != parsed_host_id {
                        continue;
                    }

                    if parsed_host_sub_id < mapping.host_sub_id
                        || parsed_host_sub_id > mapping.host_sub_id + mapping.host_sub_id_count
                        || parsed_host_sub_id + parsed_host_sub_id_size
                            > mapping.host_sub_id + mapping.host_sub_id_count
                    {
                        let (message, sub_id) = if kind == "u" {
                            (
                                "LXC config's host sub uid range outside of host mapping range",
                                SubID::UID,
                            )
                        } else {
                            (
                                "LXC config's host sub gid range outside of host mapping range",
                                SubID::GID,
                            )
                        };

                        self.findings.push(Finding {
                            kind: FindingKind::Bad,
                            message,
                            host_mapping_highlights: vec![(mapping.host_user_id.clone(), sub_id)],
                            lxc_config_mapping_highlights: vec![(filename.clone(), sub_id)],
                            rootfs_highlights: Vec::new(),
                        });
                    }
                }
            }

            // TODO: This still needs a test
            if !has_user_idmap {
                self.findings.push(Finding {
                    kind: FindingKind::Bad,
                    message: "lxc.idmap for uid is not set in config",
                    host_mapping_highlights: Vec::new(),
                    lxc_config_mapping_highlights: vec![(filename.clone(), SubID::UID)],
                    rootfs_highlights: Vec::new(),
                });
            }

            // TODO: This still needs a test
            if !has_group_idmap {
                self.findings.push(Finding {
                    kind: FindingKind::Bad,
                    message: "lxc.idmap for gid is not set in config",
                    host_mapping_highlights: Vec::new(),
                    lxc_config_mapping_highlights: vec![(filename.clone(), SubID::GID)],
                    rootfs_highlights: Vec::new(),
                });
            }
        }

        self.findings.sort_by_key(|f| f.kind != FindingKind::Bad);
    }
}

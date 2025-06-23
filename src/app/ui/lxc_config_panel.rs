use std::path::Path;

use ahash::RandomState;
use compact_str::CompactString;
use indexmap::IndexMap;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Row, Table, Widget};

use crate::app::ui::Finding;
use crate::fs::subid::SubID;
use crate::lxc::config::Config;

pub struct LXCConfigPanel<'a> {
    configs: &'a IndexMap<CompactString, Config, RandomState>,
    selected_finding: Option<&'a Finding>,
    lxc_config_dir: &'a Path,
}

impl<'a> LXCConfigPanel<'a> {
    pub fn new(
        configs: &'a IndexMap<CompactString, Config, RandomState>,
        selected_finding: Option<&'a Finding>,
        lxc_config_dir: &'a Path,
    ) -> Self {
        Self {
            configs,
            selected_finding,
            lxc_config_dir,
        }
    }
}

impl Widget for LXCConfigPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header = Row::new([
            Text::from("Config").alignment(Alignment::Center),
            Text::from("Kind").alignment(Alignment::Center),
            Text::from("ID").alignment(Alignment::Center),
            Text::from("Sub ID").alignment(Alignment::Center),
            Text::from("Sub ID Size").alignment(Alignment::Center),
            Text::from("Sub ID Range").alignment(Alignment::Center),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let mut rows = Vec::new();

        for (filename, config) in self.configs {
            let section = config.section(None);

            if section.get_unprivileged() != Some("1") {
                continue;
            }

            let mut first = true;
            let mut has_user_idmap = false;
            let mut has_group_idmap = false;

            for idmap in section.get_lxc_idmaps() {
                let filename_display = if first {
                    first = false;
                    filename
                } else {
                    ""
                };

                let mut idmap = idmap.trim().split(' ');
                let Some(kind) = idmap.next() else {
                    unreachable!("Invalid ID map entry kind");
                };
                let Some(host_user_id) = idmap.next() else {
                    unreachable!("Invalid ID map entry host user id");
                };
                let Some(host_sub_id) = idmap.next() else {
                    unreachable!("Invalid ID map entry host sub id");
                };
                let Some(host_sub_id_size) = idmap.next() else {
                    unreachable!("Invalid ID map entry host sub id count");
                };
                let sub_id = if kind == "u" {
                    has_user_idmap = true;
                    SubID::UID
                } else if kind == "g" {
                    has_group_idmap = true;
                    SubID::GID
                } else {
                    unreachable!("Invalid ID map entry kind");
                };

                let mut style = Style::default();

                if let Some(finding) = self.selected_finding {
                    if finding
                        .lxc_config_mapping_highlights
                        .contains(&(filename.clone(), sub_id))
                    {
                        style = style.bg(finding.selected_bg()).fg(Color::Black);
                    }
                }

                rows.push(
                    Row::new([
                        Text::from(filename_display).alignment(Alignment::Center),
                        Text::from(if kind == "u" { "UID" } else { "GID" }).alignment(Alignment::Center),
                        Text::from(host_user_id).alignment(Alignment::Center),
                        Text::from(host_sub_id.to_string()).alignment(Alignment::Center),
                        Text::from(host_sub_id_size).alignment(Alignment::Center),
                        Text::from(format!(
                            "{host_sub_id} → {}",
                            host_sub_id.parse::<u32>().expect("fixme")
                                + host_sub_id_size.parse::<u32>().expect("fixme")
                                - 1
                        ))
                        .alignment(Alignment::Center),
                    ])
                    .style(style),
                );
            }

            let mut first = true;

            if !has_user_idmap {
                first = false;

                let mut style = Style::default();

                if let Some(finding) = self.selected_finding {
                    if finding
                        .lxc_config_mapping_highlights
                        .contains(&(filename.clone(), SubID::UID))
                    {
                        style = style.bg(finding.selected_bg()).fg(Color::Black);
                    }
                }

                rows.push(
                    Row::new([
                        Text::from(&**filename).alignment(Alignment::Center),
                        Text::from("UID").alignment(Alignment::Center),
                        Text::from("?").alignment(Alignment::Center),
                        Text::from("?").alignment(Alignment::Center),
                        Text::from("?").alignment(Alignment::Center),
                        Text::from("? → ?").alignment(Alignment::Center),
                    ])
                    .style(style),
                );
            }

            if !has_group_idmap {
                let filename_display = if first { &**filename } else { "" };

                let mut style = Style::default();

                if let Some(finding) = self.selected_finding {
                    if finding
                        .lxc_config_mapping_highlights
                        .contains(&(filename.clone(), SubID::GID))
                    {
                        style = style.bg(finding.selected_bg()).fg(Color::Black);
                    }
                }

                rows.push(
                    Row::new([
                        Text::from(filename_display).alignment(Alignment::Center),
                        Text::from("GID").alignment(Alignment::Center),
                        Text::from("?").alignment(Alignment::Center),
                        Text::from("?").alignment(Alignment::Center),
                        Text::from("?").alignment(Alignment::Center),
                        Text::from("? → ?").alignment(Alignment::Center),
                    ])
                    .style(style),
                );
            }
        }

        let block = Block::default()
            .title(format!("LXC Mappings ({})", self.lxc_config_dir.display()))
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);

        Table::new(rows, &[]).header(header).block(block).render(area, buf);
    }
}

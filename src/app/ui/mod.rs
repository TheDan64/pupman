use super::App;
use footer::{Footer, FooterItem};
use logs_page::LogsPage;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect, Rows};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Row, Table, Widget};
use tui_widgets::popup::Popup;

use std::fmt::Display;
use std::iter::repeat;

mod findings_list;
mod footer;
mod logs_page;

use findings_list::FindingsList;

impl Widget for &App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        let host = &self.state.host_mapping;
        let configs = &self.state.lxc_configs;
        let outer_block = Block::bordered()
            .title("Proxmox UnPrivileged Manager")
            .title_alignment(Alignment::Center)
            .borders(Borders::TOP)
            .border_type(BorderType::Rounded);

        outer_block.clone().render(area, buf);

        let inner_area = outer_block.inner(area);

        if inner_area.height < 1 || inner_area.width < 1 {
            return;
        }

        if self.state.show_logs_page {
            LogsPage::new(&self.state.logger_page_state).render(inner_area, buf);
            return;
        }

        if self.state.show_settings_page {
            // Render settings page
            Paragraph::new("Settings page is not yet implemented")
                .alignment(Alignment::Center)
                .render(inner_area, buf);
            return;
        }

        let [main_area, footer_area] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(inner_area);

        // Command Bar Footer

        let items = if self.state.show_fix_popup {
            vec![FooterItem::Key("Esc", "Back", Color::LightRed)]
        } else {
            // Esc: Quit  │  ↑↓: Navigate  e: Explain  f: Fix  |  s: Settings  l: Logs
            let mut items = vec![
                FooterItem::Key("Esc", "Quit", Color::LightRed),
                FooterItem::Div,
                FooterItem::Key("↑↓", "Navigate", Color::LightGreen),
            ];

            if self.selected_finding().is_some_and(|f| f.kind == FindingKind::Bad) {
                items.extend([
                    FooterItem::Key("e", "Explain", Color::LightCyan),
                    FooterItem::Key("f", "Fix", Color::Rgb(255, 102, 0)),
                ]);
            }

            items.extend([
                FooterItem::Div,
                FooterItem::Key("s", "Settings", Color::White),
                FooterItem::Key("l", "Logs", Color::White),
            ]);

            items
        };

        Footer::new(&items).render(footer_area, buf);

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(75), Constraint::Percentage(25)]).areas(main_area);
        let [host_area, container_area, rootfs_area] = Layout::vertical([
            Constraint::Length(3 + (host.subgid.len() + host.subuid.len()) as u16),
            Constraint::Min(2),
            Constraint::Length(3),
        ])
        .areas(left_area);

        let rootfs_header = Row::new([Text::from("Path").alignment(Alignment::Center)])
            .style(Style::default().add_modifier(Modifier::BOLD));

        // ── RootFS Table ──
        let mut rootfs_rows = Vec::new();

        for (rootfs, _info) in &self.state.rootfs_info {
            rootfs_rows.push(Row::new(vec![Text::from(&**rootfs).alignment(Alignment::Center)]));
        }

        Table::new(rootfs_rows, &[])
            .header(rootfs_header)
            .block(
                Block::default()
                    .title("RootFS")
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Center),
            )
            .render(rootfs_area, buf);

        let selected_finding = self.selected_finding();

        // ── Host Table ──
        let mut host_rows = Vec::new();

        let entries = host
            .subuid
            .iter()
            .zip(repeat("UID"))
            .chain(host.subgid.iter().zip(repeat("GID")))
            .enumerate();

        for (i, (entry, kind)) in entries {
            let mut style = Style::default();

            if let Some(finding) = selected_finding {
                if finding.host_mapping_highlights.contains(&i) {
                    style = style.bg(finding.selected_bg()).fg(Color::Black);
                }
            }

            host_rows.push(
                Row::new([
                    Text::from(kind).alignment(Alignment::Center),
                    Text::from(&*entry.host_user_id).alignment(Alignment::Center),
                    Text::from(entry.host_sub_id.to_string()).alignment(Alignment::Center),
                    Text::from(entry.host_sub_id_count.to_string()).alignment(Alignment::Center),
                    Text::from(format!(
                        "{} → {}",
                        entry.host_sub_id,
                        entry.host_sub_id + entry.host_sub_id_count - 1
                    ))
                    .alignment(Alignment::Center),
                ])
                .style(style),
            );
        }

        let host_header = Row::new([
            Text::from("Kind").alignment(Alignment::Center),
            Text::from("ID").alignment(Alignment::Center),
            Text::from("Sub ID").alignment(Alignment::Center),
            Text::from("Sub ID Size").alignment(Alignment::Center),
            Text::from("Sub ID Range").alignment(Alignment::Center),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let host_table = Table::new(
            host_rows,
            &[
                // Constraint::Length(4),
                // Constraint::Length(12),
                // Constraint::Length(12),
                // Constraint::Length(12),
            ],
        )
        .header(host_header)
        .block(
            Block::default()
                .title("Host Mappings (/etc/subuid /etc/subgid)")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center),
        );

        host_table.render(host_area, buf);

        // ── LXC Config Table ──
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

        for (i, (filename, config)) in configs.iter().enumerate() {
            let section = config.section(None);

            if section.get_unprivileged() != Some("1") {
                continue;
            }

            let mut first = true;

            for (j, idmap) in section.get_lxc_idmaps().enumerate() {
                let filename = if first {
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

                let mut style = Style::default();

                if let Some(finding) = selected_finding {
                    if finding.lxc_config_mapping_highlights.contains(&(i + j)) {
                        style = style.bg(finding.selected_bg()).fg(Color::Black);
                    }
                }

                rows.push(
                    Row::new([
                        Text::from(filename).alignment(Alignment::Center),
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
        }

        let block = Block::default()
            .title(format!("LXC Mappings ({})", self.metadata.lxc_config_dir.display()))
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);
        let constraints = [
            // Constraint::Length(20),
            // Constraint::Length(20),
            // Constraint::Length(12),
            // Constraint::Length(20),
            // Constraint::Length(12),
        ];

        Table::new(rows, &constraints)
            .header(header)
            .block(block)
            .render(container_area, buf);

        FindingsList::new(&self.state.findings, self.state.selected_finding).render(right_area, buf);

        if self.state.show_fix_popup {
            Popup::new(Text::from("Not yet implemented"))
                .title("Fix finding")
                // .style(Style::new().fg(Color::White).bg(Color::DarkGray)) // Normal
                .style(Style::new().fg(Color::LightRed).bg(Color::Rgb(48, 0, 0))) // Warning
                // .style(Style::new().fg(Color::LightGreen).bg(Color::Rgb(0, 48, 0))) // Success?
                .render(inner_area, buf);
        }
    }
}

// Data structures
#[derive(Debug)]
pub struct IdMapEntry {
    pub host_user_id: String,
    pub host_sub_id: u32,
    pub host_sub_id_count: u32,
}

#[derive(Debug)]
pub struct HostMapping {
    pub subuid: Vec<IdMapEntry>,
    pub subgid: Vec<IdMapEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FindingKind {
    Good,
    Bad,
}

#[derive(Clone, Debug)]
pub struct Finding {
    pub kind: FindingKind,
    pub message: &'static str,
    pub host_mapping_highlights: Vec<usize>,
    pub lxc_config_mapping_highlights: Vec<usize>,
}

impl Finding {
    fn base_fg(&self) -> Color {
        match self.kind {
            FindingKind::Good => Color::Green,
            FindingKind::Bad => Color::Red,
        }
    }

    fn selected_bg(&self) -> Color {
        match self.kind {
            FindingKind::Good => Color::LightGreen,
            FindingKind::Bad => Color::LightRed,
        }
    }

    fn badge(&self) -> &'static str {
        match self.kind {
            FindingKind::Good => "✅ ",
            FindingKind::Bad => "❌ ",
        }
    }
}

impl Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message)
    }
}

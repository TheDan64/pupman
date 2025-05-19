use super::App;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Row, Table, Widget};
use std::fmt::Display;

mod findings_list;

use findings_list::FindingsList;

impl Widget for &App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        let host = &self.host_mapping;
        let containers = &self.container_mappings;
        let outer_block = Block::bordered()
            .title("Proxmox UnPrivileged Manager")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        outer_block.clone().render(area, buf);

        let inner_area = outer_block.inner(area);

        if inner_area.height < 1 || inner_area.width < 1 {
            return;
        }

        let &[left_area, right_area] = &*Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner_area)
        else {
            unreachable!("Only two halves exist")
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3 + (host.subgid.len() + host.subuid.len()) as u16),
                Constraint::Min(0),
            ])
            .split(left_area);

        let selected_finding = self.selected_finding();

        // ── Host Table ──
        let mut host_rows = Vec::new();

        for (i, entry) in host.subuid.iter().chain(host.subgid.iter()).enumerate() {
            let mut style = Style::default();

            if let Some(finding) = selected_finding {
                if finding.host_mapping_highlights.contains(&i) {
                    style = style.bg(finding.kind.selected_bg()).fg(Color::Black);
                }
            }

            host_rows.push(
                Row::new([
                    Cell::from(&*entry.kind),
                    Cell::from(entry.host_id.to_string()),
                    Cell::from(entry.size.to_string()),
                ])
                .style(style),
            );
        }

        let host_header = Row::new([Cell::from("Kind"), Cell::from("Host ID"), Cell::from("Size")])
            .style(Style::default().add_modifier(Modifier::BOLD));

        let host_table = Table::new(
            host_rows,
            &[Constraint::Length(6), Constraint::Length(12), Constraint::Length(8)],
        )
        .header(host_header)
        .block(
            Block::default()
                .title("Host Root Mappings (/etc/subuid /etc/subgid)")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center),
        );

        host_table.render(chunks[0], buf);

        // ── Container Table ──
        let header = Row::new([
            Cell::from("Config"),
            Cell::from("UID Container ID"),
            Cell::from("UID Host ID"),
            Cell::from("GID Container ID"),
            Cell::from("GID Host ID"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let mut rows = Vec::new();

        for container in containers {
            let max = container.uid_maps.len().max(container.gid_maps.len());

            for i in 0..max {
                let mut style = Style::default();
                let uid = container.uid_maps.get(i);
                let gid = container.gid_maps.get(i);

                if let Some(finding) = selected_finding {
                    if finding.container_id_mapping_highlights.contains(&i) {
                        style = style.bg(finding.kind.selected_bg()).fg(Color::Black);
                    }
                }

                rows.push(
                    Row::new(vec![
                        Cell::from(if i == 0 { &*container.filename } else { "" }),
                        Cell::from(uid.map_or(String::new(), |e| e.container_id.to_string())),
                        Cell::from(uid.map_or(String::new(), |e| e.host_id.to_string())),
                        Cell::from(gid.map_or(String::new(), |e| e.container_id.to_string())),
                        Cell::from(gid.map_or(String::new(), |e| e.host_id.to_string())),
                    ])
                    .style(style),
                );
            }
        }

        let block = Block::default()
            .title(format!("Container ID Maps ({:?})", self.lxc_config_dir))
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);
        let constraints = [
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(20),
            Constraint::Length(12),
        ];
        let table = Table::new(rows, &constraints).header(header).block(block);

        table.render(chunks[1], buf);

        FindingsList::new(&self.findings, self.selected_finding).render(right_area, buf);
    }
}

// Data structures
#[derive(Debug)]
pub struct IdMapEntry {
    pub kind: String,
    pub container_id: u32,
    pub host_id: u32,
    pub size: u32,
}

#[derive(Debug)]
pub struct ContainerIdMaps {
    pub filename: String,
    pub uid_maps: Vec<IdMapEntry>,
    pub gid_maps: Vec<IdMapEntry>,
}

#[derive(Debug)]
pub struct HostMapping {
    pub subuid: Vec<IdMapEntry>,
    pub subgid: Vec<IdMapEntry>,
}

#[derive(Clone, Copy, Debug)]
pub enum FindingKind {
    Good,
    Bad,
}

impl FindingKind {
    fn base_fg(self) -> Color {
        match self {
            FindingKind::Good => Color::Green,
            FindingKind::Bad => Color::Red,
        }
    }

    fn selected_bg(self) -> Color {
        match self {
            FindingKind::Good => Color::LightGreen,
            FindingKind::Bad => Color::LightRed,
        }
    }

    fn badge(&self) -> &'static str {
        match self {
            FindingKind::Good => "✅ ",
            FindingKind::Bad => "❌ ",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Finding {
    pub kind: FindingKind,
    pub host_mapping_highlights: Vec<usize>,
    pub container_id_mapping_highlights: Vec<usize>,
}

impl Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            FindingKind::Good => write!(f, "Good Finding"),
            FindingKind::Bad => write!(f, "Bad Finding"),
        }
    }
}

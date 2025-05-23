use super::App;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Row, Table, Widget};
use std::fmt::Display;
use std::iter::repeat;

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
        let configs = &self.lxc_configs;
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

        let &[main_area, footer_area] = &*Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(inner_area)
        else {
            unreachable!("Only two areas exist")
        };

        // Command Bar Footer

        let spans = Line::from(vec![
            Span::styled("q", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
            Span::raw("uit  "),
            Span::styled("h", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
            Span::raw("elp  "),
            Span::styled("s", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
            Span::raw("ettings  "),
            Span::styled("f", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
            Span::raw("ix  "),
            Span::styled(
                "↑↓",
                Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" navigate"),
        ]);

        Paragraph::new(spans)
            // .style(Style::default().bg(Color::DarkGray))
            .alignment(Alignment::Center)
            .render(footer_area, buf);

        let &[left_area, right_area] = &*Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(main_area)
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
                .title("Host Root Mappings (/etc/subuid /etc/subgid)")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center),
        );

        host_table.render(chunks[0], buf);

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

        for (filename, config) in configs {
            let mut first = true;

            // TODO: We should pre-load all important config entries
            // rather than re-iterating every time.

            for idmap in config.sectionless_idmap() {
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

                rows.push(Row::new([
                    Text::from(filename).alignment(Alignment::Center),
                    Text::from(if kind == "u" { "UID" } else { "GID" }).alignment(Alignment::Center),
                    Text::from(host_user_id).alignment(Alignment::Center),
                    Text::from(host_sub_id.to_string()).alignment(Alignment::Center),
                    Text::from(host_sub_id_size).alignment(Alignment::Center),
                    Text::from(format!(
                        "{} → {}",
                        host_sub_id,
                        host_sub_id.parse::<u32>().expect("fixme") + host_sub_id_size.parse::<u32>().expect("fixme")
                            - 1
                    ))
                    .alignment(Alignment::Center),
                ]));
            }
        }

        let block = Block::default()
            .title(format!("Container ID Maps ({})", self.lxc_config_dir.display()))
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
            .render(chunks[1], buf);

        FindingsList::new(&self.findings, self.selected_finding).render(right_area, buf);
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

#[derive(Clone, Copy, Debug)]
pub enum FindingKind {
    Good,
    Bad,
}

#[derive(Clone, Debug)]
pub struct Finding {
    pub kind: FindingKind,
    pub host_mapping_highlights: Vec<usize>,
    pub container_id_mapping_highlights: Vec<usize>,
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
        match self.kind {
            FindingKind::Good => write!(f, "Good Finding"),
            FindingKind::Bad => write!(f, "Bad Finding"),
        }
    }
}

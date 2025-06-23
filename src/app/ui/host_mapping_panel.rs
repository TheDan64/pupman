use std::iter::repeat;

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Row, Table, Widget};

use crate::app::ui::{Finding, HostMapping};

pub struct HostMappingPanel<'a> {
    mapping: &'a HostMapping,
    selected_finding: Option<&'a Finding>,
}

impl<'a> HostMappingPanel<'a> {
    pub fn new(mapping: &'a HostMapping, selected_finding: Option<&'a Finding>) -> Self {
        Self {
            mapping,
            selected_finding,
        }
    }
}

impl Widget for HostMappingPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // ── Host Table ──
        let mut host_rows = Vec::new();

        let entries = self
            .mapping
            .subuid
            .iter()
            .zip(repeat("UID"))
            .chain(self.mapping.subgid.iter().zip(repeat("GID")))
            .enumerate();

        for (i, (entry, kind)) in entries {
            let mut style = Style::default();

            if let Some(finding) = self.selected_finding {
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

        Table::new(host_rows, &[])
            .header(host_header)
            .block(
                Block::default()
                    .title("Host Mappings (/etc/subuid /etc/subgid)")
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Center),
            )
            .render(area, buf);
    }
}

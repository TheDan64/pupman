use ahash::RandomState;
use color_eyre::owo_colors::OwoColorize;
use indexmap::IndexMap;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Row, Table, Widget};

use crate::app::ui::Finding;

pub struct RootFSPanel<'a> {
    info: &'a IndexMap<String, String, RandomState>,
    selected_finding: Option<&'a Finding>,
}

impl<'a> RootFSPanel<'a> {
    pub fn new(info: &'a IndexMap<String, String, RandomState>, selected_finding: Option<&'a Finding>) -> Self {
        Self { info, selected_finding }
    }
}

impl Widget for RootFSPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rootfs_header = Row::new([
            Text::from("Path").alignment(Alignment::Center),
            Text::from("Owner UID").alignment(Alignment::Center),
            Text::from("Owner GID").alignment(Alignment::Center),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));
        let mut rootfs_rows = Vec::new();

        for (rootfs, _info) in self.info {
            let mut style = Style::default();

            if let Some(finding) = self.selected_finding {
                if finding.rootfs_highlights.contains(rootfs) {
                    style = style.bg(finding.selected_bg()).fg(Color::Black);
                }
            }

            rootfs_rows.push(
                Row::new(vec![
                    Text::from(&**rootfs).alignment(Alignment::Center),
                    Text::from("TODO").alignment(Alignment::Center),
                    Text::from("TODO").alignment(Alignment::Center),
                ])
                .style(style),
            );
        }

        Table::new(rootfs_rows, &[])
            .header(rootfs_header)
            .block(
                Block::default()
                    .title("RootFS")
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Center),
            )
            .render(area, buf);
    }
}

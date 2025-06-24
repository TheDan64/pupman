use crate::app::ui::host_mapping_panel::HostMappingPanel;
use crate::app::ui::lxc_config_panel::LXCConfigPanel;
use crate::app::ui::rootfs_panel::RootFSPanel;
use crate::fs::subid::SubID;

use super::App;
use compact_str::CompactString;
use footer::{Footer, FooterItem};
use logs_page::LogsPage;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use tui_widgets::popup::Popup;

use std::fmt::Display;

mod findings_list;
mod footer;
mod host_mapping_panel;
mod logs_page;
mod lxc_config_panel;
mod rootfs_panel;

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

        let selected_finding = self.selected_finding();
        let [main_area, footer_area] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(inner_area);
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(75), Constraint::Percentage(25)]).areas(main_area);
        let [host_area, config_area, rootfs_area] = Layout::vertical([
            Constraint::Length(3 + (host.subgid.len() + host.subuid.len()) as u16),
            Constraint::Min(2),
            Constraint::Percentage(25),
        ])
        .areas(left_area);

        // Command Bar Footer

        let items = if self.state.show_fix_popup {
            vec![FooterItem::Key("Esc", "Back", Color::LightRed)]
        } else if self.state.show_explain_popup {
            vec![FooterItem::Key("Esc", "Back", Color::LightRed)]
        } else {
            // Esc: Quit  │  ↑↓: Navigate  e: Explain  f: Fix  |  s: Settings  l: Logs
            let mut items = vec![
                FooterItem::Key("Esc", "Quit", Color::LightRed),
                FooterItem::Div,
                FooterItem::Key("↑↓", "Navigate", Color::LightGreen),
            ];

            if selected_finding.is_some_and(|f| f.kind == FindingKind::Bad) {
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

        HostMappingPanel::new(&self.state.host_mapping, selected_finding).render(host_area, buf);
        LXCConfigPanel::new(&self.state.lxc_configs, selected_finding, &self.metadata.lxc_config_dir)
            .render(config_area, buf);
        RootFSPanel::new(&self.state.rootfs_info, selected_finding).render(rootfs_area, buf);
        FindingsList::new(&self.state.findings, self.state.selected_finding).render(right_area, buf);
        Footer::new(&items).render(footer_area, buf);

        if self.state.show_explain_popup {
            Popup::new(Text::from(
                "Not yet implemented. This will show detailed information about the selected finding.",
            ))
            .title("Explain finding")
            .style(Style::new().fg(Color::LightCyan).bg(Color::Rgb(0, 48, 48)))
            .render(inner_area, buf);
        }

        if self.state.show_fix_popup {
            Popup::new(Text::from("Not yet implemented. This will provide options to fix the selected finding."))
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
    pub host_user_id: CompactString,
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

// REVIEW: Vecs here should maybe be SmallVecs?
#[derive(Clone, Debug)]
pub struct Finding {
    pub kind: FindingKind,
    pub message: &'static str,
    pub host_mapping_highlights: Vec<(CompactString, SubID)>,
    pub lxc_config_mapping_highlights: Vec<(CompactString, SubID)>,
    pub rootfs_highlights: Vec<String>,
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

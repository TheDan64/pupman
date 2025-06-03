use ratatui::prelude::*;
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerSmartWidget, TuiWidgetState};

use super::footer::{Footer, FooterItem::*};

pub struct LogsPage<'s> {
    state: &'s TuiWidgetState,
}

impl<'s> LogsPage<'s> {
    pub fn new(state: &'s TuiWidgetState) -> Self {
        Self { state }
    }
}

impl Widget for LogsPage<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main_area, footer_area] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        // TODO: Footers widget

        TuiLoggerSmartWidget::default()
            .style_error(Style::default().fg(Color::Red))
            .style_debug(Style::default().fg(Color::Green))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_trace(Style::default().fg(Color::Magenta))
            .style_info(Style::default().fg(Color::Cyan))
            .output_separator(':')
            .output_timestamp(Some("%H:%M:%S".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
            .output_target(true)
            .output_file(true)
            .output_line(true)
            .state(self.state)
            .render(main_area, buf);

        let items = &[
            Key("Esc", "Back", Color::LightRed),
            Div,
            Key("↑↓", "Navigate", Color::LightGreen),
            Key("h", "Hide", Color::White),
            Key("f", "Focus", Color::White),
        ];

        Footer::new(items).render(footer_area, buf);
    }
}

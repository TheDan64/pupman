use super::Finding;
use ratatui::prelude::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};

#[derive(Clone, Copy, Debug)]
pub struct FindingsList<'f> {
    pub findings: &'f [Finding],
    pub selected: Option<usize>,
}

impl<'f> FindingsList<'f> {
    pub fn new(findings: &'f [Finding], selected: Option<usize>) -> Self {
        Self { findings, selected }
    }
}

impl<'a> Widget for FindingsList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Draw block around the list
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Findings")
            .border_style(Style::default().fg(Color::Gray));

        let inner_area = block.inner(area);

        block.render(area, buf);

        let max = self.findings.len().min(inner_area.height as usize);

        for (i, item) in self.findings.iter().take(max).enumerate() {
            let y = inner_area.y + i as u16;
            let is_selected = Some(i) == self.selected;
            let base_fg = item.kind.base_fg();
            let selected_bg = item.kind.selected_bg();
            let (fg, bg) = if is_selected {
                (Color::Black, selected_bg)
            } else {
                (base_fg, Color::Reset)
            };
            let style = Style::default().fg(fg).bg(bg).add_modifier(if is_selected {
                Modifier::BOLD
            } else {
                Modifier::empty()
            });
            let prefix = if is_selected { "→ " } else { "  " };
            let badge_content = item.kind.badge();
            let bullet = Span::styled(badge_content, Style::default().fg(base_fg));
            let content = Line::from(vec![Span::raw(prefix), bullet, Span::styled(item.to_string(), style)]);

            buf.set_line(inner_area.x, y, &content, inner_area.width);
        }
    }
}

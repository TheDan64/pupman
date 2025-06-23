use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

#[derive(Clone, Copy, Debug)]
pub enum FooterItem {
    Div,
    Key(&'static str, &'static str, Color),
}

#[derive(Clone, Copy, Debug)]
pub struct Footer<'f> {
    pub items: &'f [FooterItem],
}

impl<'f> Footer<'f> {
    pub fn new(items: &'f [FooterItem]) -> Self {
        Self { items }
    }
}

impl Widget for Footer<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut spans = Vec::with_capacity(self.items.len());

        for (i, item) in self.items.iter().enumerate() {
            match item {
                FooterItem::Div => spans.push(Span::raw("  ║")),
                FooterItem::Key(key, value, color) => {
                    if i != 0 {
                        spans.push(Span::raw("  "));
                    }

                    spans.push(Span::styled(*key, Style::default().fg(*color)).add_modifier(Modifier::BOLD));
                    spans.push(Span::raw(format!(": {value}")));
                },
            }
        }

        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}

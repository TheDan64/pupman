use ratatui::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Footer<'f> {
    pub items: &'f [()],
}

impl<'f> Footer<'f> {
    pub fn new(items: &'f [()]) -> Self {
        Self { items }
    }
}

impl<'a> Widget for Footer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {}
}

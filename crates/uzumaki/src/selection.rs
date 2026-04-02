#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SelectionRange {
    /// Anchor point (where selection started), flat grapheme index
    pub anchor: usize,
    /// Active point / cursor position, flat grapheme index
    pub active: usize,
}

impl SelectionRange {
    pub fn new(anchor: usize, active: usize) -> Self {
        Self { anchor, active }
    }

    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.active
    }

    pub fn start(&self) -> usize {
        self.anchor.min(self.active)
    }

    pub fn end(&self) -> usize {
        self.anchor.max(self.active)
    }

    pub fn set_cursor(&mut self, pos: usize) {
        self.anchor = pos;
        self.active = pos;
    }
}

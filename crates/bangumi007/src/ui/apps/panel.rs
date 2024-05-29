// ----------------------------------------------------------------------------
// Panels

#[derive(PartialEq, Eq)]
pub enum Panel {
    Library,
    Log,
    Settings,
}

impl Default for Panel {
    fn default() -> Self {
        Self::Library
    }
}

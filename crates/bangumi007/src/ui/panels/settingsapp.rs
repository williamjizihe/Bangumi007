// ----------------------------------------------------------------------------

use eframe::egui;

use crate::ui::panels::libraryapp::AppAnimeSeries;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SettingsApp {
    library: Vec<AppAnimeSeries>,
}

impl SettingsApp {
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {}
}

use eframe::egui;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LogApp {
    library: Vec<crate::ui::apps::libraryapp::AppAnimeSeries>,
}

impl LogApp {
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {}
}

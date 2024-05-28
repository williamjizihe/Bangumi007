#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)] // hide console window on Windows in release

use crate::ui::mainapp::egui::RichText;
use log4rs::append::Append;

use eframe::egui;
use eframe::egui::ecolor;
// use eframe::egui::WidgetText::RichText;

use crate::module::core::init::run_init;
use crate::ui::panels::libraryapp::LibraryApp;
use crate::ui::panels::logapp::LogApp;
use crate::ui::panels::panel::Panel;
use crate::ui::panels::settingsapp::SettingsApp;

pub fn ui_main() -> Result<(), eframe::Error> {
    run_init().unwrap();
    let ui_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Bangumi007",
        ui_options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(MainApp::new(cc))
        }),
    )
}

fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("sourcehansans_sc".to_owned(),
                           egui::FontData::from_static(include_bytes!("../../../../assets/fonts/SourceHanSans/SC/SourceHanSansSC-Normal.otf")));
    fonts.font_data.insert("sourcehansans_tc".to_owned(),
                           egui::FontData::from_static(include_bytes!("../../../../assets/fonts/SourceHanSans/TC/SourceHanSansTC-Normal.otf")));
    fonts.font_data.insert("sourcehansans_j".to_owned(),
                           egui::FontData::from_static(include_bytes!("../../../../assets/fonts/SourceHanSans/J/SourceHanSans-Normal.otf")));
    fonts.font_data.insert("cascadia_code".to_owned(),
                           egui::FontData::from_static(include_bytes!("../../../../assets/fonts/CascadiaCode.ttf")));
    // fonts.font_data.get_mut("source_han_hans_sc_vf").unwrap().index = 3;
    fonts.families.entry(egui::FontFamily::Proportional).or_default()
        .insert(0, "sourcehansans_sc".to_owned());
    fonts.families.entry(egui::FontFamily::Proportional).or_default()
        .insert(1, "sourcehansans_tc".to_owned());
    fonts.families.entry(egui::FontFamily::Proportional).or_default()
        .insert(2, "sourcehansans_j".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default()
        .insert(0, "cascadia_code".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default()
        .insert(1, "sourcehansans_sc".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default()
        .insert(2, "sourcehansans_tc".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default()
        .insert(3, "sourcehansans_j".to_owned());
    ctx.set_fonts(fonts);
}

// ----------------------------------------------------------------------------
// Main App

impl MainApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_fonts(&cc.egui_ctx);
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(ecolor::Color32::from_rgba_premultiplied(220, 220, 220, 255));
        &cc.egui_ctx.set_visuals(visuals);
        Self::default()
    }
}


impl Default for MainApp {
    fn default() -> Self {
        Self {
            library_app: LibraryApp::default(),
            log_app: LogApp::default(),
            settings_app: SettingsApp::default(),
            open_panel: Panel::default(),
        }
    }
}



// ----------------------------------------------------------------------------

#[derive(PartialEq)]
pub struct MainApp {
    pub library_app: LibraryApp,
    pub log_app: LogApp,
    pub settings_app: SettingsApp,
    pub open_panel: Panel,
}


impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            // Title Bar
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(3.0);
                // ui.heading("Bangumi007");
                ui.selectable_value(&mut self.open_panel, Panel::Library, RichText::new("Bangumi007").size(20.0));
                ui.add_space(5.0);
                ui.selectable_value(&mut self.open_panel, Panel::Log, RichText::new("RSS").size(14.0));
                ui.selectable_value(&mut self.open_panel, Panel::Log, RichText::new("日志").size(14.0));
                ui.selectable_value(&mut self.open_panel, Panel::Settings, RichText::new("设置").size(14.0));
            });
            ui.add_space(2.0);

            ui.separator();

            // Main Panels
            ui.horizontal_centered(|ui| {
                ui.add_space(3.0);
                match self.open_panel {
                    Panel::Library => {
                        self.library_app.ui(ui);
                    }
                    Panel::Log => {
                        self.log_app.ui(ui);
                    }
                    Panel::Settings => {
                        self.settings_app.ui(ui);
                    }
                }
                ui.add_space(3.0);
            });
        });
    }
}
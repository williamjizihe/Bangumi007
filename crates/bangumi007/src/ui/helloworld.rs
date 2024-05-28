#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use eframe::egui::ecolor;
use crate::module::core::init::run_init;
use crate::module::logger;

pub(crate) fn ui_main() -> Result<(), eframe::Error> {
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
            Box::new(MyApp::new(cc))
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

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_fonts(&cc.egui_ctx);
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(ecolor::Color32::from_rgba_premultiplied(220, 220, 220, 255));
        &cc.egui_ctx.set_visuals(visuals);
        Self::default()
    }
}


impl Default for MyApp {
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

#[derive(PartialEq, Eq)]
enum Panel {
    Library,
    Log,
    Settings,
}

impl Default for Panel {
    fn default() -> Self {
        Self::Library
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
struct AppAnimeSeries {
    name: String,
    episodes: Vec<String>,
}

// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
struct LibraryApp {
    library: Vec<AppAnimeSeries>,
}

impl LibraryApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("媒体库");
            ui.add_space(5.0);
            ui.separator();
        });

        ui.horizontal(|ui| {
            ui.label("动画系列");
            ui.add_space(5.0);
            ui.separator();
        });

        for series in &mut self.library {
            ui.horizontal(|ui| {
                ui.label(&series.name);
                ui.add_space(5.0);
                ui.separator();
            });

            for episode in &series.episodes {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(episode));
                    ui.add_space(5.0);
                    ui.separator();
                });
            }
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
struct LogApp {
    library: Vec<AppAnimeSeries>,
}

impl LogApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("日志");
            ui.add_space(5.0);
            ui.separator();
        });

        ui.horizontal(|ui| {
            ui.label("动画系列");
            ui.add_space(5.0);
            ui.separator();
        });

        for series in &mut self.library {
            ui.horizontal(|ui| {
                ui.label(&series.name);
                ui.add_space(5.0);
                ui.separator();
            });

            for episode in &series.episodes {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(episode));
                    ui.add_space(5.0);
                    ui.separator();
                });
            }
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
struct SettingsApp {
    library: Vec<AppAnimeSeries>,
}

impl SettingsApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("设置");
            ui.add_space(5.0);
            ui.separator();
        });

        ui.horizontal(|ui| {
            ui.label("动画系列");
            ui.add_space(5.0);
            ui.separator();
        });

        for series in &mut self.library {
            ui.horizontal(|ui| {
                ui.label(&series.name);
                ui.add_space(5.0);
                ui.separator();
            });

            for episode in &series.episodes {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(episode));
                    ui.add_space(5.0);
                    ui.separator();
                });
            }
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(PartialEq)]
pub struct MyApp {
    library_app: LibraryApp,
    log_app: LogApp,
    settings_app: SettingsApp,
    open_panel: Panel,
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(3.0);
                ui.heading("Bangumi007");
                ui.add_space(5.0);
                ui.selectable_value(&mut self.open_panel, Panel::Library, "媒体库");
                ui.selectable_value(&mut self.open_panel, Panel::Log, "日志");
                ui.selectable_value(&mut self.open_panel, Panel::Settings, "设置");
            });
            ui.add_space(2.0);

            ui.separator();

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
        });
    }
}
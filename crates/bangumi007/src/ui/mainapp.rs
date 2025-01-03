#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)] // hide console window on Windows in release

use std::cell::RefCell;
use std::rc::Rc;
use log4rs::append::Append;

use eframe::egui;
use eframe::egui::{Align, ecolor};

use crate::module::core::init::run_init;
use crate::ui::mainapp::egui::RichText;
use crate::ui::apps::libraryapp::{BANGUMI_STATUS_UPDATE, LibraryApp};
use crate::ui::apps::logapp::LogApp;
use crate::ui::apps::panel::Panel;
use crate::ui::apps::panel::Panel::Library;
use crate::ui::apps::season_conf_dialog_window::SeasonConfDialogWindow;
use crate::ui::apps::settingsapp::SettingsApp;

// use eframe::egui::WidgetText::RichText;

pub fn ui_main() -> Result<(), eframe::Error> {
    run_init().unwrap();
    let ui_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([700.0, 500.0]),
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
        let mut app = Self::default();
        app.library_app.fetch_library();
        app
    }
}


impl Default for MainApp {
    fn default() -> Self {
        Self {
            library_app: LibraryApp::default(),
            log_app: LogApp::default(),
            settings_app: SettingsApp::default(),
            open_panel: Panel::default(),
            season_conf_dialog_window: Rc::new(RefCell::new(SeasonConfDialogWindow::new())),
        }
    }
}


// ----------------------------------------------------------------------------

pub struct MainApp {
    pub open_panel: Panel,
    pub library_app: LibraryApp,
    pub log_app: LogApp,
    pub settings_app: SettingsApp,
    pub season_conf_dialog_window: Rc<RefCell<SeasonConfDialogWindow>>,
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
                ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                    ui.add_space(5.0);
                    ui.horizontal_centered(|ui| {
                        if self.library_app.library.try_read().is_ok() {
                            let mut refresh_rss = ui.button(RichText::new("更新订阅").size(13.0));
                            if refresh_rss.clicked() {
                                self.library_app.update_rss();
                            }
                        }
                        else {
                            ui.label(RichText::new("正在更新订阅").size(13.0));
                        }
                    });
                    ui.add_space(5.0);
                    ui.horizontal_centered(|ui| {
                        if self.library_app.library.try_read().is_err() {
                            ui.spinner();
                        }
                    });
                });
            });
            ui.add_space(2.0);

            ui.separator();

            // Main Panels
            ui.horizontal_centered(|ui| {
                ui.add_space(3.0);
                match self.open_panel {
                    Panel::Library => {
                        self.library_app.ui(ui, self.season_conf_dialog_window.clone());
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
        // egui::Window::new("hello world")
        //     .default_width(200.0)
        //     .default_height(200.0)
        //     .open(&mut true)
        //     .movable(false)
        //     .resizable([false, false])
        //     .show(ctx, |ui| {
        //         // use super::View as _;
        //         ui.label("Hello world!");
        //     });
        let mut season_conf_dialog_window = self.season_conf_dialog_window.borrow_mut();
        if *season_conf_dialog_window.open.borrow() && season_conf_dialog_window.open_my {
            let series = self.library_app.library.clone();
            season_conf_dialog_window.show(ctx, series);
        }

        let flag_handle = BANGUMI_STATUS_UPDATE.read().unwrap();
        if (*flag_handle) {
            drop(flag_handle);

            let library_handle = self.library_app.library.clone();
            LibraryApp::fetch_bangumi_watch_status(library_handle);

        }


    }
}
#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)] // hide console window on Windows in release

use log4rs::append::Append;

use eframe::egui;
use eframe::egui::{ecolor, Vec2};

use crate::module::core::init::run_init;

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
// Panels

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
// Data structure of media library

#[derive(Debug, Clone, Default, PartialEq)]
struct AppAnimeEpisode {
    episode_hash: String,
    disp_episode_num: i32,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct AppAnimeSeason {
    mikan_subject_id: i32,
    mikan_subgroup_id: i32,
    disp_season_name: String,
    disp_season_num: i32,
    disp_thumbnail_url: String,
    episodes: Vec<AppAnimeEpisode>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct AppAnimeSeries {
    disp_series_name: String,
    seasons: Vec<AppAnimeSeason>,
}

// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
struct LibraryApp {
    library: Vec<AppAnimeSeries>,
}

fn series_layout(ui: &mut egui::Ui, series: &AppAnimeSeries) {
    ui.heading("NEW GAME!");
    ui.add_space(3.);
    for season in &series.seasons {
        ui.horizontal(|ui| {
            ui.add(
                egui::Image::new(egui::include_image!("../../../../assets/150775.jpg"))
                    .fit_to_exact_size(Vec2::new(60., 60.))
                    .show_loading_spinner(true)
                    .rounding(5.),
            );
            ui.vertical(|ui| {
                ui.heading(format!("S{:02} - {}", season.disp_season_num,
                    season.disp_season_name).to_string());
                ui.columns(13, |cols| {
                    let mut count_epi = 0;
                    for episode in &season.episodes {
                        // small button with small text (rich text)
                        let button = cols[count_epi].small_button("  ".to_string());
                        count_epi = (count_epi + 1) % 13;
                    }
                });
            });
        });
    }
}

impl LibraryApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        if self.library.is_empty() {
            self.library.push(
                AppAnimeSeries {
                    disp_series_name: "NEW GAME".to_string(),
                    seasons: vec![
                        AppAnimeSeason {
                            mikan_subject_id: 295017,
                            mikan_subgroup_id: 90,
                            disp_season_name: "NEW GAME!".to_string(),
                            disp_season_num: 1,
                            disp_thumbnail_url: "https://lain.bgm.tv/pic/cover/c/0f/79/150775_rRSAT.jpg".to_string(),
                            episodes: vec![
                                AppAnimeEpisode {
                                    episode_hash: "1".to_string(),
                                    disp_episode_num: 1,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "2".to_string(),
                                    disp_episode_num: 2,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "3".to_string(),
                                    disp_episode_num: 3,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "4".to_string(),
                                    disp_episode_num: 4,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "5".to_string(),
                                    disp_episode_num: 5,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "6".to_string(),
                                    disp_episode_num: 6,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "7".to_string(),
                                    disp_episode_num: 7,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "8".to_string(),
                                    disp_episode_num: 8,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "9".to_string(),
                                    disp_episode_num: 9,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "10".to_string(),
                                    disp_episode_num: 10,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "11".to_string(),
                                    disp_episode_num: 11,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "12".to_string(),
                                    disp_episode_num: 12,
                                },
                            ],
                        },
                        AppAnimeSeason {
                            mikan_subject_id: 295017,
                            mikan_subgroup_id: 90,
                            disp_season_name: "NEW GAME!!".to_string(),
                            disp_season_num: 2,
                            disp_thumbnail_url: "https://lain.bgm.tv/pic/cover/c/32/44/208908_AATp0.jpg".to_string(),
                            episodes: vec![
                                AppAnimeEpisode {
                                    episode_hash: "1".to_string(),
                                    disp_episode_num: 1,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "2".to_string(),
                                    disp_episode_num: 2,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "3".to_string(),
                                    disp_episode_num: 3,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "4".to_string(),
                                    disp_episode_num: 4,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "5".to_string(),
                                    disp_episode_num: 5,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "6".to_string(),
                                    disp_episode_num: 6,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "7".to_string(),
                                    disp_episode_num: 7,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "8".to_string(),
                                    disp_episode_num: 8,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "9".to_string(),
                                    disp_episode_num: 9,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "10".to_string(),
                                    disp_episode_num: 10,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "11".to_string(),
                                    disp_episode_num: 11,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "12".to_string(),
                                    disp_episode_num: 12,
                                },
                            ],
                        },
                    ],
                }
            );
            ui.centered_and_justified(|ui| {
                ui.label("媒体库为空");
            });
            return;
        }
        ui.columns(2, |cols| {
            let mut count = 0;
            for series in &mut self.library {
                series_layout(&mut cols[count], series);
                // cols[count].label(&series.disp_series_name);
                // cols[count].add(
                //     egui::Image::new(egui::include_image!("../../../../assets/150775.jpg"))
                //         .fit_to_exact_size(Vec2::new(100., 100.))
                //         .show_loading_spinner(true)
                //         .rounding(5.),
                // );
                // cols[0].label(format!("S{:02} - {}",
                //                       series.seasons[0].disp_season_num,
                //                       series.seasons[0].disp_season_name
                // ));
                // cols[0].columns(12, |cols| {
                //     let mut count_epi = 0;
                //     for episode in &series.seasons[0].episodes {
                //         // small button with small text (rich text)
                //         let button = cols[count_epi].small_button(episode.disp_episode_num.to_string());
                //
                //         count_epi = (count_epi + 1) % 12;
                //     }
                // });
                count = (count + 1) % 2;
            }
        })
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
struct LogApp {
    library: Vec<AppAnimeSeries>,
}

impl LogApp {
    fn ui(&mut self, ui: &mut egui::Ui) {}
}

// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
struct SettingsApp {
    library: Vec<AppAnimeSeries>,
}

impl SettingsApp {
    fn ui(&mut self, ui: &mut egui::Ui) {}
}

// ----------------------------------------------------------------------------

#[derive(PartialEq)]
pub struct MainApp {
    library_app: LibraryApp,
    log_app: LogApp,
    settings_app: SettingsApp,
    open_panel: Panel,
}


impl eframe::App for MainApp {
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
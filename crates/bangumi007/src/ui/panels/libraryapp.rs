// ----------------------------------------------------------------------------

use eframe::egui;
use eframe::egui::{RichText, Vec2, vec2};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LibraryApp {
    library: Vec<AppAnimeSeries>,
}

fn series_layout(ui: &mut egui::Ui, series: &AppAnimeSeries) {
    ui.add_space(3.);
    ui.horizontal_centered(|ui| {
        ui.add_space(3.);
        ui.vertical(|ui| {
            ui.add_space(3.);
            ui.label(RichText::new(series.disp_series_name.clone()).size(16.0));
            for season in &series.seasons {
                ui.add_space(8.);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [51., 68.],
                        egui::Image::new(egui::include_image!("../../../../../assets/150775.jpg"))
                            .show_loading_spinner(true)
                            .rounding(5.),
                    );
                    ui.vertical(|ui| {
                        ui.heading(RichText::new(format!("第 {} 季 - {}", season.disp_season_num,
                                                         season.disp_season_name).to_string()).size(14.0));
                        ui.add_space(3.);
                        ui.horizontal_wrapped(|ui| {
                            ui.style_mut().spacing.item_spacing = vec2(3.0, 3.0);
                            for episode in &season.episodes {
                                // small button with small text (rich text)
                                let button = ui
                                    .add_sized([18., 18.],
                                               egui::Button::new(RichText::new(format!("{:02}", episode.disp_episode_num)).monospace().size(9.0)),
                                    );
                            }
                        });
                    });
                });
            }
            ui.add_space(7.);
            ui.separator();
        });
        ui.add_space(3.);
    });
}

impl LibraryApp {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
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
                                    disp_episode_num: 600,
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
                                    disp_episode_num: 110,
                                },
                                AppAnimeEpisode {
                                    episode_hash: "12".to_string(),
                                    disp_episode_num: 120,
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
        egui::ScrollArea::vertical()
            .max_height(f32::INFINITY)
            .auto_shrink(false)
            .show(ui, |ui| {
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
            });
    }
}

// ----------------------------------------------------------------------------
// Data structure of media library

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppAnimeEpisode {
    pub episode_hash: String,
    pub disp_episode_num: i32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppAnimeSeason {
    pub mikan_subject_id: i32,
    pub mikan_subgroup_id: i32,
    pub disp_season_name: String,
    pub disp_season_num: i32,
    pub disp_thumbnail_url: String,
    pub episodes: Vec<AppAnimeEpisode>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppAnimeSeries {
    pub disp_series_name: String,
    pub seasons: Vec<AppAnimeSeason>,
}


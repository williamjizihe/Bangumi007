// ----------------------------------------------------------------------------

use std::sync::{Arc, RwLock};

use eframe::egui;
use eframe::egui::{RichText, vec2};

use crate::module::database::library::AnimeSeason;

#[derive(Debug, Clone, Default)]
pub struct LibraryApp {
    pub library: Arc<RwLock<Vec<AppAnimeSeries>>>,
}

fn series_layout(ui: &mut egui::Ui, series: &AppAnimeSeries) {
        ui.add_space(3.);
        ui.vertical(|ui| {
            ui.add_space(3.);
            ui.label(RichText::new(series.disp_series_name.clone()).size(16.0));
            for season in &series.seasons {
                ui.add_space(8.);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [51., 68.],
                        egui::Image::new(season.disp_thumbnail_url.clone())
                        // egui::Image::new(egui::include_image!("../../../../../assets/150775.jpg"))
                            .show_loading_spinner(true)
                            .rounding(5.),
                    );
                    ui.vertical(|ui| {
                        let mut disp_season_name = format!("第 {} 季", season.disp_season_num);
                        if disp_season_name != season.disp_season_name {
                            disp_season_name = format!("第 {} 季 - {}", season.disp_season_num,
                                                           season.disp_season_name);
                        }
                        ui.heading(RichText::new(disp_season_name).size(14.0));
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
            // ui.separator();      // Buggy separator
        });
}

impl LibraryApp {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        if self.library.try_read().and_then(|l| Ok(l.is_empty())).unwrap_or(false) {
            // self.library.push(
            //     AppAnimeSeries {
            //         disp_series_name: "NEW GAME".to_string(),
            //         seasons: vec![
            //             AppAnimeSeason {
            //                 mikan_subject_id: 295017,
            //                 mikan_subgroup_id: 90,
            //                 disp_season_name: "NEW GAME!".to_string(),
            //                 disp_season_num: 1,
            //                 disp_thumbnail_url: "https://lain.bgm.tv/pic/cover/c/0f/79/150775_rRSAT.jpg".to_string(),
            //                 episodes: vec![
            //                     AppAnimeEpisode {
            //                         episode_hash: "1".to_string(),
            //                         disp_episode_num: 1,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "2".to_string(),
            //                         disp_episode_num: 2,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "3".to_string(),
            //                         disp_episode_num: 3,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "4".to_string(),
            //                         disp_episode_num: 4,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "5".to_string(),
            //                         disp_episode_num: 5,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "6".to_string(),
            //                         disp_episode_num: 6,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "7".to_string(),
            //                         disp_episode_num: 7,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "8".to_string(),
            //                         disp_episode_num: 8,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "9".to_string(),
            //                         disp_episode_num: 9,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "10".to_string(),
            //                         disp_episode_num: 10,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "11".to_string(),
            //                         disp_episode_num: 11,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "12".to_string(),
            //                         disp_episode_num: 12,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "1".to_string(),
            //                         disp_episode_num: 1,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "2".to_string(),
            //                         disp_episode_num: 2,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "3".to_string(),
            //                         disp_episode_num: 3,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "4".to_string(),
            //                         disp_episode_num: 4,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "5".to_string(),
            //                         disp_episode_num: 5,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "6".to_string(),
            //                         disp_episode_num: 600,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "7".to_string(),
            //                         disp_episode_num: 7,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "8".to_string(),
            //                         disp_episode_num: 8,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "9".to_string(),
            //                         disp_episode_num: 9,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "10".to_string(),
            //                         disp_episode_num: 10,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "11".to_string(),
            //                         disp_episode_num: 110,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "12".to_string(),
            //                         disp_episode_num: 120,
            //                     },
            //                 ],
            //             },
            //             AppAnimeSeason {
            //                 mikan_subject_id: 295017,
            //                 mikan_subgroup_id: 90,
            //                 disp_season_name: "NEW GAME!!".to_string(),
            //                 disp_season_num: 2,
            //                 disp_thumbnail_url: "https://lain.bgm.tv/pic/cover/c/32/44/208908_AATp0.jpg".to_string(),
            //                 episodes: vec![
            //                     AppAnimeEpisode {
            //                         episode_hash: "1".to_string(),
            //                         disp_episode_num: 1,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "2".to_string(),
            //                         disp_episode_num: 2,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "3".to_string(),
            //                         disp_episode_num: 3,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "4".to_string(),
            //                         disp_episode_num: 4,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "5".to_string(),
            //                         disp_episode_num: 5,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "6".to_string(),
            //                         disp_episode_num: 6,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "7".to_string(),
            //                         disp_episode_num: 7,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "8".to_string(),
            //                         disp_episode_num: 8,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "9".to_string(),
            //                         disp_episode_num: 9,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "10".to_string(),
            //                         disp_episode_num: 10,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "11".to_string(),
            //                         disp_episode_num: 11,
            //                     },
            //                     AppAnimeEpisode {
            //                         episode_hash: "12".to_string(),
            //                         disp_episode_num: 12,
            //                     },
            //                 ],
            //             },
            //         ],
            //     }
            // );
            ui.centered_and_justified(|ui| {
                ui.label("媒体库无内容");
            });
            return;
        }
        if self.library.try_write().is_err() {
            ui.centered_and_justified(|ui| {
                ui.label("正在更新订阅源...");
            });
            return;
        }
        egui::ScrollArea::vertical()
            .max_height(f32::INFINITY)
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.columns(2, |cols| {
                    let library = self.library.try_read().unwrap();
                    // let mut count = 0;
                    // for series in &*library {
                    //     series_layout(&mut cols[count], series);
                    //     count = (count + 1) % 2;
                    // }
                    cols[0].horizontal_centered(|ui| {
                        ui.add_space(3.);
                        // For the first half of the library
                        ui.vertical(|ui| {
                            for series in &library[..(library.len() as f32 / 2.).ceil() as usize] {
                                series_layout(ui, series);
                            }
                        });
                        ui.add_space(2.);
                    });
                    cols[1].horizontal_centered(|ui| {
                        ui.add_space(2.);
                        // For the first half of the library
                        ui.vertical(|ui| {
                            for series in &library[(library.len() as f32 / 2.).ceil() as usize..] {
                                series_layout(ui, series);
                            }
                        });
                        ui.add_space(3.);
                    });
                });
            })
        ;
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

// AnimeSeason -> AppAnimeSeason
impl From<AnimeSeason> for AppAnimeSeason {
    fn from(season: AnimeSeason) -> Self {
        Self {
            mikan_subject_id: season.mikan_subject_id,
            mikan_subgroup_id: season.mikan_subgroup_id,
            disp_season_name: season.disp_season_name,
            disp_season_num: season.disp_season_num,
            disp_thumbnail_url: season.mikan_subject_image,
            episodes: vec![],
        }
    }
}

// AnimeSeasonItem -> AppAnimeEpisode
impl From<crate::module::database::library::AnimeSeasonItem> for AppAnimeEpisode {
    fn from(episode: crate::module::database::library::AnimeSeasonItem) -> Self {
        Self {
            episode_hash: episode.mikan_item_uuid,
            disp_episode_num: episode.disp_episode_num,
        }
    }
}
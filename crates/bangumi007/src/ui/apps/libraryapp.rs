// ----------------------------------------------------------------------------

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use lazy_static::lazy_static;

use eframe::egui;
use eframe::egui::{RichText, vec2};
use eframe::egui::CursorIcon::PointingHand;

use crate::module::database::library::AnimeSeason;
use crate::ui::apps::season_conf_dialog_window::SeasonConfDialogWindow;
use crate::module::scrobbler::bangumi::{BangumiEpisodeStatus, BangumiEpisodeType};

#[derive(Debug, Clone, Default)]
pub struct LibraryApp {
    pub library: Arc<RwLock<Vec<AppAnimeSeries>>>,
}

lazy_static!(
    // flag for bangumi status update
    pub static ref BANGUMI_STATUS_UPDATE: Arc<RwLock<bool>> = Arc::new(RwLock::new(false));
);

impl LibraryApp {
    fn series_layout(&mut self, ui: &mut egui::Ui, series: &AppAnimeSeries, season_conf_dialog_window: Rc<RefCell<SeasonConfDialogWindow>>) {
        ui.add_space(3.);
        ui.vertical(|ui| {
            ui.add_space(3.);
            let title = ui.label(RichText::new(series.disp_series_name.clone()).size(16.0));
            for season in &series.seasons {
                ui.add_space(8.);
                ui.horizontal(|ui| {
                    let season_image = ui.add_sized(
                        [51., 68.],
                        egui::Image::new(season.disp_thumbnail_url.clone())
                            // egui::Image::new(egui::include_image!("../../../../../assets/150775.jpg"))
                            .show_loading_spinner(true)
                            .rounding(5.),
                    );
                    // if season_image.clicked() {
                    //     let mut season_conf_dialog_window = season_conf_dialog_window.borrow_mut();
                    //     season_conf_dialog_window.subject_id = season.mikan_subject_id;
                    //     season_conf_dialog_window.subgroup_id = season.mikan_subgroup_id;
                    //     season_conf_dialog_window.open = true;
                    // }
                    ui.vertical(|ui| {
                        let mut disp_season_name = if season.conf_season_num != -1 {
                            format!("* 第 {} 季", season.conf_season_num)
                        } else {
                            let mut disp_season_name = format!("第 {} 季", season.disp_season_num);
                            if disp_season_name != season.disp_season_name {
                                disp_season_name = format!("第 {} 季 - {}", season.disp_season_num,
                                                           season.disp_season_name);
                            }
                            if season.conf_tmdb_episode_offset != 0 {
                                disp_season_name = format!("* {}", disp_season_name);
                            }
                            disp_season_name
                        };
                        let season_title = ui.heading(RichText::new(disp_season_name).size(14.0)).on_hover_cursor(PointingHand);
                        if season_title.clicked() {
                            let mut season_conf_dialog_window = season_conf_dialog_window.borrow_mut();
                            season_conf_dialog_window.subject_id = season.mikan_subject_id;
                            season_conf_dialog_window.subgroup_id = season.mikan_subgroup_id;
                            *season_conf_dialog_window.open.borrow_mut() = true;
                            season_conf_dialog_window.open_my = true;
                            season_conf_dialog_window.inited = false;
                        }
                        ui.add_space(3.);
                        ui.horizontal_wrapped(|ui| {
                            ui.style_mut().spacing.item_spacing = vec2(3.0, 3.0);
                            for episode in &season.episodes {
                                // small button with small text (rich text)
                                let button = ui
                                    .add_sized([18., 18.],
                                               egui::Button::new(RichText::new(format!("{:02}", episode.disp_episode_num)).monospace().size(9.0).color(episode.bangumi_status.get_text_color(episode.bangumi_airdate.clone()))).fill(episode.bangumi_status.get_fill_color(episode.bangumi_airdate.clone())),
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

    pub fn ui(&mut self, ui: &mut egui::Ui, season_conf_dialog_window: Rc<RefCell<SeasonConfDialogWindow>>) {
        let library = self.library.clone();
        let library = library.try_read();
        if library.is_err() {
            ui.centered_and_justified(|ui| {
                ui.label("正在更新订阅源...");
            });
            return;
        }
        let library = library.unwrap();
        if library.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("媒体库无内容");
            });
            return;
        }

        egui::ScrollArea::vertical()
            .max_height(f32::INFINITY)
            .auto_shrink(false)
            .show(ui, |ui| {
                let max_width = ui.available_width();
                let columns = (max_width / 280.).floor() as usize;
                let columns = if columns == 0 { 1 } else { columns };
                ui.columns(columns, |cols| {
                    for col_index in 0..columns {
                        cols[col_index].horizontal_centered(|ui| {
                            if col_index == 0 { ui.add_space(1.); }
                            ui.add_space(2.);
                            // For the first half of the library
                            ui.vertical(|ui| {
                                for series in &library[(col_index as f32 * library.len() as f32 / columns as f32).ceil() as usize..((col_index + 1) as f32 * library.len() as f32 / columns as f32).ceil() as usize] {
                                    self.series_layout(ui, series, season_conf_dialog_window.clone());
                                }
                            });
                            ui.add_space(2.);
                            if col_index == columns - 1 { ui.add_space(1.); }
                        });
                    }
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
    pub bangumi_sort: String,
    pub bangumi_airdate: String,
    pub bangumi_name: String,
    pub bangumi_name_cn: String,
    pub bangumi_ep_type: BangumiEpisodeType,
    pub bangumi_status: BangumiEpisodeStatus,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppAnimeSeason {
    pub mikan_subject_id: i32,
    pub mikan_subgroup_id: i32,
    pub bangumi_subject_id: i32,
    pub disp_season_name: String,
    pub disp_season_num: i32,
    pub disp_thumbnail_url: String,
    pub default_season_num: i32,
    pub conf_season_num: i32,
    pub conf_tmdb_episode_offset: i32,
    pub conf_bangumi_episode_offset: i32,
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
            bangumi_subject_id: season.bangumi_subject_id,
            disp_season_name: season.disp_season_name,
            disp_season_num: season.disp_season_num,
            disp_thumbnail_url: season.mikan_subject_image,
            default_season_num: if season.tmdb_season_num != -1 {
                season.tmdb_season_num
            } else {
                season.bangumi_season_num
            },
            episodes: vec![],
            conf_tmdb_episode_offset: season.conf_tmdb_episode_offset,
            conf_bangumi_episode_offset: season.conf_bangumi_episode_offset,
            conf_season_num: season.conf_season_num,
        }
    }
}

// AnimeSeasonItem -> AppAnimeEpisode
impl From<crate::module::database::library::AnimeSeasonItem> for AppAnimeEpisode {
    fn from(episode: crate::module::database::library::AnimeSeasonItem) -> Self {
        Self {
            episode_hash: episode.mikan_item_uuid,
            disp_episode_num: episode.disp_episode_num,
            bangumi_sort: "".to_string(),
            bangumi_airdate: "".to_string(),
            bangumi_name: "".to_string(),
            bangumi_name_cn: "".to_string(),
            bangumi_ep_type: BangumiEpisodeType::MainStory,
            bangumi_status: BangumiEpisodeStatus::NotCollected,
        }
    }
}
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use eframe::egui;
use eframe::egui::{Align, RichText};

use crate::ui::apps::libraryapp::AppAnimeSeries;
use crate::ui::binding::season_conf::{SeasonConf, update_conf};

#[derive(Debug, Clone, Default)]
pub struct SeasonConfDialogWindow {
    pub open: Rc<RefCell<bool>>,
    pub open_my: bool,
    pub inited: bool,
    pub subject_id: i32,
    pub subgroup_id: i32,
    pub default_disp_season: i32,
    pub conf_season: i32,
    pub conf_season_changed: bool,
    pub ep_num_min: i32,
    pub ep_num_max: i32,
    pub conf_ep_offset: i32,
}

impl SeasonConfDialogWindow {
    pub fn new() -> Self {
        Self {
            open: Rc::new(RefCell::new(false)),
            open_my: false,
            inited: false,
            subject_id: -1,
            subgroup_id: -1,
            default_disp_season: -1,
            conf_season: -1,
            conf_season_changed: false,
            ep_num_min: -1,
            ep_num_max: -1,
            conf_ep_offset: 0,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, library: Arc<RwLock<Vec<AppAnimeSeries>>>) {
        if !(*self.open.borrow()) || !self.open_my {
            return;
        }
        if !self.inited {
            let library = library.read().unwrap();
            'outer: for series in library.iter() {
                for season in series.seasons.iter() {
                    if season.mikan_subject_id == self.subject_id && season.mikan_subgroup_id == self.subgroup_id {
                        self.default_disp_season = season.default_season_num;
                        self.conf_season = if season.conf_season_num != -1 {
                            self.conf_season_changed = true;
                            season.conf_season_num
                        } else {
                            self.conf_season_changed = false;
                            season.disp_season_num
                        };

                        self.conf_ep_offset = season.conf_episode_offset;
                        self.ep_num_min = season.episodes.iter().map(|e| e.disp_episode_num - self.conf_ep_offset).min().unwrap();
                        self.ep_num_max = season.episodes.iter().map(|e| e.disp_episode_num - self.conf_ep_offset).max().unwrap();
                        break 'outer;
                    }
                }
            }
        }
        let mut window = if self.inited {
            egui::Window::new(RichText::new("编辑季度信息").size(17.))
                .resizable(false)
                .title_bar(true)
        } else {
            egui::Window::new(RichText::new("编辑季度信息").size(17.))
                .resizable(false)
                .title_bar(true)
                .current_pos([ctx.available_rect().center().x - 120., ctx.available_rect().center().y - 100.])
        }
            .default_width(240.)
            .default_height(200.)
            .open(&mut *self.open.borrow_mut())
            .show(ctx, |ui| {
                egui::Grid::new("season_conf_dialog")
                    .num_columns(2)
                    .min_col_width(120.)
                    .min_row_height(25.)
                    // .spacing([60., 25.])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("季度：");
                        ui.horizontal_centered( |ui| {
                            let drag = ui.add(
                                egui::DragValue::new(&mut self.conf_season)
                                    .speed(0.1)
                                    .clamp_range(0..=99)
                            );
                            if drag.is_pointer_button_down_on() {
                                self.conf_season_changed = true;
                            }
                            ui.add_enabled_ui(
                                self.conf_season_changed,
                                |ui| {
                                    let button = ui.button("x").on_hover_text("重置为默认季度");
                                    if button.clicked() {
                                        self.conf_season = self.default_disp_season;
                                        self.conf_season_changed = false;
                                    }
                                },
                            );
                        });
                        ui.end_row();
                        ui.label("原始剧集范围：");
                        ui.label(format!("{} - {}", self.ep_num_min, self.ep_num_max));
                        ui.end_row();
                        ui.label("剧集偏移：");
                        ui.horizontal_centered(|ui| {
                            let drag = ui.add(
                                egui::DragValue::new(&mut self.conf_ep_offset)
                                    .speed(0.3)
                                    .clamp_range(-self.ep_num_min + 1..=999)
                            );
                            ui.add_enabled_ui(
                                self.conf_ep_offset != 0,
                                |ui| {
                                    let button = ui.button("x").on_hover_text("重置剧集偏移");
                                    if button.clicked() {
                                        self.conf_ep_offset = 0;
                                    }
                                },
                            );
                        });
                        ui.end_row();
                        ui.label("新剧集范围：");
                        // if self.conf_ep_offset != 0 {
                            ui.label(format!("{} - {}", self.ep_num_min + self.conf_ep_offset, self.ep_num_max + self.conf_ep_offset));
                        // } else {
                        //     ui.label("(未更改)");
                        // }
                        ui.end_row();
                    },
                    );
                ui.add_space(8.);
                ui.columns(2, |cols| {
                    cols[0].vertical_centered(|ui| {
                        let button = ui.button("取消").on_hover_text("取消修改");
                        if button.clicked() {
                            // *self.open.borrow_mut() = false;
                            self.open_my = false;
                            self.inited = false;
                            self.subject_id = -1;
                            self.subgroup_id = -1;
                        }
                    });
                    cols[1].vertical_centered(|ui| {
                        let btn_apply = ui.button("应用").on_hover_text("应用修改");
                        if btn_apply.clicked() {
                            update_conf(SeasonConf {
                                subject_id: self.subject_id,
                                subgroup_id: self.subgroup_id,
                                default_disp_season: self.default_disp_season,
                                conf_season: self.conf_season,
                                conf_season_changed: self.conf_season_changed,
                                ep_num_min: self.ep_num_min,
                                ep_num_max: self.ep_num_max,
                                conf_ep_offset: self.conf_ep_offset,
                            },
                                        library.clone());
                            self.open_my = false;
                            self.inited = false;
                            self.subgroup_id = -1;
                            self.subject_id = -1;
                        }
                    });
                });
            });
        self.inited = true;
    }
}
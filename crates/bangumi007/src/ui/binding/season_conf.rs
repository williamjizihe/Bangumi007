use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use crate::module::database::library::{AnimeSeason, read_all_items, read_season_items, read_seasons, set_season_bangumi_episode_offset, set_season_conf_season_num, set_season_disp_season_num, set_season_tmdb_episode_offset};
use crate::module::downloader::qbittorrent::{clean_empty_folders, download_items, rename_torrents_files};
use crate::module::library::{auto_season_config_clean, update_library};
use crate::module::parser::mikan_parser::{expand_history_episodes, update_rss};
use crate::ui::apps::libraryapp::{AppAnimeSeason, AppAnimeSeries, LibraryApp};
use crate::ui::apps::season_conf_dialog_window::SeasonConfDialogWindow;

#[derive(Debug, Clone, Default)]
pub struct SeasonConf {
    pub subject_id: i32,
    pub subgroup_id: i32,
    pub default_disp_season: i32,
    pub conf_season: i32,
    pub conf_season_changed: bool,
    pub ep_num_min: i32,
    pub ep_num_max: i32,
    pub conf_tmdb_ep_offset: i32,
    pub conf_bangumi_ep_offset: i32,
}

pub fn update_conf(conf: SeasonConf, library: Arc<RwLock<Vec<AppAnimeSeries>>>) {

    let library_handle = library.clone();

    log::info!("Update season conf");

    let library = library_handle.clone();

    let handle = thread::spawn(move || {

        let library = library.write();

        if let Err(e) = library {
            log::error!("Library lock poisoned: {:?}", e);
            return;
        }
        log::debug!("Library locked successfully.");

        let mut library = library.unwrap();

        log::info!("Start updating season conf");

        if conf.conf_season_changed {
            set_season_conf_season_num(conf.subject_id, conf.subgroup_id, conf.conf_season);
        }
        else {
            set_season_conf_season_num(conf.subject_id, conf.subgroup_id, -1);
        }
        set_season_disp_season_num(conf.subject_id, conf.subgroup_id, conf.conf_season);

        set_season_tmdb_episode_offset(conf.subject_id, conf.subgroup_id, conf.conf_tmdb_ep_offset);
        set_season_bangumi_episode_offset(conf.subject_id, conf.subgroup_id, conf.conf_bangumi_ep_offset);

        *library = Vec::new();
        // Output media library
        let seasons = read_seasons();
        // Season arrange by series name
        let serieses: HashMap<String, Vec<AnimeSeason>> = seasons.into_iter().fold(HashMap::new(), |mut acc, season| {
            let series_name = season.disp_series_name.clone();
            if acc.contains_key(&series_name) {
                acc.get_mut(&series_name).unwrap().push(season);
            } else {
                acc.insert(series_name, vec![season]);
            }
            acc
        });
        for (series, seasons) in serieses {
            let mut series = AppAnimeSeries {
                disp_series_name: series,
                seasons: Vec::new(),
            };
            for season in seasons {
                let season_episodes = read_season_items(season.mikan_subject_id, season.mikan_subgroup_id);
                let mut app_anime_season: AppAnimeSeason = season.into();
                for episode in season_episodes {
                    app_anime_season.episodes.push(episode.into());
                }
                // sort episodes by disp_episode_num, ascending
                app_anime_season.episodes.sort_by(|a, b| a.disp_episode_num.cmp(&b.disp_episode_num));
                series.seasons.push(app_anime_season);
            }
            // sort seasons, ascending
            series.seasons.sort_by(|a, b| a.disp_season_num.cmp(&b.disp_season_num));
            (*library).push(series);
        }
        library.sort_by(|a, b| a.disp_series_name.cmp(&b.disp_series_name));

        // Add torrents to downloader
        let library_items = read_season_items(conf.subject_id, conf.subgroup_id);
        download_items(&library_items, true).unwrap();
        rename_torrents_files(&library_items).unwrap();
        clean_empty_folders("".to_string());

        log::info!("RSS updated successfully.");
        drop(library);

        if LibraryApp::fetch_bangumi_watch_status(library_handle) { return; }

    });

}


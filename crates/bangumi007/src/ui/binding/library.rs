use std::collections::HashMap;
use std::thread;
use crate::module::database::library::{AnimeSeason, read_all_items, read_season_items, read_seasons};
use crate::module::downloader::qbittorrent::{download_items, rename_torrents_files};
use crate::module::library::{auto_season_config_clean, update_library};
use crate::module::parser::mikan_parser::{expand_history_episodes, update_rss};
use crate::ui::panels::libraryapp::{AppAnimeSeason, AppAnimeSeries, LibraryApp};

impl LibraryApp {
    pub fn update_rss(&mut self) {

        let library = self.library.clone();

        let handle = thread::spawn(move || {

            let library = library.try_write();

            if let Err(_) = library {
                return;
            }

            let mut library = library.unwrap();

            log::info!("Start updating rss");

            // Fetch RSS feeds
            let rss_list = crate::module::config::CONFIG.read().unwrap().rss_config.list.clone();
            for rss in rss_list {
                if rss.active {
                    let items = update_rss(&*rss.url).unwrap();
                    // By default, only incremental, not expanding the history
                    let items = expand_history_episodes(items);
                    update_library(&items);
                }
            }

            // Rearrange the media library
            auto_season_config_clean();

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

            // for season in read_seasons() {
            //     println!("Season: {:?}", season.mikan_subject_name);
            //     print!("Ep: ");
            //     let season_items = read_season_items(season.mikan_subject_id, season.mikan_subgroup_id);
            //     for item in season_items {
            //         print!("{:?} ", item.disp_episode_num);
            //     }
            //     println!();
            // }

            // Add torrents to downloader
            let library_items = read_all_items();
            download_items(&library_items, true).unwrap();
            rename_torrents_files(&library_items).unwrap();

            drop(library);
        });
    }

    pub fn update_library(&mut self) {

        let library = self.library.clone();

        let handle = thread::spawn(move || {

            let library = library.try_write();

            if let Err(_) = library {
                return;
            }

            let mut library = library.unwrap();

            log::info!("Start updating library");

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

            drop(library);
        });
    }
}
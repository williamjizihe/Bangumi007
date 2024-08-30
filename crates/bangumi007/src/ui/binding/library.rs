use std::collections::HashMap;
use std::sync::{Arc, mpsc, RwLock};
use std::thread;
use std::time::Duration;
use rand::Rng;
use crate::module::database::library::{AnimeSeason, read_all_items, read_season_items, read_seasons};
use crate::module::downloader::qbittorrent::{clean_empty_folders, download_items, rename_torrents_files};
use crate::module::library::{auto_season_config_clean, update_library};
use crate::module::parser::mikan_parser::{expand_history_episodes, update_rss};
use crate::module::scrobbler::bangumi::get_bangumi_episode_collection_status;
use crate::ui::apps::libraryapp::{AppAnimeEpisode, AppAnimeSeason, AppAnimeSeries, BANGUMI_STATUS_UPDATE, LibraryApp};

impl LibraryApp {
    pub fn update_rss(&mut self) {

        log::info!("Update rss pressed");

        let library = self.library.clone();

        let handle = thread::spawn(move || {

            let library_handle = library.clone();

            let library = library_handle.write();
            if let Err(e) = library {
                log::error!("Library lock poisoned: {:?}", e);
                return;
            }
            log::debug!("Library locked successfully.");
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
                for season in &seasons {
                    let season_episodes = read_season_items(season.mikan_subject_id, season.mikan_subgroup_id);
                    let mut app_anime_season: AppAnimeSeason = <AnimeSeason as Clone>::clone(&(*season)).into();
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
            clean_empty_folders("".to_string());

            log::info!("RSS updated successfully.");
            drop(library);

            if Self::fetch_bangumi_watch_status(library_handle) { return; }

        });
    }

    pub fn fetch_library(&mut self) {

        let library = self.library.clone();

        let handle = thread::spawn(move || {

            let library_handle = library.clone();

            let library = library_handle.write();

            if let Err(e) = library {
                log::debug!("Library lock poisoned: {:?}", e);
                return;
            }
            log::debug!("Library locked successfully.");

            let mut library = library.unwrap();

            log::info!("Start updating library");

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

            log::debug!("Library updated successfully.");
            drop(library);

            if Self::fetch_bangumi_watch_status(library_handle) { return; }
        });
    }

    pub fn fetch_bangumi_watch_status(library_handle: Arc<RwLock<Vec<AppAnimeSeries>>>) -> bool {
        // self.fetch_bangumi_watch_status();

        let library = library_handle.read();
        if let Err(e) = library {
            log::debug!("Library lock poisoned: {:?}", e);
            return true;
        }
        log::debug!("Library locked successfully.");
        let mut library = library.unwrap();

        let subject_ids: Vec<i32> = library.iter().map(|series| {
            series.seasons.iter().map(|season| {
                season.bangumi_subject_id
            }).collect::<Vec<i32>>()
        }).flatten().collect();

        drop(library);

        // Spawn a thread for each subject ID,
        // getting Episode status using get_bangumi_episode_collection_status
        // result passed by mpsc channel, for each result, once received, update the library immedialy (TODO)
        // and fill the status field of each episode
        let (tx, rx) = mpsc::channel();
        let handles: Vec<_> = subject_ids.into_iter().map(|subject_id| {
            let tx = tx.clone();
            thread::spawn(move || {
                // delay random time from 0~1000
                thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(0..1000)));
                let status = retry::retry(retry::delay::Fixed::from_millis(1000).take(5), || {
                    get_bangumi_episode_collection_status(subject_id)
                }).unwrap();
                tx.send((subject_id, status)).unwrap();
            })
        }).collect();

        let mut succ_count = 0;
        for handle in handles {
            let res = handle.join();
            if res.is_ok() {
                succ_count += 1;
            }
        }

        let library = library_handle.write();
        if let Err(e) = library {
            log::debug!("Library lock poisoned: {:?}", e);
            return true;
        }
        log::debug!("Library locked successfully.");
        let mut library = library.unwrap();
        log::debug!("Library get.");

        if library.is_empty() {
            log::debug!("Library is empty.");
            drop(library);
            return false;
        }

        let mut count = 0;
        for (subject_id, status) in rx.iter() {
            log::debug!("Subject ID: {}", subject_id);
            // Update library for a SUBJECT
            for series in library.iter_mut() {
                for season in series.seasons.iter_mut() {
                    if season.bangumi_subject_id == subject_id {
                        // Match a SEASON in the labrary with subject_id
                        for episode in season.episodes.iter_mut() {
                            // Match an EPISODE's sort in the SEASON with disp_episode_num - season's tmdb_episode_offset + bangumi_episode_offset
                            let episode_sort = (episode.disp_episode_num - season.conf_tmdb_episode_offset + season.conf_bangumi_episode_offset).to_string();
                            for s in status.iter() {
                                if s.sort == episode_sort {
                                    episode.bangumi_sort = s.sort.clone();
                                    episode.bangumi_airdate = s.airdate.clone();
                                    episode.bangumi_name = s.name.clone();
                                    episode.bangumi_name_cn = s.name_cn.clone();
                                    episode.bangumi_ep_type = s.ep_type.clone();
                                    episode.bangumi_status = s.status.clone();
                                }
                            }
                        }
                    }
                }
            }
            count += 1;
            if count == succ_count {
                break;
            }
        }

        log::debug!("Bangumi status fetched successfully.");
        drop(library);

        let mut flag_handle = BANGUMI_STATUS_UPDATE.write().unwrap();
        *flag_handle = false;
        drop(flag_handle);

        false
    }
}
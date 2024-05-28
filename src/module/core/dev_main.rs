use crate::module::core::init::run_init;
use crate::module::database::library::{read_all_items, read_season_items, read_seasons};
use crate::module::downloader::qbittorrent::{download_items, rename_torrents_files};
use crate::module::library::{auto_season_config_clean, update_library};
use crate::module::parser::mikan_parser::{expand_history_episodes, update_rss};

pub fn run() {
    run_init().unwrap();

    log::info!("Program started");
    // Fetch RSS feeds
    let rss_list = crate::module::config::CONFIG.read().unwrap().rss_config.list.clone();
    for rss in rss_list {
        if rss.active {
            let items = update_rss(&*rss.url).unwrap();
            // By default, only incremental, not expanding the history
            // let items = expand_history_episodes(items);
            update_library(&items);
        }
    }
    // Rearrange the media library
    auto_season_config_clean();
    // Output media library
    for season in read_seasons() {
        println!("Season: {:?}", season.mikan_subject_name);
        print!("Ep: ");
        let season_items = read_season_items(season.mikan_subject_id, season.mikan_subgroup_id);
        for item in season_items {
            print!("{:?} ", item.disp_episode_num);
        }
        println!();
    }
    // Add torrents to downloader
    let library_items = read_all_items();
    download_items(&library_items, true).unwrap();
    rename_torrents_files(&library_items).unwrap();
}

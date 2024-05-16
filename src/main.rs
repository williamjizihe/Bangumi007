use crate::utils::rss_parser::{parse_rss};
use crate::utils::qbittorrent::{download_items};
use crate::utils::media_library::{auto_season_config_clean, read_all_items, read_season_items, read_seasons, update_library};

mod config;
mod utils;

fn main() {
    utils::logger::init();
    log::info!("Program started");
    // Fetch RSS feeds
    let rss_list = config::CONFIG.read().unwrap().rss_config.list.clone();
    for rss in rss_list {
        if rss.active {
            let items = parse_rss(&*rss.url).unwrap();
            // By default, only incremental, not expanding the history
            // let items = expand_rss(items);       
            update_library(&items);
        }
    }
    // Rearrange the media library
    auto_season_config_clean();
    // Output media library
    for season in read_seasons() {
        println!("Season: {:?}", season.mikan_bangumi_title);
        print!("Ep: ");
        let season_items = read_season_items(season.mikan_bangumi_id, season.mikan_subgroup_id);
        for item in season_items {
            print!("{:?} ", item.episode_num_offseted);
        }
        println!();
    }
    // Add torrents to downloader
    let library_items = read_all_items();
    download_items(&library_items).unwrap();
}

use crate::utils::rss_parser::{parse_rss, expand_rss};
use crate::utils::media_library::{auto_season_config_clean, read_season_items, read_seasons, update_library};

mod config;
mod utils;

fn main() {
    utils::logger::init();
    log::info!("Program started");
    let rss_list = config::CONFIG.read().unwrap().rss_config.list.clone();
    for rss in rss_list {
        if rss.active {
            let items = parse_rss(&*rss.url).unwrap();
            // By default, only incremental, not expanding the history
            // let items = expand_rss(items);       
            update_library(&items);
        }
    }
    auto_season_config_clean();
    for season in read_seasons() {
        println!("Season: {:?}", season.mikan_bangumi_title);
        print!("Ep: ");
        let season_items = read_season_items(season.mikan_bangumi_id, season.mikan_subgroup_id);
        for item in season_items {
            print!("{:?} ", item.episode_num_offseted);
        }
        println!();
    }
}

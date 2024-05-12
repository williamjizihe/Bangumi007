use std::process::exit;
use log::log;
use crate::utils::rss_cache::init_database;
use crate::utils::rss_parser::{parse_rss, expand_rss};
use crate::utils::media_library::{AnimeSeason, AnimeSeasonItem, auto_season_config_clean, read_season_items, read_seasons, update_library};
use crate::utils::{media_library, rss_cache};

mod config;
mod utils;

fn main() {
    utils::logger::init_logging();
    log::debug!("Program started");
    let conn = rss_cache::init_database().unwrap();
    let url = "https://mikanani.me/RSS/MyBangumi?token=";      // TODO: token config secret
    let items = parse_rss(url, &conn).unwrap();
    let items = expand_rss(items, &conn);
    let conn = media_library::init_database().unwrap();
    update_library(&items, &conn);
    auto_season_config_clean(&conn);
    for season in read_seasons(&conn) {
        println!("Season: {:?}", season.mikan_bangumi_title);
        print!("Ep: ");
        let season_items = read_season_items(season.mikan_bangumi_id, season.mikan_subgroup_id, &conn);
        for item in season_items {
            print!("{:?} ", item.episode_num_offseted);
        }
        println!();
    }

}

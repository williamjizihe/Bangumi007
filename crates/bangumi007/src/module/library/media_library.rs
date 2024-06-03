use std::collections::HashSet;

use crate::module::database::cache::rss;
use crate::module::database::cache::rss::MikanItem;
use crate::module::database::get_connection;
use crate::module::database::library::{AnimeSeason, create_item, create_season, delete_item, read_season_info, read_season_items, read_seasons};
use crate::module::parser::mikan_parser;

pub fn update_library(items: &Vec<rss::MikanItem>) {
    // For each item in the fetched updating list,
    // Match the item with the corresponding anime season
    // If the season is not found, insert the season into the database

    for item in items {
        // season in library
        if let Some(season) = read_season_info(item.mikan_subject_id, item.mikan_subgroup_id) {
            // If the season is found, insert the item into the database if the item obeys the language and codec restriction
            // TODO: RSS parser parse only the language and codec configured
            if season.conf_language != "" && season.conf_language != item.mikan_parsed_language {
                continue;
            }
            if season.conf_codec != "" && season.conf_codec != item.mikan_parsed_codec {
                continue;
            }
            create_item(&item); // episode offset logic inside.
        } else {
            // season in rss cache
            let season_cache = rss::fetch_mikan_subject_info(item.mikan_subject_id);
            match season_cache {
                Some(season) => {
                    let disp_series_name = if season.tmdb_series_name == "" {
                        season.bangumi_subject_name.clone()
                    } else {
                        season.tmdb_series_name.clone()
                    };
                    let disp_season_num = if season.tmdb_season_num == -1 {
                        season.bangumi_season_num
                    } else {
                        season.tmdb_season_num
                    };
                    // TODO: Check invalid tmdb season name logic
                    let disp_season_name = if season.tmdb_season_name == "" {
                        format!("第 {} 季", disp_season_num)
                    } else {
                        season.tmdb_season_name.clone()
                    };
                    // TODO: fetch subgroup name
                    let disp_subgroup_name = "字幕组名称".to_string();
                    // TODO: Parse episode offset between subgroup epinum and tmdb epinum

                    let season = AnimeSeason {
                        mikan_subject_id: item.mikan_subject_id,
                        mikan_subgroup_id: item.mikan_subgroup_id,
                        mikan_subject_name: season.bangumi_subject_name.clone(),
                        mikan_subject_image: season.mikan_subject_image_url,
                        bangumi_subject_id: season.bangumi_subject_id,
                        bangumi_subject_name: season.bangumi_subject_name,
                        bangumi_season_num: season.bangumi_season_num,
                        bangumi_subject_image: season.bangumi_subject_image_url,
                        tmdb_series_id: season.tmdb_series_id,
                        tmdb_series_name: season.tmdb_series_name,
                        tmdb_season_num: season.tmdb_season_num,
                        tmdb_season_name: season.tmdb_season_name,
                        bangumi_to_tmdb_episode_offset: season.bangumi_to_tmdb_episode_offset,
                        disp_series_name,
                        disp_season_name,
                        disp_subgroup_name,
                        disp_season_num,
                        conf_tmdb_episode_offset: 0,
                        conf_language: "".to_string(),
                        conf_codec: "".to_string(),
                        conf_season_num: -1,
                        conf_bangumi_episode_offset: 0,
                    };
                    create_season(&season);
                    create_item(&item);
                }
                None => {
                    // If the season is not found in the cache, warn and skip
                    log::warn!("Season not found in cache: Mikan {:?}", item.mikan_subject_id);
                    continue;
                }
            }
        }
    }
}


pub fn auto_season_config_clean() {
    let seasons = read_seasons();
    for season in seasons {
        // get episode list, add (language, codec) pair config to candidates
        let items = read_season_items(season.mikan_subject_id, season.mikan_subgroup_id);
        let mut conf_candidates = HashSet::new();
        for item in items {
            conf_candidates.insert((item.mikan_parsed_language.clone(), item.mikan_parsed_codec.clone()));
        }
        // Rank the candidates, and choose the best candidate as config, update config
        // Priority: hans*4 + hant * 2 + jpn * 1
        let mut conf_rank = Vec::new();
        for conf in &conf_candidates {
            let mut rank = 0;
            if conf.0.contains("hans") {
                rank += 8;
            } else if conf.0.contains("hant") {
                rank += 4;
            } else if conf.0.contains("jpn") {
                rank += 2;
            }
            if conf.1.contains("avc") {
                rank += 1;
            }
            conf_rank.push((conf.0.clone(), conf.1.clone(), rank));
        }
        conf_rank.sort_by(|a, b| b.2.cmp(&a.2));
        let best_conf = (conf_rank[0].0.clone(), conf_rank[0].1.clone());
        // TODO: configure the filter, leaving only one type of language and codec
        update_season_config(&AnimeSeason {
            conf_language: best_conf.0,
            conf_codec: best_conf.1,
            ..season
        },
                             true,
                             false,
        );
    }
}

pub fn update_season_config(season: &AnimeSeason, delete_items: bool, fetch_items: bool) {
    let conn = get_connection().unwrap();

    conn.execute(
        "update library_anime_season set conf_tmdb_episode_offset = ?1, conf_language = ?2, conf_codec = ?3, conf_bangumi_episode_offset = ?4, where mikan_subject_id = ?5 and mikan_subgroup_id = ?6",
        &[
            &season.conf_tmdb_episode_offset.to_string(),
            &season.conf_language,
            &season.conf_codec,
            &season.conf_bangumi_episode_offset.to_string(),
            &season.mikan_subject_id.to_string(),
            &season.mikan_subgroup_id.to_string(),
        ],
    ).unwrap();

    // delete items that do not obey the language and codec restriction
    if delete_items {
        let items = read_season_items(season.mikan_subject_id, season.mikan_subgroup_id);
        for item in items {
            if season.conf_language != "" && season.conf_language != item.mikan_parsed_language {
                delete_item(&item.mikan_item_uuid);
            } else if season.conf_codec != "" && season.conf_codec != item.mikan_parsed_codec {
                delete_item(&item.mikan_item_uuid);
            }
        }
    }

    // Update the episode number by new offset
    // for items in library_anime_season_item, 
    // where mikan_subject_id = season.mikan_subject_id 
    // and mikan_subgroup_id = season.mikan_subgroup_id,
    // update disp_episode_num = mikan_parsed_episode_num + conf_tmdb_episode_offset
    conn.execute(
        "update library_anime_season_item set disp_episode_num = mikan_parsed_episode_num + ?1 where mikan_subject_id = ?2 and mikan_subgroup_id = ?3",
        &[
            &season.conf_tmdb_episode_offset.to_string(),
            &season.mikan_subject_id.to_string(),
            &season.mikan_subgroup_id.to_string(),
        ],
    ).unwrap();

    // fetch items from the rss feed
    // TODO: fetch with updated config
    if fetch_items {
        let url = format!("https://mikanani.me/RSS/Bangumi?bangumiId={}&subgroupid={}", season.mikan_subject_id, season.mikan_subgroup_id);
        let items = mikan_parser::update_rss(&url).unwrap();
        let items = mikan_parser::expand_history_episodes(items);
        update_library(&items);
    }
}

use std::collections::HashSet;
use std::sync::RwLock;
use rusqlite::{Connection};
use crate::utils::rss_parser::MikanItem;
use crate::utils::rss_parser;

use lazy_static::lazy_static;

#[derive(Debug)]
struct InitedDb {
    inited: bool,
}

impl InitedDb {
    fn new() -> InitedDb {
        InitedDb {
            inited: false,
        }
    }

    fn set_inited(&mut self) {
        self.inited = true;
    }
}

lazy_static! {
    static ref INITED_DB: RwLock<InitedDb> = RwLock::new(InitedDb::new());
}

#[derive(Debug, Clone)]
pub struct AnimeSeason {
    pub mikan_bangumi_id: i32,      // primary key
    pub mikan_subgroup_id: i32,     // primary key
    pub mikan_bangumi_title: String,
    pub language: String,
    pub codec: String,
    pub episode_offset: i32,
}

#[derive(Debug, Clone)]
pub struct AnimeSeasonItem {
    pub mikan_item_uuid: String,
    pub mikan_bangumi_id: i32,
    pub mikan_subgroup_id: i32,
    pub mikan_bangumi_title: String,
    pub mikan_item_title: String,
    pub mikan_magnet_link: String,
    pub mikan_pub_date: String,
    pub episode_num: i32,
    pub episode_num_offseted: i32,
    pub language: String,
    pub codec: String,
}

pub fn init_database() -> rusqlite::Result<()> {
    let conn = Connection::open("data/database/media_library.db")?;

    conn.execute(
        "create table if not exists anime_season (
            mikan_bangumi_id integer,
            mikan_subgroup_id integer,
            mikan_bangumi_title text,
            language text,
            codec text,
            episode_offset integer,
            primary key (mikan_bangumi_id, mikan_subgroup_id)
        )",
        [],
    )?;

    conn.execute(
        "create table if not exists anime_season_item (
            mikan_item_uuid text primary key,
            mikan_bangumi_id integer,
            mikan_subgroup_id integer,
            mikan_bangumi_title text,
            mikan_item_title text,
            mikan_magnet_link text,
            mikan_pub_date text,
            episode_num integer,
            episode_num_offseted integer,
            language text,
            codec text,
            foreign key (mikan_bangumi_id, mikan_subgroup_id) references anime_season (mikan_bangumi_id, mikan_subgroup_id) on delete cascade on update restrict
        )",
        [],
    )?;

    INITED_DB.write().unwrap().set_inited();
    Ok(())
}

pub(crate) fn get_db_conn() -> rusqlite::Result<Connection> {
    if !INITED_DB.read().unwrap().inited {
        init_database()?;
    }
    let conn = Connection::open("data/database/media_library.db")?;
    Ok(conn)
}

pub fn read_season_info(mikan_bgm_id: i32, mikan_sub_id: i32) -> Option<AnimeSeason> {
    let conn = get_db_conn().unwrap();
    let mut stmt = conn.prepare("select * from anime_season where mikan_bangumi_id = ?1 and mikan_subgroup_id = ?2").unwrap();
    let season_iter = stmt.query_map(&[&mikan_bgm_id, &mikan_sub_id], |row| {
        Ok(AnimeSeason {
            mikan_bangumi_id: row.get(0)?,
            mikan_subgroup_id: row.get(1)?,
            mikan_bangumi_title: row.get(2)?,
            language: row.get(3)?,
            codec: row.get(4)?,
            episode_offset: row.get(5)?,
        })
    }).unwrap();

    for season in season_iter {
        return season.ok();
    }

    None
}

pub fn create_season(season: &AnimeSeason) {
    let conn = get_db_conn().unwrap();
    conn.execute(
        "insert or replace into anime_season (
            mikan_bangumi_id,
            mikan_subgroup_id,
            mikan_bangumi_title,
            language,
            codec,
            episode_offset
        ) values (?1, ?2, ?3, ?4, ?5, ?6)",
        &[
            &*season.mikan_bangumi_id.to_string(),
            &*season.mikan_subgroup_id.to_string(),
            &season.mikan_bangumi_title,
            &season.language,
            &season.codec,
            &*season.episode_offset.to_string(),
        ],
    ).unwrap();
}

pub fn create_item(item: &MikanItem) {
    let conn = get_db_conn().unwrap();
    let season = read_season_info(item.mikan_bangumi_id, item.mikan_subgroup_id).unwrap();
    let episode_num_offseted = item.episode_num + season.episode_offset;

    conn.execute(
        "insert or replace into anime_season_item (
            mikan_item_uuid,
            mikan_bangumi_id,
            mikan_subgroup_id,
            mikan_bangumi_title,
            mikan_item_title,
            mikan_magnet_link,
            mikan_pub_date,
            episode_num,
            episode_num_offseted,
            language,
            codec
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        &[
            &item.mikan_item_uuid,
            &*item.mikan_bangumi_id.to_string(),
            &*item.mikan_subgroup_id.to_string(),
            &item.mikan_bangumi_title,
            &item.mikan_item_title,
            &item.mikan_magnet_link,
            &item.mikan_pub_date,
            &*item.episode_num.to_string(),
            &*episode_num_offseted.to_string(),
            &item.language,
            &item.codec,
        ],
    ).unwrap();
}

pub fn update_library(items: &Vec<MikanItem>) {
    // For each item in the fetched updating list,
    // Match the item with the corresponding anime season
    // If the season is not found, insert the season into the database

    for item in items {
        // If the season is not found, insert the season into the database
        if let Some(season) = read_season_info(item.mikan_bangumi_id, item.mikan_subgroup_id) {
            // If the season is found, insert the item into the database if the item obeys the language and codec restriction
            if season.language == "" || season.language == item.language {
                if season.codec == "" || season.codec == item.codec {
                    create_item(&item);
                }
            }
        } else {
            // If the season is not found, insert the season into the database
            let season = AnimeSeason {
                mikan_bangumi_id: item.mikan_bangumi_id,
                mikan_subgroup_id: item.mikan_subgroup_id,
                mikan_bangumi_title: item.mikan_bangumi_title.clone(),
                language: "".to_string(),       // no restriction by default
                codec: "".to_string(),
                episode_offset: 0,
            };
            create_season(&season);
            create_item(&item);
        }
    }
}

pub fn delete_item(item_uuid: &str) {
    let conn = get_db_conn().unwrap();
    conn.execute(
        "delete from anime_season_item where mikan_item_uuid = ?1",
        &[item_uuid],
    ).unwrap();
}

#[allow(dead_code)]
pub fn delete_season(mikan_bgm_id: i32, mikan_sub_id: i32) {
    let conn = get_db_conn().unwrap();
    conn.execute(
        "delete from anime_season where mikan_bangumi_id = ?1 and mikan_subgroup_id = ?2",
        &[&mikan_bgm_id, &mikan_sub_id],
    ).unwrap();
}

pub fn read_season_items(mikan_bgm_id: i32, mikan_sub_id: i32) -> Vec<AnimeSeasonItem> {
    let conn = get_db_conn().unwrap();
    let mut stmt = conn.prepare("select * from anime_season_item where mikan_bangumi_id = ?1 and mikan_subgroup_id = ?2").unwrap();
    let item_iter = stmt.query_map(&[&mikan_bgm_id, &mikan_sub_id], |row| {
        Ok(AnimeSeasonItem {
            mikan_item_uuid: row.get(0)?,
            mikan_bangumi_id: row.get(1)?,
            mikan_subgroup_id: row.get(2)?,
            mikan_bangumi_title: row.get(3)?,
            mikan_item_title: row.get(4)?,
            mikan_magnet_link: row.get(5)?,
            mikan_pub_date: row.get(6)?,
            episode_num: row.get(7)?,
            episode_num_offseted: row.get(8)?,
            language: row.get(9)?,
            codec: row.get(10)?,
        })
    }).unwrap();

    let mut items = Vec::new();
    for item in item_iter {
        items.push(item.unwrap());
    }

    items
}

pub fn read_seasons() -> Vec<AnimeSeason> {
    let conn = get_db_conn().unwrap();
    let mut stmt = conn.prepare("select * from anime_season").unwrap();
    let season_iter = stmt.query_map([], |row| {
        Ok(AnimeSeason {
            mikan_bangumi_id: row.get(0)?,
            mikan_subgroup_id: row.get(1)?,
            mikan_bangumi_title: row.get(2)?,
            language: row.get(3)?,
            codec: row.get(4)?,
            episode_offset: row.get(5)?,
        })
    }).unwrap();

    let mut seasons = Vec::new();
    for season in season_iter {
        seasons.push(season.unwrap());
    }

    seasons
}

pub fn read_all_items() -> Vec<AnimeSeasonItem> {
    let conn = get_db_conn().unwrap();
    let mut stmt = conn.prepare("select * from anime_season_item").unwrap();
    let item_iter = stmt.query_map([], |row| {
        Ok(AnimeSeasonItem {
            mikan_item_uuid: row.get(0)?,
            mikan_bangumi_id: row.get(1)?,
            mikan_subgroup_id: row.get(2)?,
            mikan_bangumi_title: row.get(3)?,
            mikan_item_title: row.get(4)?,
            mikan_magnet_link: row.get(5)?,
            mikan_pub_date: row.get(6)?,
            episode_num: row.get(7)?,
            episode_num_offseted: row.get(8)?,
            language: row.get(9)?,
            codec: row.get(10)?,
        })
    }).unwrap();

    let mut items = Vec::new();
    for item in item_iter {
        items.push(item.unwrap());
    }

    items
}

pub fn auto_season_config_clean() {
    let seasons = read_seasons();
    for season in seasons {
        // get episode list, add (language, codec) pair config to candidates
        let items = read_season_items(season.mikan_bangumi_id, season.mikan_subgroup_id);
        let mut conf_candidates = HashSet::new();
        for item in items {
            conf_candidates.insert((item.language.clone(), item.codec.clone()));
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
        update_season_config(&AnimeSeason {
            mikan_bangumi_id: season.mikan_bangumi_id,
            mikan_subgroup_id: season.mikan_subgroup_id,
            mikan_bangumi_title: season.mikan_bangumi_title,
            language: best_conf.0,
            codec: best_conf.1,
            episode_offset: season.episode_offset,
        },
        true,
        false);
    }
}

pub fn update_season_config(season: &AnimeSeason, delete_items: bool, fetch_items: bool) {
    let conn = get_db_conn().unwrap();

    conn.execute(
        "update anime_season set language = ?1, codec = ?2, episode_offset = ?3 where mikan_bangumi_id = ?4 and mikan_subgroup_id = ?5",
        &[
            &season.language,
            &season.codec,
            &*season.episode_offset.to_string(),
            &*season.mikan_bangumi_id.to_string(),
            &*season.mikan_subgroup_id.to_string(),
        ],
    ).unwrap();

    // delete items that do not obey the language and codec restriction
    if delete_items {
        let items = read_season_items(season.mikan_bangumi_id, season.mikan_subgroup_id);
        for item in items {
            if season.language != "" && season.language != item.language {
                delete_item(&item.mikan_item_uuid);
            } else if season.codec != "" && season.codec != item.codec {
                delete_item(&item.mikan_item_uuid);
            }
        }
    }

    // fetch items from the rss feed
    if fetch_items {
        let url = format!("https://mikanani.me/RSS/Bangumi?bangumiId={}&subgroupid={}", season.mikan_bangumi_id, season.mikan_subgroup_id);
        let items = rss_parser::parse_rss(&url).unwrap();
        let items = rss_parser::expand_rss(items);
        update_library(&items);
    }
}
use std::error::Error;

use rusqlite::Connection;

use crate::module::database::get_connection;

#[derive(Debug, Clone)]
pub struct AnimeSeason {
    pub mikan_subject_id: i32,
    pub mikan_subgroup_id: i32,
    pub mikan_subject_name: String,
    pub mikan_subject_image: String,
    pub bangumi_subject_id: i32,
    pub bangumi_subject_name: String,
    pub bangumi_season_num: i32,
    pub bangumi_subject_image: String,
    pub tmdb_series_id: i32,
    pub tmdb_series_name: String,
    pub tmdb_season_num: i32,
    pub tmdb_season_name: String,
    pub bangumi_to_tmdb_episode_offset: i32,
    pub disp_series_name: String,
    pub disp_season_name: String,
    pub disp_subgroup_name: String,
    pub disp_season_num: i32,
    pub conf_episode_offset: i32,
    pub conf_language: String,
    pub conf_codec: String,
}

#[deny(dead_code)]
pub fn init_cache_library_anime_season_table(conn: &Connection) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "create table if not exists library_anime_season (
            mikan_subject_id integer primary key,
            mikan_subgroup_id integer,
            mikan_subject_name text,
            mikan_subject_image text,
            bangumi_subject_id integer,
            bangumi_subject_name text,
            bangumi_season_num integer,
            bangumi_subject_image text,
            tmdb_series_id integer,
            tmdb_series_name text,
            tmdb_season_num integer,
            tmdb_season_name text,
            bangumi_to_tmdb_episode_offset integer,
            disp_series_name text,
            disp_season_name text,
            disp_subgroup_name text,
            disp_season_num integer,
            conf_episode_offset integer,
            conf_language text,
            conf_codec text
        )",
        [],
    )?;
    Ok(())
}


pub fn read_season_info(mikan_subject_id: i32, mikan_subgroup_id: i32) -> Option<AnimeSeason> {
    let conn = get_connection().unwrap();
    let mut stmt = conn.prepare("select * from library_anime_season where mikan_subject_id = ?1 and mikan_subgroup_id = ?2").unwrap();
    let season_iter = stmt.query_map(&[&mikan_subject_id, &mikan_subgroup_id], |row| {
        Ok(AnimeSeason {
            mikan_subject_id: row.get(0)?,
            mikan_subgroup_id: row.get(1)?,
            mikan_subject_name: row.get(2)?,
            mikan_subject_image: row.get(3)?,
            bangumi_subject_id: row.get(4)?,
            bangumi_subject_name: row.get(5)?,
            bangumi_season_num: row.get(6)?,
            bangumi_subject_image: row.get(7)?,
            tmdb_series_id: row.get(8)?,
            tmdb_series_name: row.get(9)?,
            tmdb_season_num: row.get(10)?,
            tmdb_season_name: row.get(11)?,
            bangumi_to_tmdb_episode_offset: row.get(12)?,
            disp_series_name: row.get(13)?,
            disp_season_name: row.get(14)?,
            disp_subgroup_name: row.get(15)?,
            disp_season_num: row.get(16)?,
            conf_episode_offset: row.get(17)?,
            conf_language: row.get(18)?,
            conf_codec: row.get(19)?,
        })
    }).unwrap();

    for season in season_iter {
        return season.ok();
    }

    None
}

pub fn create_season(season: &AnimeSeason) {
    let conn = get_connection().unwrap();
    conn.execute(
        "insert or replace into library_anime_season (
            mikan_subject_id,
            mikan_subgroup_id,
            mikan_subject_name,
            mikan_subject_image,
            bangumi_subject_id,
            bangumi_subject_name,
            bangumi_season_num,
            bangumi_subject_image,
            tmdb_series_id,
            tmdb_series_name,
            tmdb_season_num,
            tmdb_season_name,
            bangumi_to_tmdb_episode_offset,
            disp_series_name,
            disp_season_name,
            disp_subgroup_name,
            disp_season_num,
            conf_episode_offset,
            conf_language,
            conf_codec
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
        &[
            &season.mikan_subject_id.to_string(),
            &season.mikan_subgroup_id.to_string(),
            &season.mikan_subject_name,
            &season.mikan_subject_image,
            &season.bangumi_subject_id.to_string(),
            &season.bangumi_subject_name,
            &season.bangumi_season_num.to_string(),
            &season.bangumi_subject_image,
            &season.tmdb_series_id.to_string(),
            &season.tmdb_series_name,
            &season.tmdb_season_num.to_string(),
            &season.tmdb_season_name,
            &season.bangumi_to_tmdb_episode_offset.to_string(),
            &season.disp_series_name,
            &season.disp_season_name,
            &season.disp_subgroup_name,
            &season.disp_season_num.to_string(),
            &season.conf_episode_offset.to_string(),
            &season.conf_language,
            &season.conf_codec,
        ],
    ).unwrap();
}

#[allow(dead_code)]
pub fn delete_season(mikan_subject_id: i32, mikan_subgroup_id: i32) {
    let conn = get_connection().unwrap();
    conn.execute(
        "delete from library_anime_season where mikan_subject_id = ?1 and mikan_subgroup_id = ?2",
        &[&mikan_subject_id, &mikan_subgroup_id],
    ).unwrap();
}

pub fn read_seasons() -> Vec<AnimeSeason> {
    let conn = get_connection().unwrap();
    let mut stmt = conn.prepare("select * from library_anime_season").unwrap();
    let season_iter = stmt.query_map([], |row| {
        Ok(AnimeSeason {
            mikan_subject_id: row.get(0)?,
            mikan_subgroup_id: row.get(1)?,
            mikan_subject_name: row.get(2)?,
            mikan_subject_image: row.get(3)?,
            bangumi_subject_id: row.get(4)?,
            bangumi_subject_name: row.get(5)?,
            bangumi_season_num: row.get(6)?,
            bangumi_subject_image: row.get(7)?,
            tmdb_series_id: row.get(8)?,
            tmdb_series_name: row.get(9)?,
            tmdb_season_num: row.get(10)?,
            tmdb_season_name: row.get(11)?,
            bangumi_to_tmdb_episode_offset: row.get(12)?,
            disp_series_name: row.get(13)?,
            disp_season_name: row.get(14)?,
            disp_subgroup_name: row.get(15)?,
            disp_season_num: row.get(16)?,
            conf_episode_offset: row.get(17)?,
            conf_language: row.get(18)?,
            conf_codec: row.get(19)?,
        })
    }).unwrap();

    let mut seasons = Vec::new();
    for season in season_iter {
        seasons.push(season.unwrap());
    }

    seasons
}


#[derive(Debug, Clone)]
pub struct AnimeSeasonItem {
    pub mikan_item_uuid: String,
    pub mikan_subject_id: i32,
    pub mikan_subject_name: String,
    pub mikan_subgroup_id: i32,
    pub mikan_item_title: String,
    pub mikan_item_magnet_link: String,
    pub mikan_item_pub_date: String,
    pub tmdb_series_name: String,
    pub tmdb_season_name: String,
    pub tmdb_parsed_season_num: i32,
    pub bangumi_parsed_season_num: i32,
    pub mikan_parsed_episode_num: i32,
    pub mikan_parsed_language: String,
    pub mikan_parsed_codec: String,
    pub disp_episode_num: i32,
    pub bangumi_parsed_episode_id: i32,
    pub bangumi_parsed_episode_ep: i32,
    pub bangumi_parsed_episode_sort: i32,
}

#[deny(dead_code)]
pub fn init_cache_library_anime_season_item_table(conn: &Connection) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "create table if not exists library_anime_season_item (
            mikan_item_uuid text primary key,
            mikan_subject_id integer,
            mikan_subgroup_id integer,
            mikan_subject_name text,
            mikan_item_title text,
            mikan_item_magnet_link text,
            mikan_item_pub_date text,
            tmdb_series_name text,
            tmdb_season_name text,
            tmdb_parsed_season_num integer,
            bangumi_parsed_season_num integer,
            mikan_parsed_episode_num integer,
            mikan_parsed_language text,
            mikan_parsed_codec text,
            disp_episode_num integer,
            bangumi_parsed_episode_id integer,
            bangumi_parsed_episode_ep integer,
            bangumi_parsed_episode_sort integer
        )",
        [],
    )?;
    Ok(())
}

pub fn create_item(item: &crate::module::database::cache::rss::MikanItem) {
    let conn = get_connection().unwrap();
    let season = read_season_info(item.mikan_subject_id, item.mikan_subgroup_id).unwrap();
    let episode_num_offseted = item.mikan_parsed_episode_num + season.conf_episode_offset;

    conn.execute(
        "insert or replace into library_anime_season_item (
            mikan_item_uuid,
            mikan_subject_id,
            mikan_subgroup_id,
            mikan_subject_name,
            mikan_item_title,
            mikan_item_magnet_link,
            mikan_item_pub_date,
            tmdb_series_name,
            tmdb_season_name,
            tmdb_parsed_season_num,
            bangumi_parsed_season_num,
            mikan_parsed_episode_num,
            mikan_parsed_language,
            mikan_parsed_codec,
            disp_episode_num,
            bangumi_parsed_episode_id,
            bangumi_parsed_episode_ep,
            bangumi_parsed_episode_sort
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        &[
            &item.mikan_item_uuid,
            &item.mikan_subject_id.to_string(),
            &item.mikan_subgroup_id.to_string(),
            &item.mikan_subject_name,
            &item.mikan_item_title,
            &item.mikan_item_magnet_link,
            &item.mikan_item_pub_date,
            &item.tmdb_series_name,
            &item.tmdb_season_name,
            &item.tmdb_parsed_season_num.to_string(),
            &item.bangumi_parsed_season_num.to_string(),
            &item.mikan_parsed_episode_num.to_string(),
            &item.mikan_parsed_language,
            &item.mikan_parsed_codec,
            &episode_num_offseted.to_string(),
            &item.bangumi_parsed_episode_id.to_string(),
            &item.bangumi_parsed_episode_ep.to_string(),
            &item.bangumi_parsed_episode_sort.to_string(),
        ],
    ).unwrap();
}

pub fn delete_item(item_uuid: &str) {
    let conn = get_connection().unwrap();
    conn.execute(
        "delete from library_anime_season_item where mikan_item_uuid = ?1",
        &[item_uuid],
    ).unwrap();
}


// TODO: rename to read_subject_items
pub fn read_season_items(mikan_subject_id: i32, mikan_subgroup_id: i32) -> Vec<AnimeSeasonItem> {
    let conn = get_connection().unwrap();
    let mut stmt = conn.prepare("select * from library_anime_season_item where mikan_subject_id = ?1 and mikan_subgroup_id = ?2").unwrap();
    let item_iter = stmt.query_map(&[&mikan_subject_id, &mikan_subgroup_id], |row| {
        Ok(AnimeSeasonItem {
            mikan_item_uuid: row.get(0)?,
            mikan_subject_id: row.get(1)?,
            mikan_subgroup_id: row.get(2)?,
            mikan_subject_name: row.get(3)?,
            mikan_item_title: row.get(4)?,
            mikan_item_magnet_link: row.get(5)?,
            mikan_item_pub_date: row.get(6)?,
            tmdb_series_name: row.get(7)?,
            tmdb_season_name: row.get(8)?,
            tmdb_parsed_season_num: row.get(9)?,
            bangumi_parsed_season_num: row.get(10)?,
            mikan_parsed_episode_num: row.get(11)?,
            mikan_parsed_language: row.get(12)?,
            mikan_parsed_codec: row.get(13)?,
            disp_episode_num: row.get(14)?,
            bangumi_parsed_episode_id: row.get(15)?,
            bangumi_parsed_episode_ep: row.get(16)?,
            bangumi_parsed_episode_sort: row.get(17)?,
        })
    }).unwrap();

    let mut items = Vec::new();
    for item in item_iter {
        items.push(item.unwrap());
    }

    items
}


pub fn read_all_items() -> Vec<AnimeSeasonItem> {
    let conn = get_connection().unwrap();
    let mut stmt = conn.prepare("select * from library_anime_season_item").unwrap();
    let item_iter = stmt.query_map([], |row| {
        Ok(AnimeSeasonItem {
            mikan_item_uuid: row.get(0)?,
            mikan_subject_id: row.get(1)?,
            mikan_subgroup_id: row.get(2)?,
            mikan_subject_name: row.get(3)?,
            mikan_item_title: row.get(4)?,
            mikan_item_magnet_link: row.get(5)?,
            mikan_item_pub_date: row.get(6)?,
            tmdb_series_name: row.get(7)?,
            tmdb_season_name: row.get(8)?,
            tmdb_parsed_season_num: row.get(9)?,
            bangumi_parsed_season_num: row.get(10)?,
            mikan_parsed_episode_num: row.get(11)?,
            mikan_parsed_language: row.get(12)?,
            mikan_parsed_codec: row.get(13)?,
            disp_episode_num: row.get(14)?,
            bangumi_parsed_episode_id: row.get(15)?,
            bangumi_parsed_episode_ep: row.get(16)?,
            bangumi_parsed_episode_sort: row.get(17)?,
        })
    }).unwrap();

    let mut items = Vec::new();
    for item in item_iter {
        items.push(item.unwrap());
    }

    items
}

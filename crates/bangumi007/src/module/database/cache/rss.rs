use std::error::Error;

use rusqlite::Connection;

use crate::module::database::get_connection;
use crate::module::utils::error::new_warn;

#[derive(Debug, Clone)]
pub struct MikanItem {
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
    pub bangumi_parsed_episode_id: i32,
    pub bangumi_parsed_episode_ep: i32,
    pub bangumi_parsed_episode_sort: i32,
}

#[deny(dead_code)]
pub fn init_cache_mikan_item_table(conn: &Connection) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "create table if not exists cache_mikan_item (
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
            bangumi_parsed_episode_id integer,
            bangumi_parsed_episode_ep integer,
            bangumi_parsed_episode_sort integer
        )",
        [],
    )?;
    Ok(())
}

pub fn insert_item_to_cache(item: &MikanItem) -> Result<(), Box<dyn Error>> {
    let conn = get_connection()?;
    conn.execute(
        "insert or replace into cache_mikan_item (
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
            bangumi_parsed_episode_id,
            bangumi_parsed_episode_ep,
            bangumi_parsed_episode_sort
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        &[
            &item.mikan_item_uuid,
            &*item.mikan_subject_id.to_string(),
            &*item.mikan_subgroup_id.to_string(),
            &item.mikan_subject_name,
            &item.mikan_item_title,
            &item.mikan_item_magnet_link,
            &item.mikan_item_pub_date,
            &item.tmdb_series_name,
            &item.tmdb_season_name,
            &*item.tmdb_parsed_season_num.to_string(),
            &*item.bangumi_parsed_season_num.to_string(),
            &*item.mikan_parsed_episode_num.to_string(),
            &item.mikan_parsed_language,
            &item.mikan_parsed_codec,
            &*item.bangumi_parsed_episode_id.to_string(),
            &*item.bangumi_parsed_episode_ep.to_string(),
            &*item.bangumi_parsed_episode_sort.to_string(),
        ],
    )?;
    Ok(())
}


/// Filter out the items that are already cached, return the uncached items
pub fn filter_uncached_items(items: &Vec<MikanItem>) -> Vec<MikanItem> {
    let conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return items.clone(),
    };

    let mut result: Vec<MikanItem> = Vec::new();

    for item in items {
        let mut stmt = conn.prepare("select mikan_item_uuid from cache_mikan_item where mikan_item_uuid = ?1").unwrap();
        let mut rows = stmt.query(&[&item.mikan_item_uuid]).unwrap();

        match rows.next() {
            Ok(Some(_)) => {} // If there is a match result, seen before, do not append
            Ok(None) => result.push(item.clone()), // If there is no match result, unseen, append
            Err(_) => result.push(item.clone()), // If there is an error, treat as unseen
        }
    }
    result
}

/// Fetch the details of the cached items from the database
pub fn fetch_cached_items(items: &Vec<MikanItem>) -> Vec<MikanItem> {
    let conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return items.clone(),
    };

    let mut result: Vec<MikanItem> = Vec::new();

    for item in items {
        let mut stmt = conn.prepare("select * from cache_mikan_item where mikan_item_uuid = ?1").unwrap();
        let mut rows = stmt.query(&[&item.mikan_item_uuid]).unwrap();

        match rows.next() {
            Ok(Some(row)) => {
                result.push(MikanItem {
                    mikan_item_uuid: row.get(0).unwrap(),
                    mikan_subject_id: row.get(1).unwrap(),
                    mikan_subgroup_id: row.get(2).unwrap(),
                    mikan_subject_name: row.get(3).unwrap(),
                    mikan_item_title: row.get(4).unwrap(),
                    mikan_item_magnet_link: row.get(5).unwrap(),
                    mikan_item_pub_date: row.get(6).unwrap(),
                    tmdb_series_name: row.get(7).unwrap(),
                    tmdb_season_name: row.get(8).unwrap(),
                    tmdb_parsed_season_num: row.get(9).unwrap(),
                    bangumi_parsed_season_num: row.get(10).unwrap(),
                    mikan_parsed_episode_num: row.get(11).unwrap(),
                    mikan_parsed_language: row.get(12).unwrap(),
                    mikan_parsed_codec: row.get(13).unwrap(),
                    bangumi_parsed_episode_id: row.get(14).unwrap(),
                    bangumi_parsed_episode_ep: row.get(15).unwrap(),
                    bangumi_parsed_episode_sort: row.get(16).unwrap(),
                });
            }
            Ok(None) => {} // If there is no match, skip
            Err(_) => {} // If there is an error, skip
        }
    }
    result
}

#[derive(Debug, Clone)]
pub struct MikanSubject {
    pub mikan_subject_id: i32,
    pub mikan_subject_image_url: String,
    pub bangumi_subject_id: i32,
    pub bangumi_subject_name: String,
    pub bangumi_season_num: i32,
    pub bangumi_subject_image_url: String,
    pub tmdb_series_id: i32,
    pub tmdb_series_name: String,
    pub tmdb_season_num: i32,
    pub tmdb_season_name: String,
    pub bangumi_to_tmdb_episode_offset: i32,
}

#[deny(dead_code)]
pub fn init_cache_mikan_subject_table(conn: &Connection) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "create table if not exists cache_mikan_subject (
            mikan_subject_id integer primary key,
            mikan_subject_image_url text,
            bangumi_subject_id integer,
            bangumi_subject_name text,
            bangumi_season_num integer,
            bangumi_subject_image_url text,
            tmdb_series_id integer,
            tmdb_series_name text,
            tmdb_season_num integer,
            tmdb_season_name text,
            bangumi_to_tmdb_episode_offset integer default 0
        )",
        [],
    )?;
    Ok(())
}

pub fn insert_subject_to_cache(subject: &MikanSubject) -> Result<(), Box<dyn Error>> {
    let conn = get_connection()?;
    conn.execute(
        "insert or replace into cache_mikan_subject (
            mikan_subject_id,
            mikan_subject_image_url,
            bangumi_subject_id,
            bangumi_subject_name,
            bangumi_season_num,
            bangumi_subject_image_url,
            tmdb_series_id,
            tmdb_series_name,
            tmdb_season_num,
            tmdb_season_name,
            bangumi_to_tmdb_episode_offset
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        &[
            &*subject.mikan_subject_id.to_string(),
            &subject.mikan_subject_image_url,
            &*subject.bangumi_subject_id.to_string(),
            &subject.bangumi_subject_name,
            &*subject.bangumi_season_num.to_string(),
            &subject.bangumi_subject_image_url,
            &*subject.tmdb_series_id.to_string(),
            &subject.tmdb_series_name,
            &*subject.tmdb_season_num.to_string(),
            &subject.tmdb_season_name,
            &*subject.bangumi_to_tmdb_episode_offset.to_string(),
        ],
    )?;
    Ok(())
}

/// Get the bangumi_id of a mikanani subject by it's subject_id.
pub fn fetch_mikan_subject_info(mikan_subject_id: i32) -> Option<MikanSubject> {
    let conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return None,
    };

    let mut stmt = conn.prepare("select * from cache_mikan_subject where mikan_subject_id = ?1").unwrap();
    let mut rows = stmt.query(&[&mikan_subject_id]).unwrap();

    match rows.next() {
        Ok(Some(row)) => {
            Some(MikanSubject {
                mikan_subject_id: row.get(0).unwrap(),
                mikan_subject_image_url: row.get(1).unwrap(),
                bangumi_subject_id: row.get(2).unwrap(),
                bangumi_subject_name: row.get(3).unwrap(),
                bangumi_season_num: row.get(4).unwrap(),
                bangumi_subject_image_url: row.get(5).unwrap(),
                tmdb_series_id: row.get(6).unwrap(),
                tmdb_series_name: row.get(7).unwrap(),
                tmdb_season_num: row.get(8).unwrap(),
                tmdb_season_name: row.get(9).unwrap(),
                bangumi_to_tmdb_episode_offset: row.get(10).unwrap(),
            })
        }
        Ok(None) => None,
        Err(_) => None,
    }
}

#[derive(Debug, Clone)]
pub struct BangumiEpisode {
    pub subject_id: i32,
    pub episode_id: i32,
    pub episode_type: i32,
    pub episode_ep: i32,    // raw index of episode
    pub episode_sort: i32,  // display index of episode
    pub episode_name: String,
    pub episode_name_cn: String,
    pub episode_airdate: String,
}

#[deny(dead_code)]
pub fn init_cache_bangumi_episode_table(conn: &Connection) -> Result<(), Box<dyn Error>> {
    // Create empty table, deleting existing table
    conn.execute(
        "create table if not exists cache_bangumi_episode (
            subject_id integer,
            episode_id integer primary key,
            episode_type integer,
            episode_ep integer,
            episode_sort integer,
            episode_name text,
            episode_name_cn text,
            episode_airdate text
        )",
        [],
    )?;
    conn.execute("delete from cache_bangumi_episode", [])?;
    Ok(())
}

pub fn insert_bangumi_episode_to_cache(episode: &BangumiEpisode) -> Result<(), Box<dyn Error>> {
    let conn = get_connection()?;
    conn.execute(
        "insert or replace into cache_bangumi_episode (
            subject_id,
            episode_id,
            episode_type,
            episode_ep,
            episode_sort,
            episode_name,
            episode_name_cn,
            episode_airdate
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        &[
            &*episode.subject_id.to_string(),
            &*episode.episode_id.to_string(),
            &*episode.episode_type.to_string(),
            &*episode.episode_ep.to_string(),
            &*episode.episode_sort.to_string(),
            &episode.episode_name,
            &episode.episode_name_cn,
            &episode.episode_airdate,
        ],
    )?;
    Ok(())
}

pub fn get_bangumi_episodes(bangumi_subject_id: i32) -> Result<Vec<BangumiEpisode>, Box<dyn Error>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare("select * from cache_bangumi_episode where subject_id = ?1").unwrap();
    let rows = stmt.query_map(&[&bangumi_subject_id], |row| {
        Ok(BangumiEpisode {
            subject_id: row.get(0)?,
            episode_id: row.get(1)?,
            episode_type: row.get(2)?,
            episode_ep: row.get(3)?,
            episode_sort: row.get(4)?,
            episode_name: row.get(5)?,
            episode_name_cn: row.get(6)?,
            episode_airdate: row.get(7)?,
        })
    })?;

    let mut result: Vec<BangumiEpisode> = Vec::new();
    for episode in rows {
        result.push(episode?);
    }
    Ok(result)
}

pub fn get_bangumi_episode_info(bangumi_episode_id: i32) -> Result<BangumiEpisode, Box<dyn Error>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare("select * from cache_bangumi_episode where episode_id = ?1").unwrap();
    let mut rows = stmt.query(&[&bangumi_episode_id])?;

    match rows.next() {
        Ok(Some(row)) => {
            Ok(BangumiEpisode {
                subject_id: row.get(0)?,
                episode_id: row.get(1)?,
                episode_type: row.get(2)?,
                episode_ep: row.get(3)?,
                episode_sort: row.get(4)?,
                episode_name: row.get(5)?,
                episode_name_cn: row.get(6)?,
                episode_airdate: row.get(7)?,
            })
        }
        Ok(None) => Err(new_warn("Bangumi episode info not found in cache.")),
        Err(_) => Err(new_warn("Bangumi episode info not found in cache.")),
    }
}
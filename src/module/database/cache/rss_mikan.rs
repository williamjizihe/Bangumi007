use std::error::Error;
use rusqlite::Connection;

use crate::module::database::get_connection;

#[derive(Debug, Clone)]
pub struct MikanItem {
    pub mikan_item_uuid: String,
    pub mikan_subject_id: i32,
    pub mikan_subgroup_id: i32,
    pub mikan_subject_title: String,
    pub mikan_item_title: String,
    pub mikan_item_magnet_link: String,
    pub mikan_item_pub_date: String,
    pub episode_num: i32,
    pub language: String,
    pub codec: String,
}

#[deny(dead_code)]
pub fn init_cache_mikan_item_table(conn: Connection) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "create table if not exists mikan_item (
            mikan_item_uuid text primary key,
            mikan_subject_id integer,
            mikan_subgroup_id integer,
            mikan_subject_title text,
            mikan_item_title text,
            mikan_item_magnet_link text,
            mikan_item_pub_date text,
            episode_num integer,
            language text,
            codec text
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
            mikan_subject_title,
            mikan_item_title,
            mikan_item_magnet_link,
            mikan_item_pub_date,
            episode_num,
            language,
            codec
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        &[
            &item.mikan_item_uuid,
            &*item.mikan_subject_id.to_string(),
            &*item.mikan_subgroup_id.to_string(),
            &item.mikan_subject_title,
            &item.mikan_item_title,
            &item.mikan_item_magnet_link,
            &item.mikan_item_pub_date,
            &*item.episode_num.to_string(),
            &item.language,
            &item.codec,
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
                    mikan_subject_title: row.get(3).unwrap(),
                    mikan_item_title: row.get(4).unwrap(),
                    mikan_item_magnet_link: row.get(5).unwrap(),
                    mikan_item_pub_date: row.get(6).unwrap(),
                    episode_num: row.get(7).unwrap(),
                    language: row.get(8).unwrap(),
                    codec: row.get(9).unwrap(),
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
    pub mikan_subject_bangumi_id: i32,
}

pub fn init_cache_mikan_subject_table(conn: Connection) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "create table if not exists mikan_subject (
            mikan_subject_id integer primary key,
            mikan_subject_bangumi_id integer
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
            mikan_subject_bangumi_id
        ) values (?1, ?2)",
        &[
            &*subject.mikan_subject_id.to_string(),
            &*subject.mikan_subject_bangumi_id.to_string(),
        ],
    )?;
    Ok(())
}

/// Get the bangumi_id of a mikanani subject by it's subject_id.
pub fn fetch_mikan_subject_bangumi_id(mikan_subject_id: i32) -> Option<i32> {
    let conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return None,
    };

    let mut stmt = conn.prepare("select mikan_subject_bangumi_id from cache_mikan_subject where mikan_subject_id = ?1").unwrap();
    let mut rows = stmt.query(&[&mikan_subject_id]).unwrap();

    match rows.next() {
        Ok(Some(row)) => Some(row.get(0).unwrap()),
        Ok(None) => None,
        Err(_) => None,
    }
}


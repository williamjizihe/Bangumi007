use std::sync::RwLock;
use rusqlite::{Connection, Result};
use crate::utils::rss_parser::MikanItem;
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


pub fn init_database() -> Result<()> {
    let conn = Connection::open("data/database/rss_cache.db")?;

    conn.execute(
        "create table if not exists mikan_item (
            mikan_item_uuid text primary key,
            mikan_bangumi_id integer,
            mikan_subgroup_id integer,
            mikan_bangumi_title text,
            mikan_item_title text,
            mikan_magnet_link text,
            mikan_pub_date text,
            episode_num integer,
            language text,
            codec text
        )",
        [],
    )?;
    INITED_DB.write().unwrap().set_inited();
    Ok(())
}

fn get_db_conn() -> Result<Connection> {
    if !INITED_DB.read().unwrap().inited {
        init_database()?;
    }
    let conn = Connection::open("data/database/rss_cache.db")?;
    Ok(conn)
}

pub(crate) fn insert_item(
    item: &MikanItem,
) -> Result<()> {
    let conn = get_db_conn()?;
    let MikanItem {
        mikan_item_uuid,
        mikan_bangumi_id,
        mikan_subgroup_id,
        mikan_bangumi_title,
        mikan_item_title,
        mikan_magnet_link,
        mikan_pub_date,
        episode_num,
        language,
        codec,
    } = item;
    conn.execute(
        "insert or replace into mikan_item (
            mikan_item_uuid,
            mikan_bangumi_id,
            mikan_subgroup_id,
            mikan_bangumi_title,
            mikan_item_title,
            mikan_magnet_link,
            mikan_pub_date,
            episode_num,
            language,
            codec
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        &[
            mikan_item_uuid,
            &*mikan_bangumi_id.to_string(),
            &*mikan_subgroup_id.to_string(),
            mikan_bangumi_title,
            mikan_item_title,
            mikan_magnet_link,
            mikan_pub_date,
            &*episode_num.to_string(),
            language,
            codec,
        ],
    )?;

    Ok(())
}

pub(crate) fn find_items_not_in_db(items: &Vec<MikanItem>) -> Vec<MikanItem> {
    let conn = get_db_conn().unwrap();
    let mut result: Vec<MikanItem> = Vec::new();
    for item in items {
        let mut stmt = conn.prepare("select mikan_item_uuid from mikan_item where mikan_item_uuid = ?1").unwrap();
        let mut rows = stmt.query(&[&item.mikan_item_uuid]).unwrap();
        match rows.next() {
            Ok(Some(_)) => {} // If there is a row, do nothing
            Ok(None) => result.push(item.clone()), // If there is no row, push the item
            Err(_) => {} // If there is an error, do nothing (or handle the error as needed)
        }
    }
    result
}

pub fn fetch_info_by_db(items: &Vec<MikanItem>) -> Vec<MikanItem> {
    let conn = get_db_conn().unwrap();
    let mut result: Vec<MikanItem> = Vec::new();
    for item in items {
        let mut stmt = conn.prepare("select * from mikan_item where mikan_item_uuid = ?1").unwrap();
        let mut rows = stmt.query(&[&item.mikan_item_uuid]).unwrap();
        match rows.next() {
            Ok(Some(row)) => {
                let mikan_item_uuid: String = row.get(0).unwrap();
                let mikan_bangumi_id: i32 = row.get(1).unwrap();
                let mikan_subgroup_id: i32 = row.get(2).unwrap();
                let mikan_bangumi_title: String = row.get(3).unwrap();
                let mikan_item_title: String = row.get(4).unwrap();
                let mikan_magnet_link: String = row.get(5).unwrap();
                let mikan_pub_date: String = row.get(6).unwrap();
                let episode_num: i32 = row.get(7).unwrap();
                let language: String = row.get(8).unwrap();
                let codec: String = row.get(9).unwrap();
                result.push(MikanItem {
                    mikan_item_uuid,
                    mikan_bangumi_id,
                    mikan_subgroup_id,
                    mikan_bangumi_title,
                    mikan_item_title,
                    mikan_magnet_link,
                    mikan_pub_date,
                    episode_num,
                    language,
                    codec,
                });
            }
            Ok(None) => {} // If there is no row, do nothing
            Err(_) => {} // If there is an error, do nothing (or handle the error as needed)
        }
    }
    result
}

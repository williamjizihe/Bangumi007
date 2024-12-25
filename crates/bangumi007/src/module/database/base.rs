use std::sync::RwLock;
use std::fs;

use lazy_static::lazy_static;
use rusqlite::Connection;

use crate::module::database::cache::rss::{init_cache_bangumi_episode_table, init_cache_mikan_item_table, init_cache_mikan_subject_table};
use crate::module::database::library::{init_cache_library_anime_season_item_table, init_cache_library_anime_season_table};

const DATABASE_PATH: &str = "data/database/database.db";

#[derive(Debug)]
pub struct InitedDb {
    inited: bool,
}

impl InitedDb {
    fn new() -> InitedDb {
        InitedDb {
            inited: false,
        }
    }

    pub fn set_inited(&mut self) {
        self.inited = true;
    }
}

lazy_static! {
    pub static ref INITED_DB: RwLock<InitedDb> = RwLock::new(InitedDb::new());
}

#[deny(dead_code)]
pub fn init_database() -> Result<(), Box<dyn std::error::Error>> {
    let database_dir = std::path::Path::new("data/database");
    if !database_dir.exists() {
        fs::create_dir_all(database_dir)?;
    }

    let conn = Connection::open(DATABASE_PATH)?;
    init_cache_mikan_item_table(&conn)?;
    init_cache_mikan_subject_table(&conn)?;
    init_cache_library_anime_season_table(&conn)?;
    init_cache_library_anime_season_item_table(&conn)?;
    init_cache_bangumi_episode_table(&conn)?;
    INITED_DB.write().unwrap().set_inited();
    Ok(())
}

pub fn get_connection() -> Result<Connection, Box<dyn std::error::Error>> {
    if !INITED_DB.read().unwrap().inited {
        init_database()?;
        // panic!("Database not inited");
    }
    let conn = Connection::open(DATABASE_PATH)?;
    Ok(conn)
}
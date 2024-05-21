use std::sync::RwLock;

use lazy_static::lazy_static;
use rusqlite::Connection;
use crate::module::database::cache::rss_mikan::init_cache_mikan_item_table;

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
    let conn = Connection::open(DATABASE_PATH)?;
    init_cache_mikan_item_table(conn)?;
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
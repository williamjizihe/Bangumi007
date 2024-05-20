use std::collections::HashSet;
use std::fmt::Debug;
use roxmltree::Document;
use reqwest::blocking::get;
use html_escape::decode_html_entities;
use fancy_regex::Regex;
use retry::delay::Fixed;
use retry::retry;
use crate::module::parser::mikan_parser::ParseError::CustomError;

use std::sync::RwLock;
use rusqlite::{Connection, Result};
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
        mikan_subject_id,
        mikan_subgroup_id,
        mikan_subject_title,
        mikan_item_title,
        mikan_item_magnet_link,
        mikan_item_pub_date,
        episode_num,
        language,
        codec,
    } = item;
    conn.execute(
        "insert or replace into mikan_item (
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
            mikan_item_uuid,
            &*mikan_subject_id.to_string(),
            &*mikan_subgroup_id.to_string(),
            mikan_subject_title,
            mikan_item_title,
            mikan_item_magnet_link,
            mikan_item_pub_date,
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
                let mikan_subject_id: i32 = row.get(1).unwrap();
                let mikan_subgroup_id: i32 = row.get(2).unwrap();
                let mikan_subject_title: String = row.get(3).unwrap();
                let mikan_item_title: String = row.get(4).unwrap();
                let mikan_item_magnet_link: String = row.get(5).unwrap();
                let mikan_item_pub_date: String = row.get(6).unwrap();
                let episode_num: i32 = row.get(7).unwrap();
                let language: String = row.get(8).unwrap();
                let codec: String = row.get(9).unwrap();
                result.push(MikanItem {
                    mikan_item_uuid,
                    mikan_subject_id,
                    mikan_subgroup_id,
                    mikan_subject_title,
                    mikan_item_title,
                    mikan_item_magnet_link,
                    mikan_item_pub_date,
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

#[derive(Debug, Clone)]
pub struct MikanSubject {
    pub mikan_subject_id: i32,
    pub mikan_subject_bangumi_id: i32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ParseError {
    ReqwestError(reqwest::Error),
    RoxmltreeError(roxmltree::Error),
    RegexError(fancy_regex::Error),
    RusqliteError(rusqlite::Error),
    CustomError(String),
}

fn parse_codec(title: &str) -> String {
    // avc: H.264 AVC MP4
    // hevc: H.265 HEVC MKV
    // vp9: VP9
    // av1: AV1
    let mut result: String = "".to_string();
    if title.contains("AVC") || title.contains("avc") ||
        title.contains("264") ||
        title.contains("MP4") || title.contains("mp4") {
        result.push_str("avc,");
    }
    if title.contains("HEVC") || title.contains("hevc") ||
        title.contains("265") ||
        title.contains("MKV") || title.contains("mkv") {
        result.push_str("hevc,");
    }
    if title.contains("VP9") || title.contains("vp9") {
        result.push_str("vp9,");
    }
    if title.contains("AV1") || title.contains("av1") {
        result.push_str("av1,");
    }
    result
}

fn parse_language(title: &str) -> String {
    // hans: CHS GB 简
    // hant: CHT BIG5 繁
    // jpn: JP 日 双语
    let mut result: String = "".to_string();
    if title.contains("CHS") || title.contains("GB") ||
        title.contains("chs") || title.contains("gb") ||
        title.contains("简") {
        result.push_str("hans,");
    }
    if title.contains("CHT") || title.contains("BIG5") ||
        title.contains("cht") || title.contains("big5") ||
        title.contains("繁") {
        result.push_str("hant,");
    }
    if title.contains("JP") ||
        title.contains("中日") || title.contains("汉日") ||
        title.contains("简日") || title.contains("繁日") ||
        title.contains("双语") {
        result.push_str("jpn,");
    }
    result
}

// TODO: exclude \d+-\d from title

//
pub fn parse_rss(url: &str) -> Result<Vec<MikanItem>, ParseError> {
    // RSS parser
    // Get the RSS feed from the URL and parse it with roxmltree.
    // First read channel title and link
    // Then read all the items in the feed channel. and get the title and link of each item.

    // Get RSS feed
    // "get(url).unwrap().text().unwrap();" with retry
    let response = retry(Fixed::from_millis(5000), || {
        match get(url) {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response.text().unwrap())
                } else {
                    log::warn!("Failed to get rss, status code is not 200");
                    Err(CustomError("Failed to get rss, status code is not 200".to_string()))
                }
            }
            Err(_) => {
                log::warn!("Failed to get rss");
                Err(CustomError("Failed to get rss".to_string()))
            }
        }
    }).unwrap();

    // Parse the response DOM
    let dom = Document::parse(&response).unwrap();

    // RSS channel DOM object
    let channel = dom.root().first_element_child().unwrap().first_element_child().unwrap();
    log::debug!("Channel Name: {}", channel.tag_name().name());

    // Parse Channel title and RSS link url
    let channel_title = channel
        .children()
        .find(|n| n.has_tag_name("title")).unwrap()
        .text().unwrap();
    let channel_link = channel
        .children()
        .find(|n| n.has_tag_name("link")).unwrap()
        .text().unwrap();
    log::debug!("Channel Title: {}, Channel Link: {}", channel_title, channel_link);

    // Parse items
    let items = channel.children().filter(|n| n.has_tag_name("item"));
    let mut result: Vec<MikanItem> = Vec::new();
    for item in items {

        // Item Title
        let title = item
            .children()
            .find(|n| n.has_tag_name("title")).unwrap()
            .text()
            .unwrap();

        // Episode Link (on Mikanani)
        let link = item
            .children()
            .find(|n| n.has_tag_name("link")).unwrap()
            .text()
            .unwrap();

        // Episode Torrent's pubDate
        let torrent = item
            .children()
            .find(|n| n.has_tag_name("torrent")).unwrap();
        let pubdate = torrent
            .children()
            .find(|n| n.has_tag_name("pubDate")).unwrap()
            .text()
            .unwrap();

        // Mikan Episode UUID (Torrent/Maglink hash): slice from link
        let uuid = link.rfind('/').unwrap();
        let uuid = &link[uuid + 1..];
        result.push(MikanItem {
            mikan_item_uuid: uuid.to_string(),          // Item UUID
            mikan_subject_id: 0,
            mikan_subgroup_id: 0,
            mikan_subject_title: "".to_string(),
            mikan_item_title: title.to_string(),        // Item Title
            mikan_item_magnet_link: "".to_string(),
            mikan_item_pub_date: pubdate.to_string(),   // Torrent PubDate
            episode_num: parse_filename_to_episode(&title).unwrap_or(-1),  // Episode Number
            language: parse_language(&title),           // Language
            codec: parse_codec(&title),                 // Codec
        });
    }

    // Find the items in database, get the list of items not in database
    // For each item not in database, parse the episode information
    // Then insert the item into the database
    let items_not_in_db = find_items_not_in_db(&result);
    for item in items_not_in_db {
        let item = retry(Fixed::from_millis(5000), || {
            match parse_episode(&item) {
                Ok(item) => Ok(item),
                Err(_) => {
                    log::warn!("Failed to parse episode info");
                    Err("Failed to parse episode info")
                }
            }
        }).unwrap();
        insert_item(&item).unwrap();
    }
    let items_full = fetch_info_by_db(&result);
    Ok(items_full)
}

pub fn expand_rss(items: Vec<MikanItem>) -> Vec<MikanItem> {
    // Expand the RSS and get history episodes
    // For each item in items, get its bangumi id and subgroup id
    // Then visit https://mikanani.me/RSS/Bangumi?bangumiId={}&subgroupid={}
    // Parse the RSS and get all the episodes
    let mut result = Vec::new();
    // HashSet to store visited bgm-sub pairs
    let mut visited = HashSet::new();
    for item in items {
        if visited.contains(&(item.mikan_subject_id, item.mikan_subgroup_id)) {
            continue;
        }
        let url = format!("https://mikanani.me/RSS/Bangumi?bangumiId={}&subgroupid={}", item.mikan_subject_id, item.mikan_subgroup_id);
        let items_full = parse_rss(&url).unwrap();
        // Add to the result
        for item in items_full {
            result.push(item);
        }
        // Add to visited
        visited.insert((item.mikan_subject_id, item.mikan_subgroup_id));
    }
    result
}

fn parse_episode(item: &MikanItem) -> Result<MikanItem, ParseError> {
    // build url from item's uuid
    let url = format!("https://mikanani.me/Home/Episode/{}", item.mikan_item_uuid);

    let response = retry(Fixed::from_millis(5000), || {
        match get(&url) {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response.text().unwrap())
                } else {
                    log::warn!("Failed to get episode info, status code is not 200");
                    Err(CustomError("Failed to get episode info, status code is not 200".to_string()))
                }
            }
            Err(_) => {
                log::warn!("Failed to get episode info");
                Err(CustomError("Failed to get episode info".to_string()))
            }
        }
    }).unwrap();
    // Find the first substring "bangumi-title" and get the text of the next sibling node, not using xml, but as plain text
    let title = response.find("bangumi-title").unwrap();
    // Find the following "<a" substring after the "bangumi-title"
    let link = response[title..].find("<a").unwrap();
    // Find the following ">" substring after the "<a"
    let start = response[title + link..].find(">").unwrap();
    // Find the following "<" substring after the ">"
    let end = response[title + link + start..].find("<").unwrap();
    let title = &response[title + link + start + 1..title + link + start + end];
    // decode utf-8 as html entities
    let title = decode_html_entities(title);
    log::debug!("Title: {}", title);

    // Find "magnet-link-wrap" substring
    let magnet = response.find("magnet-link-wrap").unwrap();
    // Find the ">" substring after the "magnet-link-wrap"
    let start = response[magnet..].find(">").unwrap();
    // Find the "<" substring after the ">"
    let end = response[magnet + start..].find("<").unwrap();
    let subgroup = &response[magnet + start + 1..magnet + start + end];
    let subgroup = decode_html_entities(subgroup);
    log::debug!("Subgroup: {}", subgroup);

    // Find "href="magnet:" substring
    let magnet = response.find("href=\"magnet:").unwrap();
    // Find the "\">" substring after the "href=\"magnet:"
    let start = response[magnet..].find("\">").unwrap();
    let magnet = &response[magnet + 6..magnet + start];
    let magnet = decode_html_entities(magnet);
    log::debug!("Magnet: {}", magnet);

    // Parse episode title
    // Find "episode-title"> substring
    let episode_title = response.find("episode-title\">").unwrap();
    // Find the "<" substring after the "episode-title\">"
    let end = response[episode_title..].find("<").unwrap();
    let ep_title = &response[episode_title + 15..episode_title + end];
    let ep_title = decode_html_entities(ep_title);
    // Find from right, the " [" substring
    let end = ep_title.rfind(" [").unwrap();
    let ep_title = &ep_title[..end];
    log::debug!("Episode Title: {}", ep_title);

    // Parse bangumi id and subgroup id
    // e.g. find "?bangumiId=" + 3332(bgmid) + "&subgroupId=" + 583(subgid)
    let bgmid = response.find("?bangumiId=").unwrap();
    let bgmid = &response[bgmid + 11..];
    let bgmid = bgmid.split('&').next().unwrap();
    let bgmid = bgmid.parse::<i32>().unwrap();
    log::debug!("Bangumi ID: {}", bgmid);
    let subgid = response.find("&subgroupid=").unwrap();
    let subgid = &response[subgid + 12..];
    let subgid = subgid.split('\"').next().unwrap();
    let subgid = subgid.parse::<i32>().unwrap();
    log::debug!("Subgroup ID: {}", subgid);

    Ok(MikanItem {
        mikan_item_uuid: item.mikan_item_uuid.to_string(),
        mikan_subject_id: bgmid,
        mikan_subgroup_id: subgid,
        mikan_subject_title: title.to_string(),
        mikan_item_title: item.mikan_item_title.to_string(),
        mikan_item_magnet_link: magnet.to_string(),
        mikan_item_pub_date: item.mikan_item_pub_date.to_string(),
        episode_num: item.episode_num,
        language: item.language.to_string(),
        codec: item.codec.to_string(),
    })
}


fn parse_filename_to_episode(filename: &str) -> Option<i32> {
    // Parse the filename to get the episode number
    let rules = [
        r"(.*) - (\d{1,4}(?!\d|p)|\d{1,4}\.\d{1,2}(?!\d|p))(?:v\d{1,2})?(?: )?(?:END)?(.*)",
        r"(.*)[\[\ E](\d{1,4}|\d{1,4}\.\d{1,2})(?:v\d{1,2})?(?: )?(?:END)?[\]\ ](.*)",
        r"(.*)\[(?:第)?(\d*\.*\d*)[话集話](?:END)?\](.*)",
        r"(.*)第?(\d*\.*\d*)[话話集](?:END)?(.*)",
        r"(.*)(?:S\d{2})?EP?(\d+)(.*)",
    ];
    for rule in rules.iter() {
        let re = Regex::new(rule).unwrap();
        if let Ok(Some(caps)) = re.captures(filename) {
            let episode = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();
            return Some(episode);
        }
    }
    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss() {
        let url = "https://mikanani.me/RSS/Bangumi?bangumiId=3305&subgroupid=382";
        let items = parse_rss(url).unwrap();
        for item in items {
            println!("{:?}", item);
        }
    }
}
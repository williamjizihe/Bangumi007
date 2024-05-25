use std::collections::HashSet;
use std::error::Error;

use fancy_regex::Regex;
use html_escape::decode_html_entities;
use reqwest::blocking::get;
use retry::delay::Fixed;
use retry::retry;
use roxmltree::Document;
use rusqlite::Result;

use cache::rss_mikan::{fetch_cached_items, filter_uncached_items, insert_item_to_cache};

use crate::module::database::cache;
use crate::module::database::cache::rss_mikan::{fetch_mikan_subject_info, insert_subject_to_cache, MikanItem, MikanSubject};
use crate::module::parser::bangumi_parser;
use crate::module::parser::bangumi_parser::parse_season_num_from_aliases;
use crate::module::parser::tmdb_parser::{bangumi_parse_tmdb_info};
use crate::module::utils::error::{new_err, new_warn};

fn parse_filename_to_codec(title: &str) -> String {
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

fn parse_filename_to_language(title: &str) -> String {
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

/// # RSS parser
///
/// ## Input
///
/// RSS feed URL : `&str`
///
/// ## Procedure
///
/// 1. Get the RSS feed from Mikanani
/// 2. Parse episodes not seen in cache database
/// 3. Update cache database
/// 4. Return Episode Items with detailed information
///
/// ## Output
///
/// `Vec` of `MikanItem`s.
///
pub fn update_rss(url: &str) -> Result<Vec<MikanItem>, Box<dyn Error>> {

    // Get RSS feed
    // "get(url).unwrap().text().unwrap();" with retry
    let response = retry(Fixed::from_millis(5000), || {
        match get(url) {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response.text().unwrap())
                } else {
                    log::error!("Failed to get rss, status code is not 200");
                    Err(new_err("Failed to get rss, status code is not 200"))
                }
            }
            Err(_) => {
                log::error!("Failed to get rss");
                Err(new_err("Failed to get rss"))
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
        
        // Exclude the items with \d+-\d in title
        let reg_season = Regex::new(r"\d+-\d").unwrap();
        if reg_season.is_match(&title).unwrap_or(false) {
            log::info!("Skipping item: {}", title);
            continue;
        }

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
            series_name: title.to_string(),
            season_name: title.to_string(),
            season_num: 1,
            episode_num: parse_filename_to_episode(&title).unwrap_or(-1),  // Episode Number
            language: parse_filename_to_language(&title),           // Language
            codec: parse_filename_to_codec(&title),                 // Codec
        });
    }

    // Find the items in database, get the list of items not in database
    // For each item not in database, parse the episode information
    // Then insert the item into the database
    let items_not_in_db = filter_uncached_items(&result);
    for item in items_not_in_db {
        let item = retry(Fixed::from_millis(5000), || {
            match fill_episode_information(&item) {
                Ok(item) => Ok(item),
                Err(_) => {
                    log::warn!("Failed to parse episode info");
                    Err("Failed to parse episode info")
                }
            }
        }).unwrap();
        insert_item_to_cache(&item).unwrap();
    }
    let items_full = fetch_cached_items(&result);
    Ok(items_full)
}

/// # Expand history episodes
///
/// ## Input
///
/// `Vec` of `MikanItem`s (can be of different mikanani subjects)
///
/// ## Procedure
///
/// 1. For each episode item in items, get its bangumi id and subgroup id
/// 2. Parse all the episodes from the RSS feed of the specific bangumi-subgroup page
/// 3. Append the completed episodes to the result
///
/// ## Output
///
/// Expanded `Vec` of `MikanItem`s.
///
pub fn expand_history_episodes(items: Vec<MikanItem>) -> Vec<MikanItem> {
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
        let items_full = update_rss(&url).unwrap();
        // Add to the result
        for item in items_full {
            result.push(item);
        }
        // Add to visited
        visited.insert((item.mikan_subject_id, item.mikan_subgroup_id));
    }
    result
}

/// # Parse Episode
///
/// ## Input
///
/// `MikanItem` with only `mikan_item_uuid` field filled
///
/// ## Procedure
///
/// 1. Get the episode page from Mikanani
/// 2. Parse the episode title, subgroup, magnet link, bangumi id, subgroup id
/// 3. Return the `MikanItem` with all fields filled
///
/// ## Output
///
/// `MikanItem` with all fields filled
///
fn fill_episode_information(item: &MikanItem) -> Result<MikanItem, Box<dyn Error>> {
    // build url from item's uuid
    let url = format!("https://mikanani.me/Home/Episode/{}", item.mikan_item_uuid);

    let response = retry(Fixed::from_millis(5000), || {
        match get(&url) {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response.text().unwrap())
                } else {
                    log::warn!("Failed to get episode info, status code is not 200");
                    Err(new_err("Failed to get episode info, status code is not 200"))
                }
            }
            Err(_) => {
                log::warn!("Failed to get episode info");
                Err(new_err("Failed to get episode info"))
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

    // Parse subject id and subgroup id
    // e.g. find "?bangumiId=" + 3332(bgmid) + "&subgroupId=" + 583(subgid)
    let mikan_subject_id = response.find("?bangumiId=").unwrap();
    let mikan_subject_id = &response[mikan_subject_id + 11..];
    let mikan_subject_id = mikan_subject_id.split('&').next().unwrap();
    let mikan_subject_id = mikan_subject_id.parse::<i32>().unwrap();
    log::debug!("Subject ID: {}", mikan_subject_id);
    let subgid = response.find("&subgroupid=").unwrap();
    let subgid = &response[subgid + 12..];
    let subgid = subgid.split('\"').next().unwrap();
    let subgid = subgid.parse::<i32>().unwrap();
    log::debug!("Subgroup ID: {}", subgid);

    let mikan_subject_info = match fetch_mikan_subject_info(mikan_subject_id) {
        Some(info) => Some(info),       // Use cached info
        None => {
            // 1. Parse Bangumi subject id
            // 2. Parse the season number using all the names fetched by Bangumi API
            // 3. Parse the series name by searching in TMDB API
            // 4. For failed season-num parses, try to find the name in TMDB Subject's Seasons.

            // 1. Parse Bangumi subject id
            let bangumi_subject_id = bangumi_parser::get_bangumi_subject_id(mikan_subject_id).map_or(-1, |x| x);
            log::debug!("Bangumi Subject ID: {}", bangumi_subject_id);

            // 2. Parse the season number using all the names fetched by Bangumi API
            let bangumi_aliases = bangumi_parser::get_bangumi_subject_aliases(bangumi_subject_id);
            let bangumi_season_num = match &bangumi_aliases {
                Ok(aliases) => { parse_season_num_from_aliases(aliases) } // Parse
                Err(_) => None
            };
            let bangumi_subject_name = match &bangumi_aliases {
                Ok(aliases) => { aliases.iter().next().unwrap().clone() }
                Err(_) => "".to_string()
            };

            // 3-4: Parse using TMDB API
            let tmdb_info = bangumi_parse_tmdb_info(bangumi_subject_id)
                .map_err(|e| new_warn(&format!("Failed to parse TMDB info: {}", e)));
            match tmdb_info {
                Ok(tmdb_info) => {
                    let subject = MikanSubject {
                        mikan_subject_id,
                        bangumi_subject_id,
                        bangumi_subject_name,
                        bangumi_season_num: bangumi_season_num.unwrap_or(-1),
                        tmdb_series_id: tmdb_info.media_id as i32,
                        tmdb_series_name: tmdb_info.media_name,
                        tmdb_season_num: tmdb_info.season_number as i32,
                        tmdb_season_name: tmdb_info.season_name,
                    };
                    // Insert the subject info into the database
                    insert_subject_to_cache(&subject).unwrap();
                    Some(subject)
                }
                Err(_) => Some(MikanSubject {
                    mikan_subject_id,
                    bangumi_subject_id,
                    bangumi_subject_name,
                    bangumi_season_num: bangumi_season_num.unwrap_or(-1),
                    tmdb_series_id: -1,
                    tmdb_series_name: "".to_string(),
                    tmdb_season_num: -1,
                    tmdb_season_name: "".to_string(),
                })
            }
        }
    };

    // Using tmdb season num as default. 
    let season_num = match &mikan_subject_info {
        Some(info) => match info.tmdb_season_num {
            -1 => match info.bangumi_season_num {
                -1 => 1,
                _ => info.bangumi_season_num
            },
            _ => info.tmdb_season_num
        },
        None => 1
    };

    let series_name = match &mikan_subject_info {
        Some(info) => {
            // if the series name is empty, use mikan's subject title
            if !info.tmdb_series_name.is_empty() { info.tmdb_series_name.clone() } else { title.to_string() }
        }
        None => title.to_string()
    };

    let season_name = match &mikan_subject_info {
        Some(info) => {
            // if the season name is empty, use mikan's subject title
            if !info.tmdb_season_name.is_empty() { info.tmdb_series_name.clone() } else { title.to_string() }
        }
        None => title.to_string()
    };

    Ok(MikanItem {
        mikan_item_uuid: item.mikan_item_uuid.to_string(),
        mikan_subject_id,
        mikan_subgroup_id: subgid,
        mikan_subject_title: title.to_string(),
        mikan_item_title: item.mikan_item_title.to_string(),
        mikan_item_magnet_link: magnet.to_string(),
        mikan_item_pub_date: item.mikan_item_pub_date.to_string(),
        series_name,
        season_name,
        season_num,
        episode_num: item.episode_num,
        language: item.language.to_string(),
        codec: item.codec.to_string(),
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss() {
        let url = "https://mikanani.me/RSS/Bangumi?bangumiId=3305&subgroupid=382";
        let items = update_rss(url).unwrap();
        for item in items {
            println!("{:?}", item);
        }
    }
}
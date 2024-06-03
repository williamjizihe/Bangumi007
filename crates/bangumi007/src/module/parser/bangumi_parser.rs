use std::error::Error;

use fancy_regex::Regex;
use retry::delay::Fixed;

use crate::module::database::cache::rss::BangumiEpisode;
use crate::module::utils::error::{new_err, new_warn};

pub struct BangumiSubject {
    pub bangumi_subject_id: i32,
    pub image_url: String,
    pub aliases: Vec<String>,
    pub media_type: String,
    pub season_num: i32,
}

pub fn get_bangumi_subject(bangumi_subject_id: i32) -> rusqlite::Result<BangumiSubject, Box<dyn Error>> {
    let json = get_bangumi_subject_json(bangumi_subject_id)?;
    let image_url = json.get("images")
        .ok_or_else(|| new_warn("Failed to get image url"))
        .and_then(|x| x.get("large").ok_or_else(|| new_warn("Failed to get image url")))
        .and_then(|x| x.as_str().ok_or_else(|| new_warn("Failed to get image url as str")))
        .map(|x| x.to_string())?;
    let aliases = get_bangumi_subject_aliases(&json)?;
    let media_type = get_bangumi_media_type(&json)?;
    let season_num = parse_season_num_from_aliases(&aliases).unwrap_or(-1);

    Ok(BangumiSubject {
        bangumi_subject_id,
        image_url,
        aliases,
        media_type,
        season_num,
    })
}

fn get_bangumi_media_type(json: &serde_json::Value) -> Result<String, Box<dyn Error>> {
    let media_type = json.get("platform")
        .ok_or_else(|| new_warn("Failed to get media type"))
        .and_then(|x| x.as_str().ok_or_else(|| new_warn("Failed to get media type as str")));
    let media_type = match media_type {
        Ok(media_type) => media_type,
        Err(_) => return Ok("".to_string()),
    };
    Ok(media_type.to_string())
}

fn get_bangumi_subject_json(bangumi_subject_id: i32) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let url = format!("https://api.bgm.tv/v0/subjects/{}", bangumi_subject_id);
    // add user-agent
    let client = reqwest::blocking::Client::builder()
        .user_agent("MapleWithered/Bangumi007 (https://github.com/MapleWithered/Bangumi007)")
        .build()
        .unwrap();

    let response = retry::retry(Fixed::from_millis(5000), || {
        match client.get(&url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response.text().unwrap())
                } else {
                    Err(new_warn(format!("Failed to get episode info, status code is not 200: {}",
                                         response.status()).as_str()))
                }
            }
            Err(_) => Err(new_warn("Failed to get episode info"))
        }
    }).map_err(|_| new_err("Failed to get episode info"))?;

    // Parse json result
    let json: serde_json::Value = serde_json::from_str(&response)
        .map_err(|_| new_err("Failed to parse json"))?;

    Ok(json)
}


/// Get the Bangumi subject aliases of the Bangumi subject
///
/// ## Input
///
/// Bangumi subject id : `i32`
///
/// ## Procedure
///
/// e.g. https://bgm.tv/subject/444557
///
/// 1. Get https://api.bgm.tv/v0/subjects/444557
/// 2. Parse the JSON and get the aliases
///
/// ## Output
///
/// Bangumi subject aliases : `Vec of String`
pub fn get_bangumi_subject_aliases(json: &serde_json::Value) -> Result<Vec<String>, Box<dyn Error>> {
    let mut vec_aliases: Vec<String> = Vec::new();

    // Parse default name and name_cn
    let default_name = json.get("name")
        .ok_or_else(|| new_warn("Failed to get name"));
    let default_name_cn = json.get("name_cn")
        .ok_or_else(|| new_warn("Failed to get name_cn"));
    // if success, append to vec_aliases
    if default_name_cn.is_ok() {
        let default_name_cn = default_name_cn.unwrap().as_str();
        if default_name_cn.is_some() {
            let default_name_cn = default_name_cn.unwrap();
            if default_name_cn != "" {
                vec_aliases.push(default_name_cn.to_string());
            }
        }
    };
    if default_name.is_ok() {
        let default_name = default_name.unwrap().as_str();
        if default_name.is_some() {
            let default_name = default_name.unwrap();
            if default_name != "" {
                vec_aliases.push(default_name.to_string());
            }
        }
    }

    // Parse infobox to get other aliases
    let infobox = json.get("infobox")
        .ok_or_else(|| new_warn("Failed to get infobox"))
        .and_then(|x| x.as_array().ok_or_else(|| new_warn("Failed to get infobox as array")));
    let infobox = match infobox {
        Ok(infobox) => infobox,
        Err(_) => return Ok(vec_aliases),
    };

    for item in infobox {
        let item = item.as_object().ok_or_else(|| new_warn("Failed to get item as object"));
        let item = match item {
            Ok(item) => item,
            Err(_) => continue,
        };
        let item_key = item.get("key")
            .ok_or_else(|| new_warn("Failed to get item key"))
            .and_then(|x| x.as_str().ok_or_else(|| new_warn("Failed to get item key as str")));
        let item_key = match item_key {
            Ok(item_key) => item_key,
            Err(_) => continue,
        };

        if item_key != "别名" {
            continue;
        }

        // Parse alises
        let item_value = item.get("value")
            .ok_or_else(|| new_warn("Failed to get item value"))
            .and_then(|x| x.as_array().ok_or_else(|| new_warn("Failed to get item value as str")));
        let item_value = match item_value {
            Ok(item_value) => item_value,
            Err(_) => continue,
        };

        for alias in item_value {
            let alias = alias.as_object().ok_or_else(|| new_warn("Failed to get alias as str"));
            let alias = match alias {
                Ok(alias) => alias,
                Err(_) => continue,
            };
            let alias_name = alias.get("v")
                .ok_or_else(|| new_warn("Failed to get alias name"))
                .and_then(|x| x.as_str().ok_or_else(|| new_warn("Failed to get alias name as str")));
            let alias_name = match alias_name {
                Ok(alias_name) => alias_name,
                Err(_) => continue,
            };
            vec_aliases.push(alias_name.to_string());
        }
    }

    Ok(vec_aliases)
}


pub fn parse_season_num_from_aliases(aliases: &Vec<String>) -> Option<i32> {
    // Parse the season number from the aliases
    // "第二季" -> 2
    // "第二季 第一部分" -> 2
    // "第02季" -> 2
    // "第2季 第1部分" -> 2
    // "Season 2" -> 2
    // "Season 2: Part 1" -> 2
    // "S02" -> 2
    // "S02: Part 1" -> 2
    // "第2期" -> 2

    // Change all big letters to small letters
    let aliases: Vec<String> = aliases.iter().map(|x| x.to_lowercase()).collect();

    // Regex rules
    let rules = [
        r"season (\d+)",
        r"season(\d+)",
        r"第(\d+)季",
        r"第(\d+)期",
    ];
    for rule in rules.iter() {
        let re = Regex::new(rule).unwrap();
        for alias in aliases.iter() {
            if let Ok(Some(caps)) = re.captures(alias) {
                let season = caps.get(1).unwrap().as_str();
                let season = season.parse::<i32>();
                match season {
                    Ok(season) => return Some(season),
                    Err(_) => continue,
                }
            }
        }
    }

    // Parse Chinese numbers
    let rules = [
        r"第一季", r"第二季", r"第三季", r"第四季", r"第五季",
        r"第六季", r"第七季", r"第八季", r"第九季", r"第十季",
        r"第十一季", r"第十二季", r"第十三季", r"第十四季", r"第十五季",
        r"第十六季", r"第十七季", r"第十八季", r"第十九季", r"第二十季",
    ];

    for (i, rule) in rules.iter().enumerate() {
        if aliases.iter().any(|x| x.contains(rule)) {
            return Some((i + 1) as i32);
        }
    }

    None
}


pub fn get_bangumi_episodes(bangumi_subject_id: i32) -> Result<Vec<BangumiEpisode>, Box<dyn Error>> {
    let cache_result = crate::module::database::cache::rss::get_bangumi_episodes(bangumi_subject_id).unwrap_or_else(|_| Vec::new());
    if !cache_result.is_empty() {
        return Ok(cache_result);
    }
    
    let url = format!("https://api.bgm.tv/v0/episodes?subject_id={}", bangumi_subject_id);
    // add user-agent
    let client = reqwest::blocking::Client::builder()
        .user_agent("MapleWithered/Bangumi007 (https://github.com/MapleWithered/Bangumi007)")
        .build()
        .unwrap();

    let response = retry::retry(Fixed::from_millis(5000), || {
        match client.get(&url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response.text().unwrap())
                } else {
                    Err(new_warn(format!("Failed to get bangumi episodes, status code is not 200: {}",
                                         response.status()).as_str()))
                }
            }
            Err(_) => Err(new_warn("Failed to get bangumi episodes"))
        }
    }).map_err(|_| new_err("Failed to get bangumi episodes"))?;

    // Parse json result
    let json: serde_json::Value = serde_json::from_str(&response)
        .map_err(|_| new_err("Failed to parse json"))?;

    let mut vec_episodes: Vec<BangumiEpisode> = Vec::new();
    let json = json.as_object().ok_or_else(|| new_warn("Failed to get json as object"))?;
    let json = json.get("data").ok_or_else(|| new_warn("Failed to get bangumi episodes"))?;
    let episodes = json.as_array().ok_or_else(|| new_warn("Failed to get bangumi episodes"))?;

    for episode in episodes {
        let episode = episode
            .as_object()
            .ok_or_else(|| new_warn("Failed to get episode as object"));
        let episode = match episode {
            Ok(episode) => episode,
            Err(_) => continue,
        };

        let episode_id = episode
            .get("id")
            .ok_or_else(|| new_warn("Failed to get episode id"))
            .and_then(|x| x.as_i64().ok_or_else(|| new_warn("Failed to get episode id as i64")))
            .and_then(|x| x.try_into().map_err(|_| new_warn("Failed to convert episode id to i32")));
        let episode_id = match episode_id {
            Ok(episode_id) => episode_id,
            Err(_) => continue,
        };

        let episode_type = episode
            .get("type")
            .ok_or_else(|| new_warn("Failed to get episode type"))
            .and_then(|x| x.as_i64().ok_or_else(|| new_warn("Failed to get episode type as i64")))
            .and_then(|x| x.try_into().map_err(|_| new_warn("Failed to convert episode type to i32")));
        let episode_type = match episode_type {
            Ok(episode_type) => episode_type,
            Err(_) => continue,
        };

        let episode_ep = episode
            .get("ep")
            .ok_or_else(|| new_warn("Failed to get episode ep"))
            .and_then(|x| x.as_i64().ok_or_else(|| new_warn("Failed to get episode ep as i64")))
            .and_then(|x| x.try_into().map_err(|_| new_warn("Failed to convert episode ep to i32")));
        let episode_ep = match episode_ep {
            Ok(episode_ep) => episode_ep,
            Err(_) => continue,
        };

        let episode_sort = episode
            .get("sort")
            .ok_or_else(|| new_warn("Failed to get episode sort"))
            .and_then(|x| x.as_str().ok_or_else(|| new_warn("Failed to get episode sort as str")))
            .and_then(|x| Ok(x.to_string()));
        let episode_sort = match episode_sort {
            Ok(episode_sort) => episode_sort,
            Err(_) => continue,
        };

        let episode_name = episode
            .get("name")
            .ok_or_else(|| new_warn("Failed to get episode name"))
            .and_then(|x| x.as_str().ok_or_else(|| new_warn("Failed to get episode name as str")))
            .map(|x| x.to_string());
        let episode_name = match episode_name {
            Ok(episode_name) => episode_name,
            Err(_) => continue,
        };

        let episode_name_cn = episode
            .get("name_cn")
            .ok_or_else(|| new_warn("Failed to get episode name_cn"))
            .and_then(|x| x.as_str().ok_or_else(|| new_warn("Failed to get episode name_cn as str")))
            .map(|x| x.to_string());
        let episode_name_cn = match episode_name_cn {
            Ok(episode_name_cn) => episode_name_cn,
            Err(_) => continue,
        };

        let episode_airdate = episode
            .get("airdate")
            .ok_or_else(|| new_warn("Failed to get episode airdate"))
            .and_then(|x| x.as_str().ok_or_else(|| new_warn("Failed to get episode airdate as str")))
            .map(|x| x.to_string());
        let episode_airdate = match episode_airdate {
            Ok(episode_airdate) => episode_airdate,
            Err(_) => continue,
        };

        let bangumi_episode = BangumiEpisode {
            subject_id: bangumi_subject_id,
            episode_id,
            episode_type,
            episode_ep,    // raw index of episode
            episode_sort,  // display index of episode
            episode_name,
            episode_name_cn,
            episode_airdate,
        };
        vec_episodes.push(bangumi_episode);
    }
    
    // Insert into cache
    for episode in &vec_episodes {
        crate::module::database::cache::rss::insert_bangumi_episode_to_cache(episode).unwrap();
    }

    Ok(vec_episodes)
}

pub fn parse_bangumi_episode(bangumi_subject_id: i32, mikan_episode_num: i32, episode_type: i32) -> Result<BangumiEpisode, Box<dyn Error>> {
    // let episodes = get_bangumi_episodes(bangumi_subject_id)?;
    // let episode = episodes.iter().find(|x| x.episode_sort == mikan_episode_num && x.episode_type == episode_type);
    // if episode.is_some() {
    //     return Ok(episode.unwrap().clone());
    // }
    // let episode = episodes.iter().find(|x| x.episode_ep == mikan_episode_num && x.episode_type == episode_type);
    // if episode.is_some() {
    //     return Ok(episode.unwrap().clone());
    // }
    Err(new_err("Failed to find episode"))
}

#[cfg(test)]
mod tests {
    use crate::module::logger::logger;
    use crate::module::parser::bangumi_parser::{get_bangumi_episodes, parse_bangumi_episode};

    #[test]
    fn test_get_bangumi_subject_aliases() {
        logger::init();
        // let aliases = get_bangumi_subject_aliases(303864).unwrap();
        // for alias in aliases {
        //     println!("{}", alias);
        // }
    }

    #[test]
    fn test_parse_season_num_from_aliases() {
        logger::init();
        // let ids = vec![444557, 405785, 262897, 303864, 208908];
        // for id in ids {
        //     let aliases = get_bangumi_subject_aliases(id).unwrap();
        //     print!("Bangumi: {}, ", aliases[0]);
        //     let season = parse_season_num_from_aliases(&aliases).unwrap_or(-1);
        //     println!("Season: {}", season);
        // }
    }

    #[test]
    fn test_get_bangumi_episodes() {
        logger::init();
        let res = get_bangumi_episodes(425978);
        println!("{:?}", res);
    }
    
    #[test]
    fn test_parse_bangumi_episode() {
        logger::init();
        let res = parse_bangumi_episode(425978, 8, 0);
        println!("{:?}", res);
    }
}
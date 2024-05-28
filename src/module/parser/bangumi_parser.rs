use std::error::Error;

use fancy_regex::Regex;
use retry::delay::Fixed;

use crate::module::utils::error::{new_err, new_warn};

pub struct BangumiSubject {
    pub bangumi_subject_id: i32,
    pub aliases: Vec<String>,
    pub media_type: String,
    pub season_num: i32,
}

pub fn get_bangumi_subject(bangumi_subject_id: i32) -> rusqlite::Result<BangumiSubject, Box<dyn Error>> {
    let json = get_bangumi_subject_json(bangumi_subject_id)?;
    let aliases = get_bangumi_subject_aliases(&json)?;
    let media_type = get_bangumi_media_type(&json)?;
    let season_num = parse_season_num_from_aliases(&aliases).unwrap_or(-1);

    Ok(BangumiSubject {
        bangumi_subject_id,
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

fn get_bangumi_subject_json (bangumi_subject_id: i32) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
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

#[cfg(test)]
mod tests {
    use crate::module::logger::logger;
    use super::*;

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
}
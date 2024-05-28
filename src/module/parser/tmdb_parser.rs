use std::collections::HashMap;
use std::error::Error;
use log::trace;

use retry::delay::Fixed;

use crate::module::config::CONFIG;
use crate::module::parser::bangumi_parser::{get_bangumi_subject, get_bangumi_subject_aliases};
use crate::module::utils::error::{new_err, new_warn};


pub fn tmdb_search_tv(series_name: &str) -> rusqlite::Result<i64, Box<dyn Error>> {

    // curl --request GET \
    //      --url 'https://api.themoviedb.org/3/search/multi?query={name}&include_adult=false&language=en-US&page=1' \
    //      --header 'Authorization: Bearer {Access Token Auth}' \
    //      --header 'accept: application/json'

    let api_access_token_auth = CONFIG.read().unwrap().parser_config.tmdb_config.api_access_token_auth.clone();
    let include_adult = CONFIG.read().unwrap().parser_config.tmdb_config.include_adult;

    let url = format!("https://api.themoviedb.org/3/search/tv?query={}&include_adult={}&language=zh-CN", series_name, include_adult);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {}", api_access_token_auth).parse().unwrap());
    headers.insert("accept", "application/json".parse().unwrap());

    let response = retry::retry(Fixed::from_millis(5000), || {
        match reqwest::blocking::Client::new().get(&url).headers(headers.clone()).send() {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response.text().unwrap())
                } else {
                    Err(new_warn(format!("Failed to get tmdb media info, status code is {}", response.status()).as_str()))
                }
            }
            Err(_) => Err(new_warn("Failed to get tmdb media info"))
        }
    }).map_err(|_| new_err("Failed to get tmdb media info"))?;

    // Parse json
    let json: serde_json::Value = serde_json::from_str(&response)
        .map_err(|_| new_err("Failed to parse json"))?;

    // Get json['results'][0]['id'] and json['results'][0]['media_type']
    let first_result = json.get("results")
        .ok_or_else(|| new_err("Failed to get results"))?
        .as_array()
        .ok_or_else(|| new_err("Failed to get results as array"))?
        .get(0)
        .ok_or_else(|| new_warn(format!("TMDB Search result empty for {}", series_name).as_str()))?;

    let media_id = first_result.get("id")
        .ok_or_else(|| new_err("Failed to get id"))?
        .as_i64()
        .ok_or_else(|| new_err("Failed to get id as i64"))?;

    Ok(media_id)
}

pub fn tmdb_search_multi(series_name: &str) -> rusqlite::Result<(String, i64), Box<dyn Error>> {

    // curl --request GET \
    //      --url 'https://api.themoviedb.org/3/search/multi?query={name}&include_adult=false&language=en-US&page=1' \
    //      --header 'Authorization: Bearer {Access Token Auth}' \
    //      --header 'accept: application/json'

    let api_access_token_auth = CONFIG.read().unwrap().parser_config.tmdb_config.api_access_token_auth.clone();
    let include_adult = CONFIG.read().unwrap().parser_config.tmdb_config.include_adult;

    let url = format!("https://api.themoviedb.org/3/search/multi?query={}&include_adult={}&language=zh-CN", series_name, include_adult);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {}", api_access_token_auth).parse().unwrap());
    headers.insert("accept", "application/json".parse().unwrap());

    let response = retry::retry(Fixed::from_millis(5000), || {
        match reqwest::blocking::Client::new().get(&url).headers(headers.clone()).send() {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response.text().unwrap())
                } else {
                    Err(new_warn("Failed to get tmdb media info, status code is not 200"))
                }
            }
            Err(_) => Err(new_warn("Failed to get tmdb media info"))
        }
    }).map_err(|_| new_err("Failed to get tmdb media info"))?;

    // Parse json
    let json: serde_json::Value = serde_json::from_str(&response)
        .map_err(|_| new_err("Failed to parse json"))?;

    // Get json['results'][0]['id'] and json['results'][0]['media_type']
    let first_result = json.get("results")
        .ok_or_else(|| new_err("Failed to get results"))?
        .as_array()
        .ok_or_else(|| new_err("Failed to get results as array"))?
        .get(0)
        .ok_or_else(|| new_warn(format!("TMDB Search result empty for {}", series_name).as_str()))?;

    let media_id = first_result.get("id")
        .ok_or_else(|| new_err("Failed to get id"))?
        .as_i64()
        .ok_or_else(|| new_err("Failed to get id as i64"))?;

    let media_type = first_result.get("media_type")
        .ok_or_else(|| new_err("Failed to get media_type"))?
        .as_str()
        .ok_or_else(|| new_err("Failed to get media_type as str"))?;

    Ok((media_type.to_string(), media_id))
}

fn tmdb_get_media_info_internal(media_type: &str, media_id: i64, lang: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    // curl --request GET \
    //      --url 'https://api.themoviedb.org/3/{media_type}/{media_id}?language=zh-CN' \
    //      --header 'Authorization: Bearer {Access Token Auth}' \
    //      --header 'accept: application/json'

    let api_access_token_auth = CONFIG.read().unwrap().parser_config.tmdb_config.api_access_token_auth.clone();


    let url = format!("https://api.themoviedb.org/3/{}/{}?language={}", media_type, media_id, lang);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {}", api_access_token_auth).parse().unwrap());
    headers.insert("accept", "application/json".parse().unwrap());

    let response = retry::retry(Fixed::from_millis(5000), || {
        match reqwest::blocking::Client::new().get(&url).headers(headers.clone()).send() {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response.text().unwrap())
                } else {
                    Err(new_warn("Failed to get tmdb media info, status code is not 200"))
                }
            }
            Err(_) => Err(new_warn("Failed to get tmdb media info"))
        }
    }).map_err(|_| new_err("Failed to get tmdb media info"))?;

    // Parse json
    let json: serde_json::Value = serde_json::from_str(&response)
        .map_err(|_| new_err("Failed to parse json"))?;

    Ok(json)
}

pub fn tmdb_get_media_info(media_type: &str, media_id: i64) -> Result<HashMap<String, serde_json::Value>, Box<dyn Error>> {
    let languages = vec!["ja", "zh-CN", "en-US"];
    let mut result_dict = HashMap::new();

    for lang in languages {
        let result = tmdb_get_media_info_internal(media_type, media_id, lang);
        if result.is_ok() {
            result_dict.insert(lang.to_string(), result.unwrap());
        }
    }

    Ok(result_dict)
}

pub fn tmdb_parse_media_name(json: &serde_json::Value) -> Result<String, Box<dyn Error>> {

    // Get json['name']
    let media_name = json.get("name")
        .ok_or_else(|| new_err("Failed to get name"))?
        .as_str()
        .ok_or_else(|| new_err("Failed to get name as str"))?;

    Ok(media_name.to_string())
}

pub fn tmdb_search_season_in_infos(lang_json: &HashMap<String, serde_json::Value>, aliases: &Vec<String>) -> Result<(i64, String), Box<dyn Error>> {
    let mut season_num_to_names: HashMap<i64, HashMap<String, String>> = HashMap::new();

    for (lang, lang_info) in lang_json {
        let seasons = lang_info.get("seasons")
            .ok_or_else(|| new_err("Failed to get seasons"))?
            .as_array()
            .ok_or_else(|| new_err("Failed to get seasons as array"))?;

        for season in seasons {
            let season_name = season.get("name")
                .ok_or_else(|| new_err("Failed to get season name"))?
                .as_str()
                .ok_or_else(|| new_err("Failed to get season name as str"))?;

            let season_number = season.get("season_number")
                .ok_or_else(|| new_err("Failed to get season number"))?
                .as_i64()
                .ok_or_else(|| new_err("Failed to get season number as i64"))?;

            if !season_num_to_names.contains_key(&season_number) {
                season_num_to_names.insert(season_number, HashMap::new());
            }

            season_num_to_names.get_mut(&season_number).unwrap().insert(lang.to_string(), season_name.to_string());
        }
    }

    log::trace!("SeasonNumToNames: {:?}", season_num_to_names);


    // For each season, match the name with all aliases and get the longest match as match
    // Choose the longest match as the season
    let mut max_match = -1;
    let mut res_season_number = 0;
    let mut res_season_name: String = "".to_string();
    for (season_number, season_name_dict) in &season_num_to_names {
        log::trace!("SeasonNumber: {}, SeasonNameDict: {:?}", season_number, season_name_dict);
        for alias in aliases {
            log::trace!("Alias: {}", alias);
            // Replace chinese number with arabic number in alias (第一季 -> 第1季)
            let alias = alias
                .replace("十一", "11")
                .replace("十二", "12")
                .replace("十三", "13")
                .replace("十四", "14")
                .replace("十五", "15")
                .replace("十六", "16")
                .replace("十七", "17")
                .replace("十八", "18")
                .replace("十九", "19")
                .replace("十", "10")
                .replace("一", "1")
                .replace("二", "2")
                .replace("三", "3")
                .replace("四", "4")
                .replace("五", "5")
                .replace("六", "6")
                .replace("七", "7")
                .replace("八", "8")
                .replace("九", "9");
            let zh_cn_season_name = season_name_dict.get("zh-CN").unwrap().clone();
            for (_, season_name) in season_name_dict {
                // Replace 第 1 季 as 第1季
                let season_name = season_name.replace("第 ", "第").replace(" 季", "季");
                // Replace シーズン3 or シーズン 3 as 第3シーズン
                let reg_season = regex::Regex::new(r"シーズン ?(\d+)").unwrap();
                let season_name = reg_season.replace_all(&season_name, "第$1シーズン").to_string();
                let alias = reg_season.replace_all(&alias, "第$1シーズン").to_string();
                // alias与season_name的最长公共子串
                let mut match_len = 0;
                let mut i = 0;
                let mut j = 0;
                while i < alias.chars().count() && j < season_name.chars().count() {
                    if alias.chars().nth(i).unwrap() == season_name.chars().nth(j).unwrap() {
                        match_len += 1;
                        i += 1;
                        j += 1;
                    } else {
                        i += 1;
                    }
                }
                log::trace!("Alias: {}, SeasonName: {}, MatchLen: {}", alias, season_name, match_len);
                if match_len > max_match {
                    max_match = match_len;
                    res_season_number = *season_number;
                    res_season_name = zh_cn_season_name.clone();
                    log::debug!("update match! season_number: {}, season_name: {}, alias: {}, match_len: {}", season_number, season_name, alias, match_len);
                }
            }
        }
    }
    if max_match == 0 {
        res_season_number = 1;
        let try_operations = || -> Result<String, Box<dyn Error>> {
            let season_name = season_num_to_names.get(&1).unwrap().get("zh-CN").unwrap().clone();
            Ok(season_name)
        };
        res_season_name = try_operations().unwrap_or("".to_string());
        if res_season_name.is_empty() {
            let try_operations = || -> Result<String, Box<dyn Error>> {
                let season_name = lang_json.get("zh-CN").unwrap().get("name").unwrap().as_str().unwrap().to_string();
                Ok(season_name)
            };
            res_season_name = try_operations().unwrap_or("".to_string());
        }
        log::debug!("No match, use the first season: {}", res_season_name)
    }

    log::debug!("Best match length: {}, SeasonNumber: {}, SeasonName: {}", max_match, res_season_number, res_season_name);

    Ok((res_season_number, res_season_name))
}

#[derive(Debug)]
pub struct TMDBParseResult {
    pub bangumi_subject_id: i32,
    pub media_id: i64,
    pub media_name: String,
    pub season_number: i64,
    pub season_name: String,
}

pub fn bangumi_parse_tmdb_info(bangumi_subject_id: i32) -> Result<TMDBParseResult, Box<dyn Error>> {
    let bangumi_info = get_bangumi_subject(bangumi_subject_id)?;
    let aliases = bangumi_info.aliases;
    if bangumi_info.media_type == "剧场版" || bangumi_info.media_type == "OVA" {
        return Err(new_err(format!("Media type is {}. Not implemented.", bangumi_info.media_type).as_str()));
    }
    // For each alias, search in tmdb
    // If found, get media_name and season_number
    // If Err, continue to next alias
    let mut alias_id = 0;
    let search_result = loop {
        let alias = &aliases[alias_id];
        let result = tmdb_search_tv(alias);
        if result.is_ok() {
            break result;
        }
        alias_id += 1;
        if alias_id >= aliases.len() {
            break Err(new_warn("Failed to search media in tmdb"));
        }
    };
    let search_result = match search_result {
        Ok(search_result) => Ok(search_result),
        Err(_) => {
            let mut alias_id = 0;
            let search_result = loop {
                // only take the first half of the alias
                let char_count = aliases[alias_id].chars().count();
                // let alias = &aliases[alias_id][..char_count / 2];
                let alias = &aliases[alias_id].chars().take(char_count / 2).collect::<String>();
                let result = tmdb_search_tv(alias);
                if result.is_ok() {
                    break result;
                }
                alias_id += 1;
                if alias_id >= aliases.len() {
                    break Err(new_warn("Failed to search media in tmdb"));
                }
            };
            search_result
        }
    };
    let media_id = search_result?;
    let media_infos = tmdb_get_media_info("tv", media_id)?;
    let media_name = tmdb_parse_media_name(&media_infos["zh-CN"])?;
    let (season_number, season_name) = tmdb_search_season_in_infos(&media_infos, &aliases)?;

    println!("BangumiSubject: {}, TMDBSeries: {}, TMDBSeason: {}, SeasonNumber: {}", aliases[0], media_name, season_name, season_number);

    Ok(TMDBParseResult {
        bangumi_subject_id,
        media_name,
        media_id,
        season_name,
        season_number,
    })
}

#[cfg(test)]
mod tests {
    use log::debug;
    use crate::module::logger;
    use crate::module::parser::bangumi_parser::get_bangumi_subject_aliases;

    use super::*;

    #[test]
    fn test_tmdb_search_media() {
        logger::init();
        let series_name = "異世界魔王と召喚少女の奴隷魔術Ω";
        let result = tmdb_search_tv(series_name);
        assert!(result.is_ok());
        println!("{:?}", result.unwrap());
    }

    #[test]
    fn test_tmdb_get_media_info() {
        logger::init();
        let media_type = "tv";
        let media_id = 84958;
        let result = tmdb_get_media_info(media_type, media_id);
        assert!(result.is_ok());
        println!("{:?}", result.unwrap());
    }

    #[test]
    fn test_tmdb() {
        logger::init();
        let bangumi_ids = vec![283643, ];     //285666, 303864, 444557, 208908, 455345, 425978, 455835, 416777, 350235, 364522, 342667
        for bangumi_id in bangumi_ids {
            let bangumi_subject = get_bangumi_subject(bangumi_id).unwrap();
            let aliases = bangumi_subject.aliases;
            // For each alias, search in tmdb
            // If found, get media_name and season_number
            // If Err, continue to next alias
            let mut alias_id = 0;
            let search_result = loop {
                let alias = &aliases[alias_id];
                let result = tmdb_search_tv(alias);
                if result.is_ok() {
                    break result;
                }
                alias_id += 1;
                if alias_id >= aliases.len() {
                    break Err(new_warn("Failed to search media in tmdb"));
                }
            };
            let search_result = match search_result {
                Ok(search_result) => Ok(search_result),
                Err(_) => {
                    let mut alias_id = 0;
                    let search_result = loop {
                        // only take the first half of the alias
                        let char_count = aliases[alias_id].chars().count();
                        // let alias = &aliases[alias_id][..char_count / 2];
                        let alias = &aliases[alias_id].chars().take(char_count / 2).collect::<String>();
                        let result = tmdb_search_tv(alias);
                        if result.is_ok() {
                            break result;
                        }
                        alias_id += 1;
                        if alias_id >= aliases.len() {
                            break Err(new_warn("Failed to search media in tmdb"));
                        }
                    };
                    search_result
                }
            };
            let media_id = search_result.unwrap();
            let media_infos = tmdb_get_media_info("tv", media_id).unwrap();
            let media_name = tmdb_parse_media_name(&media_infos["zh-CN"]).unwrap();
            let (season_number, season_name) = tmdb_search_season_in_infos(&media_infos, &aliases).unwrap();

            debug!("BangumiSubject: {}, TMDBSeries: {}, TMDBSeason: {}, SeasonNumber: {}", aliases[0], media_name, season_name, season_number);
        }
    }
}
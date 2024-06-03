use std::error::Error;

use serde_json::json;
use eframe::egui::Color32;

use crate::module::config::CONFIG;
use crate::module::parser::bangumi_parser::get_bangumi_episodes;
use crate::module::utils::error::new_err;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum BangumiEpisodeStatus {
    #[default]
    NotCollected,
    WantToWatch,
    Watched,
    Dropped,
}

impl BangumiEpisodeStatus {
    pub(crate) fn get_fill_color(&self, date: String) -> impl Into<Color32> + Sized {
        match self {
            BangumiEpisodeStatus::NotCollected => Color32::from_rgb(170, 197, 230),
            BangumiEpisodeStatus::WantToWatch => Color32::from_rgb(255, 173, 209),
            BangumiEpisodeStatus::Watched => Color32::from_rgb(72, 151, 255),
            BangumiEpisodeStatus::Dropped => Color32::from_rgb(204, 204, 204),
        }
    }

    pub(crate) fn get_text_color(&self, date: String) -> impl Into<Color32> + Sized {
        match self {
            BangumiEpisodeStatus::NotCollected => Color32::from_rgb(0, 102, 204),
            BangumiEpisodeStatus::WantToWatch => Color32::from_rgb(163, 73, 164),
            BangumiEpisodeStatus::Watched => Color32::from_rgb(255, 255, 255),
            BangumiEpisodeStatus::Dropped => Color32::from_rgb(154, 144, 144),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum BangumiEpisodeType {
    #[default]
    MainStory,
    Special,
    OP,
    ED,
    Other,
}

#[derive(Debug)]
pub struct BangumiEpisodeCollection {
    pub id: i32,
    pub sort: String,
    pub ep: i32,
    pub airdate: String,
    pub name: String,
    pub name_cn: String,
    pub ep_type: BangumiEpisodeType,
    pub status: BangumiEpisodeStatus,
}

pub fn get_bangumi_episode_collection_status(bangumi_subject_id: i32) -> Result<Vec<BangumiEpisodeCollection>, Box<dyn Error>> {
    // curl -X 'GET' \
    //   'https://api.bgm.tv/v0/users/-/collections/{bangumi_subject_id}/episodes' \
    //   -H 'accept: application/json' \
    //   -H 'Authorization: Bearer {access_token}'

    let access_token = CONFIG.read().unwrap().scrobbler_config.bangumi_access_token.clone();
    let url = format!("https://api.bgm.tv/v0/users/-/collections/{}/episodes", bangumi_subject_id);
    let client = reqwest::blocking::Client::new();
    let res = retry::retry(retry::delay::Fibonacci::from_millis(1000).take(5), || {
        client.get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("accept", "application/json")
            // UA is forced
            .header("User-Agent", "MapleWithered/Bangumi007 (https://github.com/MapleWithered/Bangumi007)")
            .send()
            .map_err(|e| new_err(&format!("Failed to send request: {}", e)))
    }).map_err(|e| new_err(&format!("Failed to send request: {}", e)))?;

    if !res.status().is_success() {
        return Err(new_err(&format!("Failed to get bangumi episode collection status: {}", res.text()?)));
    }

    // parse json as object
    let json: serde_json::Value = res
        .json()
        .map_err(|e| new_err(&format!("Failed to parse json: {}", e)))?;
    let mut result = Vec::new();

    let data = json.as_object()
        .and_then(|o| o.get("data"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| new_err("Failed to parse json"))?;

    for item in data {
        let item = item.as_object().ok_or_else(|| new_err("Failed to parse json"))?;
        let episode = item.get("episode")
            .and_then(|e| e.as_object())
            .ok_or_else(|| new_err("Failed to parse episode"))?;
        let status = item.get("type")
            .and_then(|t| t.as_i64())
            .ok_or_else(|| new_err("Failed to parse type"))?;

        result.push(BangumiEpisodeCollection {
            id: episode.get("id")
                .and_then(|i| i.as_i64())
                .ok_or_else(|| new_err("Failed to parse id"))? as i32,
            sort: episode.get("sort")
                .and_then(|s| s.as_f64())
                .and_then(|s| Some(s.to_string()))
                .ok_or_else(|| new_err(format!("Failed to parse sort: {:?}", episode).as_str()))?,
            ep: episode.get("ep")
                .and_then(|e| e.as_i64())
                .ok_or_else(|| new_err("Failed to parse ep"))? as i32,
            airdate: episode.get("airdate")
                .and_then(|a| a.as_str())
                .unwrap_or("").to_string(),
            name: episode.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("").to_string(),
            name_cn: episode.get("name_cn")
                .and_then(|n| n.as_str())
                .unwrap_or("").to_string(),
            ep_type: match episode.get("type")
                .and_then(|t| t.as_i64())
                .ok_or_else(|| new_err("Failed to parse type"))? as i32 {
                0 => BangumiEpisodeType::MainStory,
                1 => BangumiEpisodeType::Special,
                2 => BangumiEpisodeType::OP,
                3 => BangumiEpisodeType::ED,
                _ => BangumiEpisodeType::Other,
            },
            status: match status {
                0 => BangumiEpisodeStatus::NotCollected,
                1 => BangumiEpisodeStatus::WantToWatch,
                2 => BangumiEpisodeStatus::Watched,
                3 => BangumiEpisodeStatus::Dropped,
                _ => BangumiEpisodeStatus::NotCollected,
            },
        });
    }

    Ok(result)
}

pub fn update_bangumi_episode_status(bangumi_subject_id: i32, bangumi_episode_sort: String, status: BangumiEpisodeStatus) -> Result<(), Box<dyn Error>> {
    // First, get all the episode ids of the bangumi subject
    let episodes = get_bangumi_episodes(bangumi_subject_id)
        .map_err(|e| new_err(&format!("Failed to get bangumi episodes: {}", e)))?;

    // Then, match with the episode number
    // find episode_type=0 and episode_sort=bangumi_episode_num
    let episode_id = episodes.iter().find(|e| e.episode_type == 0 && e.episode_sort == bangumi_episode_sort)
        .ok_or_else(|| new_err("Failed to find the episode"))?;
    let episode_id = (*episode_id).episode_id;

    // Finally, update the status of the episode
    // curl -X 'PATCH' \
    //   'https://api.bgm.tv/v0/users/-/collections/{subject_id}/episodes' \
    //   -H 'accept: */*' \
    //   -H 'Authorization: Bearer {access_token}' \
    //   -H 'Content-Type: application/json' \
    //   -d '{
    //   "episode_id": [
    //     {episode_id}
    //   ],
    //   "type": 2
    // }'
    let access_token = CONFIG.read().unwrap().scrobbler_config.bangumi_access_token.clone();
    let url = format!("https://api.bgm.tv/v0/users/-/collections/{}/episodes", bangumi_subject_id);
    let client = reqwest::blocking::Client::new();
    let res = retry::retry(retry::delay::Fibonacci::from_millis(1000).take(5), || {
        client.patch(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            // UA is forced
            .header("User-Agent", "MapleWithered/Bangumi007 (https://github.com/MapleWithered/Bangumi007)")
            .json(&json!({
            "episode_id": [episode_id],
            "type": match status {
                BangumiEpisodeStatus::NotCollected => 0,
                BangumiEpisodeStatus::WantToWatch => 1,
                BangumiEpisodeStatus::Watched => 2,
                BangumiEpisodeStatus::Dropped => 3,
            }
        }))
            .send()
            .map_err(|e| new_err(&format!("Failed to send request: {}", e)))
    }).map_err(|e| new_err(&format!("Failed to send request: {}", e)))?;

    if !res.status().is_success() {
        return Err(new_err(&format!("Failed to update bangumi episode status: {}", res.text()?)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::module::logger;

    use super::*;

    #[test]
    fn test_get_bangumi_episode_collection_status() {
        logger::init();
        let result = get_bangumi_episode_collection_status(425909).unwrap();
        println!("{:?}", result);
    }

    #[test]
    fn test_update_bangumi_episode_status() {
        logger::init();
        update_bangumi_episode_status(425909, 7, BangumiEpisodeStatus::Watched).unwrap();
    }
}
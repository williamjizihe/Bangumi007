use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::RwLock;
use lazy_static::lazy_static;
use reqwest::blocking::{Client, multipart};
use serde::Deserialize;
use crate::config::{CONFIG, DownloaderConfig};
use crate::module::library::AnimeSeasonItem;

#[derive(Debug)]
struct Downloader {
    client: Client,
    cookie: String,
    last_login: i64,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct TorrentInfo {
    added_on: i64,
    amount_left: i64,
    auto_tmm: bool,
    availability: f32,
    category: String,
    completed: i64,
    completion_on: i64,
    content_path: String,
    dl_limit: i64,
    dlspeed: i64,
    downloaded: i64,
    downloaded_session: i64,
    eta: i64,
    f_l_piece_prio: bool,
    force_start: bool,
    hash: String,
    // isPrivate: bool,     // TODO: added in 5.0.0
    last_activity: i64,
    magnet_uri: String,
    max_ratio: f32,
    max_seeding_time: i64,
    name: String,
    num_complete: i64,
    num_incomplete: i64,
    num_leechs: i64,
    num_seeds: i64,
    priority: i64,
    progress: f32,
    ratio: f32,
    ratio_limit: f32,
    save_path: String,
    seeding_time: i64,
    seeding_time_limit: i64,
    seen_complete: i64,
    seq_dl: bool,
    size: i64,
    state: String,
    super_seeding: bool,
    tags: String,
    time_active: i64,
    total_size: i64,
    tracker: String,
    up_limit: i64,
    uploaded: i64,
    uploaded_session: i64,
    upspeed: i64,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct TorrentFile {
    index: i64,
    name: String,
    size: i64,
    progress: f32,
    priority: i64,
    is_seed: bool,
    piece_range: Vec<i64>,
    availability: f32,
}

lazy_static! {
    static ref DOWNLOADER: RwLock<Downloader> = RwLock::new(Downloader {
        client: Client::new(),
        cookie: String::new(),
        last_login: 0,
    });
}

/*
Login authenticate

$ curl -i --header 'Referer: http://localhost:8080' --data 'username=admin&password=adminadmin' http://localhost:8080/api/v2/auth/login
HTTP/1.1 200 OK
Content-Encoding:
Content-Length: 3
Content-Type: text/plain; charset=UTF-8
Set-Cookie: SID=hBc7TxF76ERhvIw0jQQ4LZ7Z1jQUV0tQ; path=/
$ curl http://localhost:8080/api/v2/torrents/info --cookie "SID=hBc7TxF76ERhvIw0jQQ4LZ7Z1jQUV0tQ"
 */

fn get_config() -> DownloaderConfig {
    CONFIG.read().unwrap().downloader_config.clone()
}

fn login() -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Attempting to login");
    let config = get_config();
    let url = format!("http://{}:{}/api/v2/auth/login", config.host, config.port);
    let resp = DOWNLOADER.write().unwrap().client.post(&url)
        .header("Referer", format!("http://{}:{}", config.host, config.port))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("username={}&password={}", config.username, config.password))
        .send()?;
    let status = resp.status().as_u16();
    let headers = resp.headers().clone();
    let body = resp.text().unwrap().clone();
    // assert code=200
    if status != 200 {
        // 403: Banned, others: unknown
        match status {
            403 => log::error!("Login failed, banned by WebUI."),
            _ => log::error!("Login failed, unknown error, status code: {}", status),
        }
        return Err("Login failed".into());
    }
    // assert "Ok." in body
    if body != "Ok." {
        log::error!("Login failed, perhaps password error. Response: {}", body);
        return Err("Login failed".into());
    }
    let cookie = headers.get("Set-Cookie").unwrap().to_str().unwrap().to_string();
    DOWNLOADER.write().unwrap().cookie = cookie;
    DOWNLOADER.write().unwrap().last_login = chrono::Local::now().timestamp();
    log::debug!("Successfully logged in");
    Ok(())
}

fn relogin_if_needed() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config();
    if chrono::Local::now().timestamp() - DOWNLOADER.read().unwrap().last_login > config.ttl {
        log::debug!("Session expired, login needed");
        login()?;
    }
    Ok(())
}

fn add_torrent_item(item: &AnimeSeasonItem) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Adding torrent");
    let config = get_config();
    relogin_if_needed()?;
    // replace \ / : * ? " < > |
    let title = item.mikan_subject_title
        .replace("\\", "")
        .replace("/", "")
        .replace(":", "")
        .replace("*", "")
        .replace("?", "")
        .replace("\"", "")
        .replace("<", "")
        .replace(">", "")
        .replace("|", "");
    let savepath = format!("{}/{}", config.download_dir, title);
    let url = format!("http://{}:{}/api/v2/torrents/add", config.host, config.port);
    let form = multipart::Form::new()
        .text("urls", item.mikan_item_magnet_link.clone())
        .text("savepath", savepath)
        .text("category", config.category)
        .text("tags", config.tags)
        .text("paused", if config.paused_after_add { "true" } else { "false" })
        .text("autoTMM", "false")
        .text("sequentialDownload", if config.sequential_download { "true" } else { "false" })
        .text("firstLastPiecePrio", if config.first_last_piece_prio { "true" } else { "false" });
    let cookie = &DOWNLOADER.read().unwrap().cookie.clone();
    let resp = DOWNLOADER.write().unwrap().client.post(&url)
        .header("Referer", format!("http://{}:{}", config.host, config.port))
        .header("Origin", format!("http://{}:{}", config.host, config.port))
        .header("Cookie", cookie)
        .multipart(form)
        .send()?;
    log::debug!("Add torrent response status: {}", resp.status());
    let body = resp.text()?;
    log::debug!("Add torrent response: {}", body);

    Ok(())
}


// fn add_torrent(magnet_link: String) -> Result<(), Box<dyn std::error::Error>> {
//     log::debug!("Adding torrent");
//     let config = get_config();
//     relogin_if_needed()?;
//     let url = format!("http://{}:{}/api/v2/torrents/add", config.host, config.port);
//     let form = multipart::Form::new()
//         .text("urls", magnet_link)
//         .text("savepath", config.download_dir)
//         .text("category", config.category)
//         .text("tags", config.tags)
//         .text("paused", if config.paused_after_add { "true" } else { "false" })
//         .text("autoTMM", "false")
//         .text("sequentialDownload", if config.sequential_download { "true" } else { "false" })
//         .text("firstLastPiecePrio", if config.first_last_piece_prio { "true" } else { "false" });
//     let cookie = &DOWNLOADER.read().unwrap().cookie.clone();
//     let resp = DOWNLOADER.write().unwrap().client.post(&url)
//         .header("Referer", format!("http://{}:{}", config.host, config.port))
//         .header("Origin", format!("http://{}:{}", config.host, config.port))
//         .header("Cookie", cookie)
//         .multipart(form)
//         .send()?;
//     log::debug!("Add torrent response status: {}", resp.status());
//     let body = resp.text()?;
//     log::debug!("Add torrent response: {}", body);
//
//     Ok(())
// }

// fn add_torrents(magnet_link: &Vec<AnimeSeasonItem>) -> Result<(), Box<dyn std::error::Error>> {
//     log::debug!("Adding torrents");
//     let config = get_config();
//     relogin_if_needed()?;
//     let download_dir = config.download_dir.clone();
//     // join bangumi name into download_dir
//
//     let url = format!("http://{}:{}/api/v2/torrents/add", config.host, config.port);
//     let form = multipart::Form::new()
//         .text("urls", magnet_link.join("\n"))
//         .text("savepath", download_dir)
//         .text("category", config.category)
//         .text("tags", config.tags)
//         .text("paused", if config.paused_after_add { "true" } else { "false" })
//         .text("autoTMM", "false")
//         .text("sequentialDownload", if config.sequential_download { "true" } else { "false" })
//         .text("firstLastPiecePrio", if config.first_last_piece_prio { "true" } else { "false" });
//     let cookie = &DOWNLOADER.read().unwrap().cookie.clone();
//     let resp = DOWNLOADER.write().unwrap().client.post(&url)
//         .header("Referer", format!("http://{}:{}", config.host, config.port))
//         .header("Origin", format!("http://{}:{}", config.host, config.port))
//         .header("Cookie", cookie)
//         .multipart(form)
//         .send()?;
//     log::debug!("Add torrent response status: {}", resp.status());
//     let body = resp.text()?;
//     log::debug!("Add torrent response: {}", body);
//
//     Ok(())
// }

fn list_torrents() -> Result<Vec<TorrentInfo>, Box<dyn Error>> {
    log::debug!("Listing torrents");
    let config = get_config();
    relogin_if_needed()?;
    let url = format!("http://{}:{}/api/v2/torrents/info", config.host, config.port);
    // ?filter=downloading&category=sample%20category&sort=ratio
    // url encode
    let category = urlencoding::encode(&config.category);
    let tags = urlencoding::encode(&config.tags);
    let url = format!("{}?category={}&tag={}", url, category, tags);
    let cookie = &DOWNLOADER.read().unwrap().cookie.clone();
    let resp = DOWNLOADER.write().unwrap().client.get(&url)
        .header("Referer", format!("http://{}:{}", config.host, config.port))
        .header("Origin", format!("http://{}:{}", config.host, config.port))
        .header("Cookie", cookie)
        .send()?;
    log::debug!("List torrents response status: {}", resp.status());
    let body = resp.text()?;
    // log::debug!("List torrents response: {}", body);
    // body is json
    let list_torrents: Vec<TorrentInfo> = serde_json::from_str(&body)?;
    Ok(list_torrents)
}

pub fn download_items(items: &Vec<AnimeSeasonItem>) -> Result<(), Box<dyn Error>> {
    let downloader_torrents = list_torrents()?;
    let downloader_hash: HashSet<String> = downloader_torrents.iter().map(|x| x.hash.clone()).collect();
    let mut library_hash: HashSet<String> = HashSet::new();
    let mut library_hash_to_item: HashMap<String, AnimeSeasonItem> = HashMap::new();
    for item in items {
        let hash = item.mikan_item_magnet_link.split("btih:").last().unwrap();
        let hash = hash.split("&").next().unwrap();
        library_hash.insert(hash.to_string());
        library_hash_to_item.insert(hash.to_string(), item.clone());
    }
    let hash_to_download = library_hash.difference(&downloader_hash).collect::<HashSet<&String>>();
    let items_to_download = hash_to_download.iter().map(|x| library_hash_to_item.get(*x).unwrap()).collect::<Vec<&AnimeSeasonItem>>();
    for item in items_to_download {
        add_torrent_item(item)?;
    }
    Ok(())
}

fn get_fileinfo(hash: &String) -> Result<Vec<TorrentFile>, Box<dyn Error>> {
    log::debug!("Get filename");
    let config = get_config();
    relogin_if_needed()?;
    let url = format!("http://{}:{}/api/v2/torrents/files", config.host, config.port);
    // ?hash={}
    let url = format!("{}?hash={}", url, hash);
    let cookie = &DOWNLOADER.read().unwrap().cookie.clone();
    let resp = DOWNLOADER.write().unwrap().client.get(&url)
        .header("Referer", format!("http://{}:{}", config.host, config.port))
        .header("Origin", format!("http://{}:{}", config.host, config.port))
        .header("Cookie", cookie)
        .send()?;
    log::debug!("List torrents response status: {}", resp.status());
    let body = resp.text()?;
    // log::debug!("List torrents response: {}", body);
    // body is json array
    let list_files: Vec<TorrentFile> = serde_json::from_str(&body)?;
    Ok(list_files)
}
// 
// pub fn rename_torrents_files(items: &Vec<AnimeSeasonItem>) -> Result<(), Box<dyn std::error::Error>> {
//     let config = get_config();
//     let hash_to_item: HashMap<String, AnimeSeasonItem> = items.iter().map(|x| {
//         let hash = x.mikan_item_magnet_link.split("btih:").last().unwrap();
//         let hash = hash.split("&").next().unwrap();
//         (hash.to_string(), x.clone())
//     }).collect();
//     let downloader_torrents = list_torrents()?;
//     let downloader_torrents_file_info: HashMap<String, Vec<TorrentFile>> =
//         downloader_torrents.iter().map(|x| (x.hash.clone(), get_fileinfo(&x.hash).unwrap())).collect();
//     // For each file, rename if SxxxExxx(x might be >=2 digits) not in filename
//     let regex = fancy_regex::Regex::new(r"S\d{2,}E\d{2,}").unwrap();
//     for (hash, files) in downloader_torrents_file_info {
//         let item = hash_to_item.get(&hash).unwrap();
//         for file in files {
//             // If match, continue
//             if regex.is_match(&file.name).unwrap() {
//                 continue;
//             }
//             // If not match, rename
//             let old_path = file.name;
//             // mikan_subject_title SxxExx.xxx
//             let new_path = format!(
//                 // "{} S{:02}E{:02}.{}",
//                 "{} S01E{:02}.{}",
//                 item.mikan_subject_title,
//                 item.episode_num_offseted,
//                 old_path.split(".").last().unwrap(),
//             );
//             log::debug!("Renaming file: {} -> {}", old_path, new_path);
//             // url encode
//             let old_path = urlencoding::encode(&old_path);
//             let new_path = urlencoding::encode(&new_path);
//             relogin_if_needed()?;
//             let url = format!("http://{}:{}/api/v2/torrents/renameFile", config.host, config.port);
//             // POST
//             // hash={}&f={}&n={}
//             log::debug!("Rename file response status: {}", resp.status());
//             let body = resp.text()?;
//             log::debug!("Rename file response: {}", body);
//         }
//     }
// 
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use crate::module::logger;
    use super::*;

    #[test]
    fn test_get_config() {
        get_config();
    }

    #[test]
    fn test_login() {
        logger::init();
        login().unwrap();
        let cookie = DOWNLOADER.read().unwrap().cookie.clone();
        assert!(!cookie.is_empty());
    }
    //
    // #[test]
    // fn test_add_torrent() {
    //     logger::init();
    //     login().unwrap();
    //     add_torrent_item("magnet:?xt=urn:btih:bc5fe73ecf6667dcefabdbdeb0f47fd985cc776e".to_string()).unwrap();
    // }
    //
    // #[test]
    // fn test_add_torrents() {
    //     logger::init();
    //     login().unwrap();
    //     add_torrents(&vec![
    //         "magnet:?xt=urn:btih:e7e23234005cb9c7b3dd4115ee7f19651753ee98".to_string(),
    //         "magnet:?xt=urn:btih:bc5fe73ecf6667dcefabdbdeb0f47fd985cc776e".to_string(),
    //         "magnet:?xt=urn:btih:0f83082453d63c3286a23d7f59faf38665d9a37b".to_string(),
    //     ]).unwrap();
    // }
    //
    // #[test]
    // fn test_list_torrents() {
    //     logger::init();
    //     login().unwrap();
    //     add_torrents(&vec![
    //         "magnet:?xt=urn:btih:e7e23234005cb9c7b3dd4115ee7f19651753ee98".to_string(),
    //         "magnet:?xt=urn:btih:bc5fe73ecf6667dcefabdbdeb0f47fd985cc776e".to_string(),
    //         "magnet:?xt=urn:btih:0f83082453d63c3286a23d7f59faf38665d9a37b".to_string(),
    //     ]).unwrap();
    //     let torrents_list = list_torrents().unwrap();
    //     for torrent in torrents_list {
    //         println!("{}", torrent.hash);
    //     }
    // }
}

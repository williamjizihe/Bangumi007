use std::fs;
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use futures::stream::StreamExt;

use std::time::Duration;
use async_std::task;
use async_std::prelude::*;

use async_std::task::spawn;
use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::module::database::library::find_season_by_disp;
use crate::module::scrobbler::bangumi::{BangumiEpisodeStatus, update_bangumi_episode_status, update_bangumi_episode_status_send};
use crate::module::utils::error::new_err;
use crate::ui::apps::libraryapp;
use crate::ui::apps::libraryapp::BANGUMI_STATUS_UPDATE;


pub(crate) async fn http_main() {
    let listener = TcpListener::bind("127.0.0.1:8007").await.unwrap();
    listener
        .incoming()
        .for_each_concurrent(/* limit */ None, |stream| async move {
            let stream = stream.unwrap();
            spawn(handle_connection(stream));
        })
        .await;
}

async fn handle_connection(mut stream: TcpStream) {

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap();

    // Parse buffer and decode url parameters
    // url: /report?series={}&season={}&episode={}&status={}
    // series: String,
    // season: i32,
    // episode: i32,
    // status: i32,
    let request = String::from_utf8_lossy(&buffer[..]);
    let request = request.split_whitespace().collect::<Vec<&str>>();
    let request = request[1].split("?").collect::<Vec<&str>>();
    let request = request[1].split("&").collect::<Vec<&str>>();
    let mut url_params = HashMap::new();
    for param in request {
        let param = param.split("=").collect::<Vec<&str>>();
        url_params.insert(param[0], param[1]);
    }

    let series = url_params.get("series");
    let season = url_params.get("season");
    let episode = url_params.get("episode");
    let status = url_params.get("status");

    match (series, season, episode, status) {
        (Some(series), Some(season), Some(episode), Some(status)) => {
            // continue to update_bangumi_episode_status(series, season, episode, status);
        },
        _ => {
            // return error
            let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
            stream.write(response.as_bytes()).await.unwrap();
            stream.flush().await.unwrap();
        }
    }

    let series = series.unwrap();
    let season = season.unwrap();
    let episode = episode.unwrap();
    let status = status.unwrap();

    // url decode
    let series = urlencoding::decode(series);
    // parse int
    let season = season.parse::<i32>();
    let episode = episode.parse::<i32>();
    let status = status.parse::<i32>();

    match (&series, &season, &episode, &status) {
        (Ok(series), Ok(season), Ok(episode), Ok(status)) => {
            // continue to update_bangumi_episode_status(series, season, episode, status);
        },
        _ => {
            // return error
            let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
            stream.write(response.as_bytes()).await.unwrap();
            stream.flush().await.unwrap();
        }
    }

    let series = series.unwrap().to_string();
    let season = season.unwrap();
    let episode = episode.unwrap();
    let status = status.unwrap();

    // Find (disp_series_name, disp_season_num) in database and get bangumi_subject_id, conf_tmdb_episode_offset, conf_bangumi_episode_offset
    let seasoninfo = find_season_by_disp(series.to_string(), season);

    if let None = seasoninfo {
        // return error
        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
        stream.write(response.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
        return;
    }
    let seasoninfo = seasoninfo.unwrap();

    let bangumi_subject_id = seasoninfo.bangumi_subject_id;

    // Push status to bangumi
    let result = update_bangumi_episode_status_send(
        bangumi_subject_id,
        (episode - seasoninfo.conf_tmdb_episode_offset + seasoninfo.conf_bangumi_episode_offset).to_string(),
        BangumiEpisodeStatus::Watched,
    );
    if !result {
        // return success
        let response ="HTTP/1.1 200 OK\r\n\r\n";
        stream.write(response.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();

        let mut handle = BANGUMI_STATUS_UPDATE.write().unwrap();
        *handle = true;
        drop(handle);
    } else {
        // return error
        let response ="HTTP/1.1 500 Internal Server Error\r\n\r\n";
        stream.write(response.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    };

}

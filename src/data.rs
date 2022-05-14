use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rand::prelude::*;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref DATA: HashMap<String, Show> = load_data("data");
}

#[derive(Deserialize)]
pub struct Show {
    pub title: String,
    pub seasons: Vec<Season>,
}

#[derive(Deserialize)]
pub struct Season {
    pub title: String,
    pub episodes: Vec<Episode>,
}

#[derive(Deserialize)]
pub struct Episode {
    pub title: String,
    pub aired: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct EpisodeResponse {
    pub season_idx: usize,
    pub season_name: String,
    pub episode_idx: usize,
    pub episode_name: String,
    pub episode_aired: String,
    pub episode_description: String,
}

pub fn get_shows() -> Vec<(String, String)> {
    let mut shows = DATA.iter()
        .map(|(k, v)| (k.clone(), v.title.clone()))
        .collect::<Vec<(String, String)>>();

    shows.sort_by(|a, b| a.cmp(&b));
    shows
}

pub fn get_random_episode<S: Into<String>>(show_id: S) -> Option<EpisodeResponse> {
    let mut rng = rand::thread_rng();

    let show = match DATA.get(&show_id.into()) {
        Some(s) => s,
        None => return None,
    };

    let season_id = rng.gen_range(0..show.seasons.len());
    let season = match show.seasons.get(season_id) {
        Some(s) => s,
        None => return None,
    };

    let episode_id = rng.gen_range(0..season.episodes.len());
    let episode = match season.episodes.get(episode_id) {
        Some(s) => s,
        None => return None,
    };

    Some(EpisodeResponse {
        season_idx: season_id + 1,
        season_name: season.title.clone(),
        episode_idx: episode_id + 1,
        episode_name: episode.title.clone(),
        episode_aired: episode.aired.clone(),
        episode_description: episode.description.clone(),
    })
}

fn load_data<P: AsRef<Path>>(path: P) -> HashMap<String, Show> {
    let files: Vec<PathBuf> = std::fs::read_dir(path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .unwrap();

    let data = files
        .into_iter()
        .map(|p| (p.clone(), std::fs::read_to_string(p).unwrap()));
    let data: HashMap<PathBuf, String> = HashMap::from_iter(data);

    data.iter()
        .map(|(k, v)| {
            let filename = k.file_stem().unwrap().to_os_string().into_string().unwrap();
            let show: Show = toml::from_str(v).unwrap();

            (filename, show)
        })
        .collect()
}

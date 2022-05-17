use std::collections::HashMap;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

macro_rules! add_show {
    ($map:expr, $name:tt) => {
        $map.insert(
            $name.into(),
            toml::from_str(include_str!(concat!("../data/", $name, ".toml"))).unwrap(),
        );
    };
}

lazy_static! {
    static ref DATA: HashMap<String, Show> = {
        let mut data = HashMap::new();

        add_show!(data, "brooklyn-nine-nine");
        add_show!(data, "community");
        add_show!(data, "friends");
        add_show!(data, "halt-and-catch-fire");
        add_show!(data, "mad-men");
        add_show!(data, "peaky-blinders");
        add_show!(data, "silicon-valley");
        add_show!(data, "superstore");

        data
    };
    static ref NAMES: Vec<(String, String)> = {
        let mut shows = DATA
            .iter()
            .map(|(k, v)| (k.clone(), v.title.clone()))
            .collect::<Vec<(String, String)>>();

        shows.sort_by(|a, b| a.cmp(&b));
        shows
    };
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

#[derive(Serialize)]
pub struct EpisodeResponse {
    pub season_idx: usize,
    pub season_name: String,
    pub episode_idx: usize,
    pub episode_name: String,
    pub episode_aired: String,
    pub episode_description: String,
}

pub fn get_shows() -> Vec<(String, String)> {
    NAMES.to_vec()
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

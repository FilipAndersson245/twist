use chrono::{DateTime, Duration as CDuration, NaiveDate, NaiveTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{de, ser};

use crate::{
    api::{fetch_all_animes, fetch_anime},
    types::{Anime, Animes, Episode, Episodes, ID},
};
use std::{
    collections::HashMap,
    error::Error,
    fs::{create_dir_all, read_to_string, write},
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub struct Store<T> {
    pub data: T,
    pub last_updated: DateTime<Utc>,
    pub path: PathBuf,
    pub update_intervall: Duration,
}

impl<T> Store<T>
where
    T: DeserializeOwned + Serialize,
{
    pub fn load(path: &Path) -> Result<Option<Self>, Box<dyn Error>> {
        if !path.exists() {
            create_dir_all(path)?;
        }

        let data_string = read_to_string(path)?;
        let data = de::from_str(&data_string)?;
        Ok(Some(data))
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        if !self.path.exists() {
            create_dir_all(&self.path)?;
        }
        let data = ser::to_string_pretty(self)?;
        write(&self.path, data)?;
        Ok(())
    }

    pub fn should_update(&self) -> bool {
        self.last_updated
            .naive_utc()
            .signed_duration_since(Utc::now().naive_utc())
            > CDuration::from_std(self.update_intervall.clone()).unwrap()
    }

    pub fn update(&mut self, data: T) -> Result<(), Box<dyn Error>> {
        self.data = data;
        self.save()?;
        Ok(())
    }
}

pub type AnimeStore = Store<Animes>;
pub static ANIME_PATH: &str = "./.cache/animes.json";

impl AnimeStore {
    pub fn new() -> Self {
        Self {
            data: Animes::default(),
            update_intervall: Duration::from_secs(60 * 60 * 24 * 3),
            last_updated: DateTime::from_utc(
                NaiveDate::from_ymd(1970, 1, 1).and_time(NaiveTime::from_hms(1, 1, 1)),
                Utc,
            ),
            path: Path::new(ANIME_PATH).to_path_buf(),
        }
    }

    pub async fn fetch(&mut self) -> Result<Animes, Box<dyn Error>> {
        if self.should_update() {
            let data = fetch_all_animes().await?;
            self.update(data)?;
        }

        Ok(self.data.clone())
    }
}

pub type EpisodeStore = Store<Episodes>;
pub type EpisodeMap = HashMap<Anime, EpisodeStore>;

impl EpisodeStore {
    pub fn new(anime: &Anime) -> Self {
        Self {
            data: Episodes::default(),
            update_intervall: Duration::from_secs(60 * 60 * 24 * 3),
            last_updated: DateTime::from_utc(
                NaiveDate::from_ymd(1970, 1, 1).and_time(NaiveTime::from_hms(1, 1, 1)),
                Utc,
            ),
            path: Path::new(&format!("./.cache/{}/episodes.json", anime.title)).to_path_buf(),
        }
    }

    pub async fn fetch(&mut self, anime: &Anime) -> Result<Episodes, Box<dyn Error>> {
        if self.should_update() {
            let data = fetch_anime(anime).await?;
            self.update(data)?;
        }
        Ok(self.data.clone())
    }
}

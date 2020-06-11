use serde::{Deserialize, Serialize};

type ID = u64;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Anime {
    pub id: ID,
    pub title: String,
    pub alt_title: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Episode {
    pub id: ID,
    pub source: String,
    pub number: i64,
    pub anime_id: ID,
}

pub type Episodes = Vec<Episode>;

pub type Animes = Vec<Anime>;

#[derive(Debug, Clone)]
pub struct DownloadInfo(pub Anime, pub Episode);

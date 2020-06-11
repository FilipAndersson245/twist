use crate::{
    types::{Anime, Animes, Episode, Episodes},
    ui::{DownloadMessage, Message},
};
use base64::decode;

use aes::Aes256;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};

use md5::{Digest, Md5};

use reqwest::{
    header::{HeaderMap, HeaderValue, CACHE_CONTROL, USER_AGENT},
    Response,
};

use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{prelude::*, SeekFrom},
    path::Path,
};
use tokio::sync::mpsc::Sender;
use url::Url;

use serde::{Deserialize, Serialize};

use chrono::{NaiveDate, Utc};
use serde_json::{de, ser};

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

static KEY: &[u8] = b"LXgIVP&PorO68Rq7dTx8N^lP!Fa5sGJ^*XK";

static USER_AGENT_VALUE: &'static str = "Mozilla/5.0 (iPhone; CPU iPhone OS 12_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148";

#[inline]
pub fn construct_header() -> HeaderMap {
    let user_agent = HeaderValue::from_str(USER_AGENT_VALUE).unwrap();
    let access_token = HeaderValue::from_str("1rj2vRtegS8Y60B3w3qNZm5T2Q0TN2NR").unwrap();
    let referer = HeaderValue::from_str("https://twist.moe/").unwrap();
    let cache_timer = HeaderValue::from_str(&format!("max-age={}", 1000)).unwrap();

    let mut header = HeaderMap::new();
    header.insert("Referer", referer);
    header.insert(USER_AGENT, user_agent);
    header.insert("x-access-token", access_token);
    header.insert(CACHE_CONTROL, cache_timer);

    header
}

/// Will only fetch main anime data without any additional properies.
pub async fn fetch_all_animes() -> Result<Animes, Box<dyn Error>> {
    if let Ok(cache) = CachedRequest::<Animes>::load() {
        if !cache.should_update() {
            return Ok(cache.data);
        }
    }
    let response: Animes = reqwest::Client::new()
        .get("https://twist.moe/api/anime")
        .headers(construct_header())
        .send()
        .await?
        .json()
        .await?;
    let _ = CachedRequest {
        data: response.clone(),
        updated_at: Utc::now().naive_utc().date(),
    }
    .save();
    Ok(response)
}
pub fn clear_title(s: &str) -> String {
    s.trim()
        .replace(&[' ', '\''][..], "-")
        .replace(
            &[
                ' ', '~', '@', '#', '$', '&', '(', ')', '*', '!', '+', '=', ':', ';', ',', '.',
                '?', '/', '\'',
            ][..],
            "",
        )
        .to_lowercase()
}

pub async fn fetch_anime(anime: &Anime) -> Result<Episodes, Box<dyn Error>> {
    let url = Url::parse(&format!(
        "https://twist.moe/api/anime/{}/sources",
        clear_title(&anime.title)
    ))?;
    let episodes: Episodes = reqwest::Client::new()
        .get(url)
        .headers(construct_header())
        .send()
        .await?
        .json()
        .await?;
    Ok(episodes)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedRequest<T>
where
    T: Default,
{
    pub data: T,
    pub updated_at: NaiveDate,
}

impl<T: Default> Default for CachedRequest<T> {
    fn default() -> Self {
        Self {
            data: T::default(),
            updated_at: NaiveDate::from_ymd(1980, 1, 1),
        }
    }
}

impl CachedRequest<Animes> {
    pub fn should_update(&self) -> bool {
        let now = Utc::now().naive_utc().date();
        let time = now.signed_duration_since(self.updated_at);

        time.num_days() > 4
    }

    pub fn load() -> Result<CachedRequest<Animes>, Box<dyn Error>> {
        let path = Path::new("./.cache/");
        fs::create_dir_all(path)?;
        let path = path.join("animes.json");
        let s = fs::read_to_string(path)?;
        let cache: CachedRequest<Animes> = de::from_str(&s)?;
        Ok(cache)
    }
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let data = ser::to_string(self)?;
        let path = Path::new("./.cache/");
        fs::create_dir_all(path)?;
        let path = path.join("animes.json");
        fs::write(path, data)?;
        Ok(())
    }
}

fn get_salt_and_data(data: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
    if b"Salted__" == &data[0..8] {
        ()
    }

    let salt = data[8..16].to_vec();
    let text = data[16..].to_vec();

    Some((salt, text))
}

fn bytes_to_key(data: Vec<u8>, salt: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    let data_and_salt = [data, salt].concat();
    let mut key = Md5::digest(data_and_salt.as_ref());

    // let mut key = hash(MessageDigest::md5(), data_and_salt.as_ref())?;
    let mut final_key: Vec<u8> = Vec::with_capacity(64);
    // key.as_slice().clone().to_vec();
    final_key.append(&mut key.to_vec());

    while final_key.len() < 48 {
        key = Md5::digest([key.to_vec(), data_and_salt.clone()].concat().as_ref());
        final_key.append(&mut key.to_vec())
    }

    Ok(final_key[0..48].to_vec())
}

fn decrypt_data(encrypted_data: &str) -> Result<String, Box<dyn Error>> {
    let decoded_encrypted_data = decode(encrypted_data)?;
    let (salt, mut text): (Vec<u8>, Vec<u8>) = get_salt_and_data(decoded_encrypted_data).unwrap();
    let text = text.as_mut_slice();

    let key_iv = bytes_to_key(KEY.to_vec(), salt)?;
    let key = key_iv[0..32].to_vec();
    let iv = key_iv[32..].to_vec();

    // let a = BlockMode::new_var(&key, &iv);
    let cipher = Aes256Cbc::new_var(&key, &iv)?;
    cipher.decrypt(text)?;

    //let decrypted_text = decrypt(cipher, &key, Some(&iv), &text)?;
    let decrypted_string = String::from_utf8(text.to_vec())?;
    Ok(decrypted_string)
}

fn decrypt_source_url(episode: &Episode) -> Result<Url, Box<dyn Error>> {
    let decrypted_path = decrypt_data(&episode.source)?;
    let url_string = format!("https://twist.moe{}", decrypted_path);
    let url = Url::parse(&url_string)?;
    Ok(url)
}

pub async fn fetch_video(
    episode: &Episode,
    anime: &Anime,
    mut sender: Sender<Message>,
) -> Result<(), Box<dyn Error>> {
    let path = format!("./animes/{}", clear_title(&anime.title));

    let path = Path::new(&path);
    fs::create_dir_all(path)?; // Create folder if it don't exist.
    let path = path.join(format!("{}.mp4", episode.number));

    let mut file = OpenOptions::new().write(true).create(true).open(path)?;

    let mut header = construct_header();

    let file_size = file.seek(SeekFrom::End(0))?; // Find file size and set file pointer there.
    if file_size > 0 {
        // If resume, skip these bytes.
        let range = HeaderValue::from_str(&format!("bytes={}-", file_size))?;
        header.append(reqwest::header::RANGE, range);
    }

    let video_url = decrypt_source_url(episode)?;
    let mut response: Response = reqwest::Client::new()
        .get(video_url)
        .headers(header)
        .send()
        .await?;

    let content_length = match response.content_length() {
        Some(length) => length + file_size,
        None => {
            sender
                .send(Message::Notification(tui::widgets::Text::raw(
                    "Could not find how large the file would be :(",
                )))
                .await?;
            file_size
        }
    };

    let mut fetched_so_far = file_size;

    sender
        .send(Message::Download(DownloadMessage::Starting))
        .await?;

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
        fetched_so_far += chunk.len() as u64;
        sender
            .send(Message::Download(DownloadMessage::Progress(
                fetched_so_far,
                content_length,
            )))
            .await?;
    }
    sender
        .send(Message::Download(DownloadMessage::Finished))
        .await?;

    Ok(())
}

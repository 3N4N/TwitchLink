use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Secrets {
  client_id: String,
  client_secret: String,
  oauth_token: String,
}

pub fn get_vod_link(
  vod_id: String,
  secrets: Secrets,
) -> Result<String, Box<dyn std::error::Error>> {
  #[derive(Serialize, Deserialize)]
  pub struct Datum {
    #[serde(rename = "id")]
    id: String,

    #[serde(rename = "thumbnail_url")]
    thumbnail_url: String,

    #[serde(rename = "title")]
    title: String,

    #[serde(rename = "url")]
    url: String,

    #[serde(rename = "user_id")]
    user_id: String,
  }

  #[derive(Serialize, Deserialize)]
  struct VideosResponse {
    #[serde(rename = "data")]
    data: Vec<Datum>,
  }

  let client = reqwest::blocking::Client::new();
  let res = client
    .get("https://api.twitch.tv/helix/videos?id=".to_string() + vod_id.as_str())
    .header("Content-Type", "application/json")
    .header("Client-Id", secrets.client_id)
    .header("Authorization", secrets.oauth_token)
    .send()?
    .text()?;

  let res: VideosResponse = serde_json::from_str(&res)?;
  assert!(res.data.len() == 1, "[ERR] Unexpected helix response");
  let thumbnail_url = res.data[0].thumbnail_url.to_string();

  let re = Regex::new(
    r"https://static-cdn\.jtvnw\.net/cf_vods/([a-z0-9_]*)/([a-z0-9_]*)//thumb/thumb\d-%\{width\}x%\{height\}\.jpg",
  )?;
  let caps = re.captures(thumbnail_url.as_str()).unwrap();
  let vod_link = "https://".to_string()
    + caps.get(1).unwrap().as_str()
    + ".cloudfront.net"
    + "/"
    + caps.get(2).unwrap().as_str()
    + "/720p60/index-dvr.m3u8";

  Ok(vod_link)
}

fn main() {
  let args: Vec<String> = env::args().collect();
  assert!(args.len() == 2, "[ERR] Link not found.");

  let vod_id = (&args[1]).to_string();
  println!("VOD ID: {vod_id}");

  // Read Twitch secrets
  let home_dir = dirs::home_dir()
    .expect("[ERR] $HOME unset")
    .to_str()
    .expect("[ERR] $HOME cannot be accessed")
    .to_string();
  let path = home_dir + "/.TwitchLink/secrets.json";
  let data =
    fs::read_to_string(path).expect("[ERR] Unable to read secrets.json");
  let secrets: Secrets =
    serde_json::from_str(&data).expect("[ERR] Unable to parse secrets.json");

  let vodlink =
    get_vod_link(vod_id, secrets).expect("[ERR] GET request failed");
  println!("VOD link: {}", vodlink);
}

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Secrets {
  client_id: String,
  client_secret: String,
  oauth_token: String,
}

pub fn get_vod_info(
  vod_id: String,
  secrets: Secrets,
) -> Result<String, Box<dyn std::error::Error>> {
  let client = reqwest::blocking::Client::new();
  let res = client
    .get("https://api.twitch.tv/helix/videos?id=".to_string() + vod_id.as_str())
    .header("Content-Type", "application/json")
    .header("Client-Id", secrets.client_id)
    .header("Authorization", secrets.oauth_token)
    .send()?
    .text()?;

  Ok(res)
}

pub fn estimate_vod_link(
  vod_info: String,
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

  let res: VideosResponse = serde_json::from_str(&vod_info)?;
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

pub fn get_vod_link(
  vod_id: String,
  secrets: Secrets,
) -> Result<String, Box<dyn std::error::Error>> {
  let vod_info = get_vod_info(vod_id, secrets)?;
  let vod_link = estimate_vod_link(vod_info)?;

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_estimate_vod_link() {
    let vod_info = r#"
{
  "data": [
    {
      "created_at": "2023-04-26T17:03:25Z",
      "description": "",
      "duration": "6h46m38s",
      "id": "1804186756",
      "language": "en",
      "muted_segments": null,
      "published_at": "2023-04-26T17:03:25Z",
      "stream_id": "48380328493",
      "thumbnail_url": "https://static-cdn.jtvnw.net/cf_vods/d1m7jfoe9zdc1j/51b4df78ae6d180ce585_elizabethzaks_48380328493_1682528600//thumb/thumb0-%{width}x%{height}.jpg",
      "title": "RE8 FINALE AND SPOOKS W/ @nikkiblackketter !socials",
      "type": "archive",
      "url": "https://www.twitch.tv/videos/1804186756",
      "user_id": "214714452",
      "user_login": "elizabethzaks",
      "user_name": "ElizabethZaks",
      "view_count": 1262,
      "viewable": "public"
    }
  ],
  "pagination": {}
}"#;

    let vod_link = estimate_vod_link(vod_info.to_string()).expect("[FAIL] Cannot estimate VOD link.");
    assert_eq!(vod_link, "https://d1m7jfoe9zdc1j.cloudfront.net/51b4df78ae6d180ce585_elizabethzaks_48380328493_1682528600/720p60/index-dvr.m3u8");
  }
}

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
  vod_id: &str,
  secrets: Secrets,
) -> Result<String, Box<dyn std::error::Error>> {
  let client = reqwest::blocking::Client::new();
  let res = client
    .get("https://api.twitch.tv/helix/videos?id=".to_string() + vod_id)
    .header("Content-Type", "application/json")
    .header("Client-Id", secrets.client_id)
    .header("Authorization", secrets.oauth_token)
    .send()?
    .text()?;

  Ok(res)
}

pub fn print_vod_links(vod_id: &str, secrets: Secrets) {
  #[derive(Serialize, Deserialize)]
  struct Datum {
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

  const DOMAINS: [&str; 14] = [
    "https://d1m7jfoe9zdc1j.cloudfront.net",
    "https://d1mhjrowxxagfy.cloudfront.net",
    "https://d1ymi26ma8va5x.cloudfront.net",
    "https://d2aba1wr3818hz.cloudfront.net",
    "https://d2e2de1etea730.cloudfront.net",
    "https://d2nvs31859zcd8.cloudfront.net",
    "https://d2vjef5jvl6bfs.cloudfront.net",
    "https://d3aqoihi2n8ty8.cloudfront.net",
    "https://d3c27h4odz752x.cloudfront.net",
    "https://d3vd9lfkzbru3h.cloudfront.net",
    "https://ddacn6pr5v0tl.cloudfront.net",
    "https://dgeft87wbj63p.cloudfront.net",
    "https://dqrpb9wgowsf5.cloudfront.net",
    "https://ds0h3roq6wcgc.cloudfront.net",
  ];

  let vod_info =
    get_vod_info(vod_id, secrets).expect("[ERR] Could not get VOD info");
  let res: VideosResponse =
    serde_json::from_str(&vod_info).expect("[ERR] Could not parse vod info");
  assert!(res.data.len() == 1, "[ERR] Unexpected helix response");
  let thumbnail_url = res.data[0].thumbnail_url.to_string();
  // println!("{}", &thumbnail_url);

  let re = Regex::new(
    r"https://static-cdn\.jtvnw\.net/cf_vods/([a-z0-9_]+)/([a-z0-9_]+)//?thumb/.+%\{width\}x%\{height\}\.jpe?g",
  ).expect("[ERR] Could not init regex for capturing thumbnail url");
  let caps = re.captures(thumbnail_url.as_str()).unwrap();
  for domain in DOMAINS.iter() {
    let vod_link = domain.to_string()
      + "/"
      + caps.get(2).unwrap().as_str()
      + "/720p60/index-dvr.m3u8";
    // println!("{}", &vod_link);

    let client = reqwest::blocking::Client::new();
    let res = client
      .get(&vod_link)
      .send()
      .expect("[ERR] Could not send get request to tentative vod link")
      .status();
    if res.is_success() {
      println!("VOD link: {}", &vod_link);
    }
  }
}

// Ref: https://github.com/TwitchRecover/TwitchRecover/blob/ebf0bd413216e6ddcba72e9947b9cadd3110fe6d/src/TwitchRecover.Core/API/API.java#L204
//
// FIXME: It requires a client id, but it's using the client id used in
// TwitchRecover.  It's not the user-provided client id.  The user's client id
// doesn't seem to work with gql api.  Maybe the client id it expects is a
// different one.
pub fn get_stream_info(
  channel: &str,
) -> Result<String, Box<dyn std::error::Error>> {
  let client = reqwest::blocking::Client::new();
  let res = client
    .post("https://gql.twitch.tv/gql")
    .header("Content-Type", "Content-Type: text/plain")
    .header("Client-Id", "kimne78kx3ncx6brgo4mv6wki5h1ko")
    .body("{\"operationName\": \"PlaybackAccessToken\",\"variables\": {\"isLive\": true,\"login\": \"".to_string() + channel + "\",\"isVod\": false,\"vodID\": \"\",\"playerType\": \"channel_home_live\"},\"extensions\": {\"persistedQuery\": {\"version\": 1,\"sha256Hash\": \"0828119ded1c13477966434e15800ff57ddacf13ba1911c129dc2200705b0712\"}}}")
    .send()?
    .text()?;

  Ok(res)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamPlaybackAccessToken {
  #[serde(rename = "value")]
  value: String,

  #[serde(rename = "signature")]
  signature: String,
}

pub fn get_stream_token(
  stream_info: String,
) -> Result<StreamPlaybackAccessToken, Box<dyn std::error::Error>> {
  #[derive(Serialize, Deserialize)]
  struct StreamInfo {
    #[serde(rename = "data")]
    data: Data,
  }

  #[derive(Serialize, Deserialize)]
  struct Data {
    #[serde(rename = "streamPlaybackAccessToken")]
    stream_playback_access_token: StreamPlaybackAccessToken,
  }

  let res: StreamInfo = serde_json::from_str(&stream_info)?;
  let access_token = res.data.stream_playback_access_token;

  Ok(access_token)
}

pub fn print_stream_links(channel: &str) {
  let stream_info =
    get_stream_info(channel).expect("[ERR] Could not get stream info");
  let token =
    get_stream_token(stream_info).expect("[ERR] Could not get stream token");
  // println!("{}", token.value);
  // println!("{}", token.signature);

  let req = "https://usher.ttvnw.net/api/channel/hls/".to_string()
    + channel
    + ".m3u8?sig="
    + token.signature.as_str()
    + "&token="
    + token.value.as_str()
    + "&allow_source=true&allow_audio_only=true";

  let resp =
    reqwest::blocking::get(req).expect("[ERR] usher.ttvnw.net request failed");
  let body = resp
    .text()
    .expect("[ERR] usher.ttvnw.net response body invalid");

  let split = body.split("\n");
  for s in split {
    if &s[0..5] == "https" {
      println!("{}\n", s);
    } else if &s[0..7] != "#EXTM3U" && &s[24..32] == "GROUP-ID" {
      let re = Regex::new(r#"NAME="(.+?)""#)
        .expect("[ERR] Could not init regex for capturing stream format");
      let caps = re.captures(s).unwrap();
      print!("{}: ", caps.get(1).unwrap().as_str());
    }
  }
}

pub fn get_secrets() -> Secrets {
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

  secrets
}

fn main() {
  let args: Vec<String> = env::args().collect();
  assert!(args.len() == 2, "[ERR] Link not found.");

  let user_arg = &args[1];
  println!("ID: {user_arg}");

  // Guess if user_arg is for vod or stream.
  // VOD arg should be all numeric.
  // Stream arg should be alphanumeric.
  let mut want_vod = true;
  for c in user_arg.chars() {
    if c.is_alphabetic() {
      want_vod = false;
      break;
    }
  }

  if want_vod {
    // Read user secrets
    let secrets = get_secrets();
    print_vod_links(user_arg, secrets);
  } else {
    print_stream_links(user_arg);
  }
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

    let vod_link = estimate_vod_link(vod_info.to_string())
      .expect("[FAIL] Cannot estimate VOD link.");
    println!("{}", vod_link);
    assert_eq!(vod_link, "https://d1m7jfoe9zdc1j.cloudfront.net/51b4df78ae6d180ce585_elizabethzaks_48380328493_1682528600/720p60/index-dvr.m3u8");
  }
}

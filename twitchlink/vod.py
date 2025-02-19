import re
import json
import requests

from utils import Secrets

def get_stream_info(channel: str) -> str:
    url = "https://gql.twitch.tv/gql"
    headers = {
        "Content-Type": "Content-Type: text/plain",  # replicating the original header value
        "Client-Id": "kimne78kx3ncx6brgo4mv6wki5h1ko"
    }

    payload = {
        "operationName": "PlaybackAccessToken",
        "variables": {
            "isLive": True,
            "login": channel,
            "isVod": False,
            "vodID": "",
            "playerType": "channel_home_live"
        },
        "extensions": {
            "persistedQuery": {
                "version": 1,
                "sha256Hash": "0828119ded1c13477966434e15800ff57ddacf13ba1911c129dc2200705b0712"
            }
        }
    }

    # Convert the payload dictionary to a JSON-formatted string.
    body = json.dumps(payload)

    response = requests.post(url, headers=headers, data=body)
    response.raise_for_status()  # Raise an HTTPError if the request returned an unsuccessful status code
    return response.text

def get_vod_info(vod_id: str, secrets: Secrets) -> str:
    url = "https://api.twitch.tv/helix/videos?id=" + vod_id
    headers = {
        "Content-Type": "application/json",
        "Client-Id": secrets.client_id,
        "Authorization": secrets.oauth_token,
    }

    response = requests.get(url, headers=headers)
    response.raise_for_status()  # Raises an error for bad responses (4xx or 5xx)
    return response.text

def print_vod_links(vod_id: str, secrets: Secrets):
    # Get the VOD info as a JSON string.
    vod_info = get_vod_info(vod_id, secrets)

    # Parse the JSON response.
    parsed = json.loads(vod_info)

    # Ensure that the "data" key exists and contains exactly one item.
    if "data" not in parsed or len(parsed["data"]) != 1:
        raise Exception("[ERR] Unexpected helix response")

    # Extract the thumbnail URL.
    thumbnail_url = parsed["data"][0]["thumbnail_url"]

    # Compile the regex to capture parts of the thumbnail URL.
    pattern = re.compile(
        r"https://static-cdn\.jtvnw\.net/cf_vods/([a-z0-9_]+)/([a-z0-9_]+)//?thumb/.+%\{width\}x%\{height\}\.jpe?g"
    )
    match = pattern.search(thumbnail_url)
    if not match:
        raise Exception("[ERR] Could not capture thumbnail url")

    # List of potential domains.
    domains = [
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
    ]

    # Try each domain to build the VOD link and check if it is accessible.
    for domain in domains:
        vod_link = f"{domain}/{match.group(2)}/chunked/index-dvr.m3u8"
        try:
            response = requests.get(vod_link)
            if response.ok:
                print(f"VOD: {vod_link}")
                break
        except requests.RequestException:
            # If the request fails, try the next domain.
            continue

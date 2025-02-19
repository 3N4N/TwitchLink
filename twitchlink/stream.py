import json
import requests
from dataclasses import dataclass

def get_stream_info(channel: str) -> str:
    url = "https://gql.twitch.tv/gql"
    headers = {
        "Content-Type": "Content-Type: text/plain",
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
    response.raise_for_status()  # Raise an exception for HTTP errors
    return response.text

@dataclass
class StreamPlaybackAccessToken:
    value: str
    signature: str

def get_stream_token(stream_info: str) -> StreamPlaybackAccessToken:
    """
    Parse the provided JSON stream_info string and extract the stream playback access token.

    Expected JSON structure:
    {
      "data": {
        "streamPlaybackAccessToken": {
          "value": "<token_value>",
          "signature": "<token_signature>"
        }
      }
    }

    :param stream_info: JSON string containing stream info.
    :return: StreamPlaybackAccessToken object.
    :raises Exception: If parsing fails or the required keys are missing.
    """
    try:
        parsed = json.loads(stream_info)
        token_data = parsed["data"]["streamPlaybackAccessToken"]
        return StreamPlaybackAccessToken(
            value=token_data["value"],
            signature=token_data["signature"]
        )
    except (KeyError, json.JSONDecodeError) as e:
        raise Exception(f"Failed to parse stream token: {e}")

def print_stream_links(channel: str):
    stream_info = get_stream_info(channel)
    stream_token = get_stream_token(stream_info)

    url = "https://usher.ttvnw.net/api/channel/hls/{}.m3u8?sig={}&token={}&allow_source=true&allow_audio_only=true".format(
        channel, stream_token.signature, stream_token.value
    )
    response = requests.get(url)
    response.raise_for_status()
    print(response.text)

import re
import json
import requests
import argparse

import vod
import stream
from utils import Secrets

if __name__ == "__main__":
    secrets = Secrets(
        client_id="<your-client-id>",
        oauth_token="Bearer <your-oauth-token>"
    )

    parser = argparse.ArgumentParser()
    parser.add_argument("-v", "--vod", type=str, default=None)
    parser.add_argument("-s", "--stream", type=str, default=None)
    args = parser.parse_args()

    vod_id = args.vod
    channel_name = args.stream

    if channel_name is not None:
        stream.print_stream_links(channel_name)
    if vod_id is not None:
        vod.print_vod_links(vod_id, secrets)

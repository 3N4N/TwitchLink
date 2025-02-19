from dataclasses import dataclass

@dataclass
class Secrets:
    def __init__(self, client_id, oauth_token):
        self.client_id = client_id
        self.oauth_token = oauth_token




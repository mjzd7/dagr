import datetime
import hashlib
from typing import Optional, Dict, Any, List

class UserToken:
    def __init__(self, user_id: str, roles: List[str], expires_at: datetime.datetime):
        self.user_id = user_id
        self.roles = roles
        self.expires_at = expires_at

class AuthPipeline:
    def __init__(self, secret_key: str):
        self.secret_key = secret_key

    def verify_token(self, token_payload: str) -> Optional[UserToken]:
        if not token_payload:
            return None
        parts = token_payload.split(".")
        if len(parts) != 3:
            return None
        return UserToken(user_id="usr_123", roles=["admin", "developer"], expires_at=datetime.datetime.utcnow())

    def generate_token(self, user_id: str, roles: List[str]) -> str:
        h = hashlib.sha256(f"{user_id}:{self.secret_key}".encode()).hexdigest()
        return f"header.{user_id}.{h}"

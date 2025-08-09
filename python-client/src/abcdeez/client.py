"""
Main client for interacting with the Graph Learning API.
"""

import json
import time
from typing import Optional, Dict, List, Any, Union
from datetime import datetime, timedelta
import requests
from urllib.parse import urljoin

from .models import (
    User,
    Session,
    Task,
    TaskResponse,
    Analytics,
    SessionConfig,
    BatchTrialSubmission,
)
from .exceptions import (
    AbcdeezError,
    AuthenticationError,
    RateLimitError,
    ValidationError,
    NetworkError,
)


class GraphLearningClient:
    """Client for interacting with the Graph Learning System API."""
    
    def __init__(
        self,
        base_url: str = "https://api.graphlearning.app/v1",
        api_key: Optional[str] = None,
        timeout: int = 30,
        retry_count: int = 3,
        retry_delay: float = 1.0,
    ):
        """
        Initialize the Graph Learning client.
        
        Args:
            base_url: Base URL of the API
            api_key: Optional API key for authentication
            timeout: Request timeout in seconds
            retry_count: Number of retries for failed requests
            retry_delay: Delay between retries in seconds
        """
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.timeout = timeout
        self.retry_count = retry_count
        self.retry_delay = retry_delay
        
        self.session = requests.Session()
        self.session.headers.update({
            "Content-Type": "application/json",
            "User-Agent": f"abcdeez-python/{__import__('abcdeez').__version__}",
        })
        
        self._access_token: Optional[str] = None
        self._refresh_token: Optional[str] = None
        self._token_expires: Optional[datetime] = None
        self._user: Optional[User] = None
    
    def _request(
        self,
        method: str,
        endpoint: str,
        data: Optional[Dict] = None,
        params: Optional[Dict] = None,
        authenticated: bool = True,
    ) -> Dict:
        """Make an HTTP request to the API."""
        url = urljoin(self.base_url, endpoint)
        
        headers = {}
        if authenticated and self._access_token:
            headers["Authorization"] = f"Bearer {self._access_token}"
        elif self.api_key:
            headers["X-API-Key"] = self.api_key
        
        for attempt in range(self.retry_count):
            try:
                response = self.session.request(
                    method=method,
                    url=url,
                    json=data,
                    params=params,
                    headers=headers,
                    timeout=self.timeout,
                )
                
                if response.status_code == 429:
                    # Rate limited
                    retry_after = int(response.headers.get("X-RateLimit-Reset", 60))
                    raise RateLimitError(f"Rate limited. Retry after {retry_after} seconds")
                
                if response.status_code == 401:
                    if authenticated and self._refresh_token:
                        # Try to refresh token
                        self.refresh_token()
                        headers["Authorization"] = f"Bearer {self._access_token}"
                        continue
                    raise AuthenticationError("Authentication failed")
                
                if response.status_code == 422:
                    error_data = response.json()
                    raise ValidationError(error_data.get("error", {}).get("message", "Validation error"))
                
                response.raise_for_status()
                
                if response.content:
                    return response.json()
                return {}
                
            except requests.exceptions.RequestException as e:
                if attempt < self.retry_count - 1:
                    time.sleep(self.retry_delay * (2 ** attempt))
                    continue
                raise NetworkError(f"Network error: {str(e)}")
    
    # Authentication methods
    
    def register(self, email: str, password: str, display_name: str) -> User:
        """Register a new user account."""
        data = {
            "email": email,
            "password": password,
            "display_name": display_name,
        }
        
        response = self._request("POST", "/auth/register", data=data, authenticated=False)
        return User(**response)
    
    def login(self, email: str, password: str) -> User:
        """Login with email and password."""
        data = {
            "email": email,
            "password": password,
        }
        
        response = self._request("POST", "/auth/login", data=data, authenticated=False)
        
        self._access_token = response["access_token"]
        self._refresh_token = response["refresh_token"]
        self._token_expires = datetime.now() + timedelta(seconds=response["expires_in"])
        self._user = User(**response["user"])
        
        return self._user
    
    def refresh_token(self) -> None:
        """Refresh the access token."""
        if not self._refresh_token:
            raise AuthenticationError("No refresh token available")
        
        data = {"refresh_token": self._refresh_token}
        response = self._request("POST", "/auth/refresh", data=data, authenticated=False)
        
        self._access_token = response["access_token"]
        self._token_expires = datetime.now() + timedelta(seconds=response["expires_in"])
    
    def logout(self) -> None:
        """Logout and clear tokens."""
        self._access_token = None
        self._refresh_token = None
        self._token_expires = None
        self._user = None
    
    # Session methods
    
    def create_session(
        self,
        domain: str,
        config: Optional[SessionConfig] = None,
    ) -> Session:
        """Create a new learning session."""
        data = {
            "domain": domain,
            "config": config.dict() if config else {},
        }
        
        response = self._request("POST", "/sessions", data=data)
        return Session(**response)
    
    def get_sessions(
        self,
        limit: int = 20,
        offset: int = 0,
        status: Optional[str] = None,
        domain: Optional[str] = None,
        from_date: Optional[datetime] = None,
        to_date: Optional[datetime] = None,
    ) -> List[Session]:
        """Get list of sessions."""
        params = {
            "limit": limit,
            "offset": offset,
        }
        
        if status:
            params["status"] = status
        if domain:
            params["domain"] = domain
        if from_date:
            params["from_date"] = from_date.isoformat()
        if to_date:
            params["to_date"] = to_date.isoformat()
        
        response = self._request("GET", "/sessions", params=params)
        return [Session(**s) for s in response["sessions"]]
    
    def get_session(self, session_id: str) -> Session:
        """Get detailed session information."""
        response = self._request("GET", f"/sessions/{session_id}")
        return Session(**response)
    
    def end_session(self, session_id: str) -> Session:
        """End an active session."""
        response = self._request("PUT", f"/sessions/{session_id}/end")
        return Session(**response)
    
    # Task methods
    
    def get_next_task(
        self,
        session_id: str,
        previous_response: Optional[TaskResponse] = None,
    ) -> Task:
        """Get the next task for a session."""
        data = {}
        if previous_response:
            data["previous_response"] = previous_response.dict()
        
        response = self._request("POST", f"/sessions/{session_id}/tasks", data=data)
        return Task(**response)
    
    def submit_response(
        self,
        session_id: str,
        task_id: str,
        response: str,
        response_time: float,
    ) -> Dict:
        """Submit a response to a task."""
        data = {
            "response": response,
            "response_time": response_time,
            "client_timestamp": datetime.now().isoformat(),
        }
        
        return self._request("POST", f"/sessions/{session_id}/tasks/{task_id}/response", data=data)
    
    def batch_submit_trials(
        self,
        session_id: str,
        trials: List[Dict],
        client_info: Optional[Dict] = None,
    ) -> Dict:
        """Submit multiple trial responses at once."""
        data = {
            "trials": trials,
            "client_info": client_info or {
                "app_version": __import__('abcdeez').__version__,
                "platform": "python",
            },
        }
        
        return self._request("POST", f"/batch/sessions/{session_id}/trials", data=data)
    
    # Analytics methods
    
    def get_analytics(
        self,
        user_id: Optional[str] = None,
        from_date: Optional[datetime] = None,
        to_date: Optional[datetime] = None,
        domain: Optional[str] = None,
        granularity: str = "day",
    ) -> Analytics:
        """Get analytics for a user."""
        if not user_id and self._user:
            user_id = self._user.user_id
        
        if not user_id:
            raise ValueError("User ID required")
        
        params = {"granularity": granularity}
        
        if from_date:
            params["from_date"] = from_date.isoformat()
        if to_date:
            params["to_date"] = to_date.isoformat()
        if domain:
            params["domain"] = domain
        
        response = self._request("GET", f"/users/{user_id}/analytics", params=params)
        return Analytics(**response)
    
    def get_leaderboard(
        self,
        domain: str,
        metric: str = "accuracy",
        period: str = "week",
        limit: int = 10,
    ) -> Dict:
        """Get the global leaderboard."""
        params = {
            "domain": domain,
            "metric": metric,
            "period": period,
            "limit": limit,
        }
        
        return self._request("GET", "/analytics/leaderboard", params=params)
    
    # Data export methods
    
    def export_data(
        self,
        format: str = "json",
        include: List[str] = None,
        from_date: Optional[datetime] = None,
        to_date: Optional[datetime] = None,
    ) -> Union[Dict, bytes]:
        """Export user data."""
        if not self._user:
            raise AuthenticationError("Must be logged in to export data")
        
        params = {
            "format": format,
            "include": ",".join(include) if include else "sessions,trials,analytics",
        }
        
        if from_date:
            params["from_date"] = from_date.isoformat()
        if to_date:
            params["to_date"] = to_date.isoformat()
        
        response = self.session.get(
            urljoin(self.base_url, f"/users/{self._user.user_id}/export"),
            params=params,
            headers={"Authorization": f"Bearer {self._access_token}"},
            timeout=self.timeout,
        )
        
        response.raise_for_status()
        
        if format == "json":
            return response.json()
        else:
            return response.content
    
    # Context manager support
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.session.close()
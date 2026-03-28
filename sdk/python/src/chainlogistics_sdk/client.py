"""Main ChainLogistics API client."""

import json
from typing import Any, Dict, Optional, Tuple, TypeVar, Union
from urllib.parse import urljoin

import requests
from requests import Response, Session

from .config import Config
from .exceptions import (
    ApiError,
    AuthenticationError,
    ChainLogisticsError,
    ConfigError,
    NetworkError,
    NotFoundError,
    RateLimitError,
    TimeoutError,
    ValidationError,
)
from .models import (
    DbHealthResponse,
    EventListQuery,
    GlobalStats,
    HealthResponse,
    NewTrackingEvent,
    PaginationMeta,
    Product,
    ProductListQuery,
    TrackingEvent,
)
from .services.events import EventsService
from .services.products import ProductsService
from .services.stats import StatsService

T = TypeVar("T")


class ChainLogisticsClient:
    """Main ChainLogistics API client."""
    
    def __init__(self, config: Config):
        """Initialize the client.
        
        Args:
            config: Configuration for the client
        """
        self.config = config
        self.session = Session()
        self.session.headers.update(
            {
                "Authorization": f"Bearer {config.api_key}",
                "User-Agent": config.user_agent,
                "Content-Type": "application/json",
                "Accept": "application/json",
            }
        )
        
        # Initialize services
        self.products = ProductsService(self)
        self.events = EventsService(self)
        self.stats = StatsService(self)
    
    def _build_url(self, path: str) -> str:
        """Build full URL from path."""
        return urljoin(self.config.base_url.rstrip("/") + "/", path.lstrip("/"))
    
    def _handle_response(self, response: Response) -> Any:
        """Handle HTTP response and raise appropriate exceptions."""
        try:
            response.raise_for_status()
        except requests.exceptions.HTTPError as e:
            status_code = response.status_code
            
            try:
                error_data = response.json()
                message = error_data.get("error", response.text)
            except (ValueError, KeyError):
                message = response.text
            
            # Map status codes to appropriate exceptions
            if status_code == 401:
                raise AuthenticationError(message, status_code)
            elif status_code == 429:
                raise RateLimitError(message, status_code)
            elif status_code == 404:
                raise NotFoundError(message, status_code)
            elif 400 <= status_code < 500:
                raise ValidationError(message, status_code)
            elif 500 <= status_code < 600:
                raise ApiError(message, status_code)
            else:
                raise ApiError(message, status_code)
        
        # Parse JSON response
        try:
            return response.json()
        except ValueError as e:
            if response.text:
                return response.text
            raise ApiError(f"Invalid JSON response: {e}")
    
    def _request(
        self,
        method: str,
        path: str,
        params: Optional[Dict[str, Any]] = None,
        data: Optional[Dict[str, Any]] = None,
        timeout: Optional[int] = None,
    ) -> Any:
        """Make an HTTP request."""
        url = self._build_url(path)
        timeout = timeout or self.config.timeout
        
        try:
            response = self.session.request(
                method=method,
                url=url,
                params=params,
                json=data,
                timeout=timeout,
            )
            return self._handle_response(response)
        except requests.exceptions.Timeout as e:
            raise TimeoutError(f"Request timeout: {e}")
        except requests.exceptions.ConnectionError as e:
            raise NetworkError(f"Connection error: {e}")
        except requests.exceptions.RequestException as e:
            raise NetworkError(f"Network error: {e}")
    
    def get(
        self,
        path: str,
        params: Optional[Dict[str, Any]] = None,
        timeout: Optional[int] = None,
    ) -> Any:
        """Make a GET request."""
        return self._request("GET", path, params=params, timeout=timeout)
    
    def post(
        self,
        path: str,
        data: Optional[Dict[str, Any]] = None,
        timeout: Optional[int] = None,
    ) -> Any:
        """Make a POST request."""
        return self._request("POST", path, data=data, timeout=timeout)
    
    def put(
        self,
        path: str,
        data: Optional[Dict[str, Any]] = None,
        timeout: Optional[int] = None,
    ) -> Any:
        """Make a PUT request."""
        return self._request("PUT", path, data=data, timeout=timeout)
    
    def delete(
        self,
        path: str,
        timeout: Optional[int] = None,
    ) -> Any:
        """Make a DELETE request."""
        return self._request("DELETE", path, timeout=timeout)
    
    def health_check(self) -> HealthResponse:
        """Perform a health check."""
        data = self.get("health")
        return HealthResponse(**data)
    
    def db_health_check(self) -> DbHealthResponse:
        """Perform a database health check."""
        data = self.get("health/db")
        return DbHealthResponse(**data)
    
    def close(self) -> None:
        """Close the HTTP session."""
        self.session.close()
    
    def __enter__(self):
        """Context manager entry."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()

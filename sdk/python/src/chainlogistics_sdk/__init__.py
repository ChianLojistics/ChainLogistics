"""ChainLogistics Python SDK

This SDK provides a convenient interface for interacting with the ChainLogistics API.
"""

from .client import ChainLogisticsClient
from .config import Config
from .exceptions import (
    ChainLogisticsError,
    ApiError,
    AuthenticationError,
    RateLimitError,
    NotFoundError,
    ValidationError,
    ConfigError,
)
from .models import (
    Product,
    NewProduct,
    UpdateProduct,
    TrackingEvent,
    NewTrackingEvent,
    User,
    ApiKey,
    ApiKeyTier,
    Webhook,
    ProductStats,
    GlobalStats,
    HealthResponse,
    DbHealthResponse,
    ProductListQuery,
    EventListQuery,
    PaginationMeta,
)

__version__ = "1.0.0"
__author__ = "ChainLogistics Team"
__email__ = "support@chainlogistics.io"

__all__ = [
    # Main client
    "ChainLogisticsClient",
    "Config",
    # Exceptions
    "ChainLogisticsError",
    "ApiError",
    "AuthenticationError",
    "RateLimitError",
    "NotFoundError",
    "ValidationError",
    "ConfigError",
    # Models
    "Product",
    "NewProduct",
    "UpdateProduct",
    "TrackingEvent",
    "NewTrackingEvent",
    "User",
    "ApiKey",
    "ApiKeyTier",
    "Webhook",
    "ProductStats",
    "GlobalStats",
    "HealthResponse",
    "DbHealthResponse",
    "ProductListQuery",
    "EventListQuery",
    "PaginationMeta",
]

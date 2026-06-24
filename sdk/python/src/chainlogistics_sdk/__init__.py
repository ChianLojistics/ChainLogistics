"""ChainLogistics Python SDK

This SDK provides a convenient interface for interacting with the ChainLogistics API.
"""

from .client import ChainLogisticsClient
from .config import Config
from .storage import (
    ContentStore,
    StorageBackend,
    StorageBridgeConfig,
    StorageService,
    StorageUploadResult,
    hash_content,
    MAX_CONTENT_SIZE,
)
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
from . import ring_signature
from .ring_signature import (
    KeyPair,
    RingSignature,
    sign,
    verify,
    aggregate_ring,
)

__version__ = "1.0.0"
__author__ = "ChainLogistics Team"
__email__ = "support@chainlogistics.io"

__all__ = [
    # Main client
    "ChainLogisticsClient",
    "Config",
    # Storage
    "StorageService",
    "StorageBackend",
    "StorageBridgeConfig",
    "ContentStore",
    "StorageUploadResult",
    "hash_content",
    "MAX_CONTENT_SIZE",
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
    # Ring signatures (privacy-preserving audit trail)
    "ring_signature",
    "KeyPair",
    "RingSignature",
    "sign",
    "verify",
    "aggregate_ring",
]

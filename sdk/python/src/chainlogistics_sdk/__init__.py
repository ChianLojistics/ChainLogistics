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
from . import ring_signature
from .storage import (
    MAX_FILE_BYTES,
    StorageBackend,
    StorageBridge,
    StorageBridgeConfig,
    UploadResult,
)
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
    # Decentralized storage bridge
    "MAX_FILE_BYTES",
    "StorageBackend",
    "StorageBridge",
    "StorageBridgeConfig",
    "UploadResult",
]

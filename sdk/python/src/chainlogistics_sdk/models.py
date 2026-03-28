"""Data models for the ChainLogistics SDK."""

from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Optional
from uuid import UUID

from pydantic import BaseModel, Field


class ApiKeyTier(str, Enum):
    """API key tier levels."""
    BASIC = "Basic"
    STANDARD = "Standard"
    PREMIUM = "Premium"
    ENTERPRISE = "Enterprise"


class Product(BaseModel):
    """Product model representing a supply chain product."""
    id: str
    name: str
    description: str
    origin_location: str
    category: str
    tags: List[str]
    certifications: List[str]
    media_hashes: List[str]
    custom_fields: Dict[str, Any]
    owner_address: str
    is_active: bool
    created_at: datetime
    updated_at: datetime
    created_by: str
    updated_by: str


class NewProduct(BaseModel):
    """New product for creation requests."""
    id: str
    name: str
    description: str
    origin_location: str
    category: str
    tags: List[str]
    certifications: List[str]
    media_hashes: List[str]
    custom_fields: Dict[str, Any]
    owner_address: str
    created_by: str


class UpdateProduct(BaseModel):
    """Product update request."""
    name: Optional[str] = None
    description: Optional[str] = None
    origin_location: Optional[str] = None
    category: Optional[str] = None
    tags: Optional[List[str]] = None
    certifications: Optional[List[str]] = None
    media_hashes: Optional[List[str]] = None
    custom_fields: Optional[Dict[str, Any]] = None
    is_active: Optional[bool] = None
    updated_by: str


class TrackingEvent(BaseModel):
    """Tracking event model."""
    id: int
    product_id: str
    actor_address: str
    timestamp: datetime
    event_type: str
    location: str
    data_hash: str
    note: str
    metadata: Dict[str, Any]
    created_at: datetime


class NewTrackingEvent(BaseModel):
    """New tracking event for creation requests."""
    product_id: str
    actor_address: str
    timestamp: datetime
    event_type: str
    location: str
    data_hash: str
    note: str
    metadata: Dict[str, Any]


class User(BaseModel):
    """User model."""
    id: UUID
    email: str
    stellar_address: Optional[str] = None
    api_key: Optional[str] = None
    api_key_hash: Optional[str] = None
    is_active: bool
    is_admin: bool
    created_at: datetime
    updated_at: datetime
    last_login_at: Optional[datetime] = None


class ApiKey(BaseModel):
    """API key model."""
    id: UUID
    user_id: UUID
    key_hash: str
    name: str
    tier: ApiKeyTier
    rate_limit_per_minute: int
    is_active: bool
    expires_at: Optional[datetime] = None
    last_used_at: Optional[datetime] = None
    created_at: datetime


class NewApiKey(BaseModel):
    """New API key for creation."""
    user_id: UUID
    key_hash: str
    name: str
    tier: ApiKeyTier
    rate_limit_per_minute: int
    expires_at: Optional[datetime] = None


class Webhook(BaseModel):
    """Webhook model."""
    id: UUID
    user_id: UUID
    url: str
    secret: str
    events: List[str]
    is_active: bool
    created_at: datetime
    updated_at: datetime


class ProductStats(BaseModel):
    """Product statistics."""
    product_id: str
    event_count: int
    is_active: bool
    last_event_at: Optional[datetime] = None
    last_event_type: Optional[str] = None


class GlobalStats(BaseModel):
    """Global statistics."""
    total_products: int
    active_products: int
    total_events: int
    total_users: int
    active_api_keys: int


class HealthResponse(BaseModel):
    """Health check response."""
    status: str
    timestamp: datetime
    service: str


class DbHealthResponse(BaseModel):
    """Database health response."""
    status: str
    database: str
    timestamp: datetime


class ErrorResponse(BaseModel):
    """API error response."""
    error: str
    status: int


class PaginationMeta(BaseModel):
    """Pagination metadata."""
    total: int
    offset: int
    limit: int


class ProductListQuery(BaseModel):
    """Product list query parameters."""
    offset: Optional[int] = None
    limit: Optional[int] = None
    owner_address: Optional[str] = None
    category: Optional[str] = None
    is_active: Optional[bool] = None
    search: Optional[str] = None


class EventListQuery(BaseModel):
    """Event list query parameters."""
    offset: Optional[int] = None
    limit: Optional[int] = None
    product_id: Optional[str] = None
    event_type: Optional[str] = None


class ProductListResponse(BaseModel):
    """Product list response."""
    products: List[Product]
    total: int
    offset: int
    limit: int


class EventListResponse(BaseModel):
    """Event list response."""
    events: List[TrackingEvent]
    total: int
    offset: int
    limit: int

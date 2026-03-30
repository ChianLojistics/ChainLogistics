"""Events service for managing tracking events."""

from typing import Optional, Tuple

from ..client import ChainLogisticsClient
from ..models import (
    EventListQuery,
    NewTrackingEvent,
    PaginationMeta,
    TrackingEvent,
)


class EventsService:
    """Service for managing tracking events."""
    
    def __init__(self, client: ChainLogisticsClient):
        """Initialize the service.
        
        Args:
            client: ChainLogistics client instance
        """
        self.client = client
    
    def list(
        self,
        query: Optional[EventListQuery] = None,
    ) -> Tuple[list[TrackingEvent], PaginationMeta]:
        """List events with optional filtering.
        
        Args:
            query: Query parameters for filtering (product_id is required)
            
        Returns:
            Tuple of (events list, pagination metadata)
        """
        if not query or not query.product_id:
            from ..exceptions import ValidationError
            raise ValidationError("product_id is required for event listing")
        
        params = {}
        if query.offset is not None:
            params["offset"] = query.offset
        if query.limit is not None:
            params["limit"] = query.limit
        if query.product_id:
            params["product_id"] = query.product_id
        if query.event_type:
            params["event_type"] = query.event_type
        
        data = self.client.get("api/v1/events", params=params)
        
        events = [TrackingEvent(**item) for item in data["events"]]
        pagination = PaginationMeta(
            total=data["total"],
            offset=data["offset"],
            limit=data["limit"],
        )
        
        return events, pagination
    
    def get(self, event_id: int) -> TrackingEvent:
        """Get a specific event by ID.
        
        Args:
            event_id: Event ID
            
        Returns:
            Event details
        """
        data = self.client.get(f"api/v1/events/{event_id}")
        return TrackingEvent(**data)
    
    def create(self, event: NewTrackingEvent) -> TrackingEvent:
        """Create a new tracking event.
        
        Args:
            event: Event data to create
            
        Returns:
            Created event
        """
        data = self.client.post("api/v1/admin/events", data=event.dict())
        return TrackingEvent(**data)
    
    def list_by_product(
        self,
        product_id: str,
        offset: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> Tuple[list[TrackingEvent], PaginationMeta]:
        """List events for a specific product.
        
        Args:
            product_id: Product ID
            offset: Pagination offset
            limit: Maximum number of results
            
        Returns:
            Tuple of (events list, pagination metadata)
        """
        query = EventListQuery(
            offset=offset,
            limit=limit,
            product_id=product_id,
        )
        return self.list(query)
    
    def list_by_product_and_type(
        self,
        product_id: str,
        event_type: str,
        offset: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> Tuple[list[TrackingEvent], PaginationMeta]:
        """List events for a specific product by event type.
        
        Args:
            product_id: Product ID
            event_type: Event type
            offset: Pagination offset
            limit: Maximum number of results
            
        Returns:
            Tuple of (events list, pagination metadata)
        """
        query = EventListQuery(
            offset=offset,
            limit=limit,
            product_id=product_id,
            event_type=event_type,
        )
        return self.list(query)
    
    def get_all_for_product(self, product_id: str) -> list[TrackingEvent]:
        """Get all events for a product.
        
        Args:
            product_id: Product ID
            
        Returns:
            List of all events for the product
        """
        events, _ = self.list_by_product(product_id)
        return events
    
    def get_by_type_for_product(
        self,
        product_id: str,
        event_type: str,
    ) -> list[TrackingEvent]:
        """Get events of a specific type for a product.
        
        Args:
            product_id: Product ID
            event_type: Event type
            
        Returns:
            List of events of the specified type for the product
        """
        events, _ = self.list_by_product_and_type(product_id, event_type)
        return events

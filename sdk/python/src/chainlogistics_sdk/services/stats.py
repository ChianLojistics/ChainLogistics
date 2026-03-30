"""Stats service for accessing statistics and analytics."""

from ..client import ChainLogisticsClient
from ..models import DbHealthResponse, GlobalStats, HealthResponse


class StatsService:
    """Service for accessing statistics and analytics."""
    
    def __init__(self, client: ChainLogisticsClient):
        """Initialize the service.
        
        Args:
            client: ChainLogistics client instance
        """
        self.client = client
    
    def get_global(self) -> GlobalStats:
        """Get global statistics.
        
        Returns:
            Global statistics
        """
        data = self.client.get("api/v1/stats")
        return GlobalStats(**data)
    
    def health(self) -> HealthResponse:
        """Get system health status.
        
        Returns:
            Health response
        """
        data = self.client.get("health")
        return HealthResponse(**data)
    
    def db_health(self) -> DbHealthResponse:
        """Get database health status.
        
        Returns:
            Database health response
        """
        data = self.client.get("health/db")
        return DbHealthResponse(**data)

"""Exception classes for the ChainLogistics SDK."""


class ChainLogisticsError(Exception):
    """Base exception for all ChainLogistics SDK errors."""
    
    def __init__(self, message: str, status_code: int = None):
        super().__init__(message)
        self.message = message
        self.status_code = status_code
    
    def __str__(self) -> str:
        if self.status_code:
            return f"HTTP {self.status_code}: {self.message}"
        return self.message
    
    def is_client_error(self) -> bool:
        """Check if this is a client error (4xx)."""
        return self.status_code is not None and 400 <= self.status_code < 500
    
    def is_server_error(self) -> bool:
        """Check if this is a server error (5xx)."""
        return self.status_code is not None and 500 <= self.status_code < 600
    
    def is_retryable(self) -> bool:
        """Check if this error is retryable."""
        return isinstance(self, (RateLimitError, ApiError)) and self.is_server_error()


class ApiError(ChainLogisticsError):
    """General API error."""
    pass


class AuthenticationError(ChainLogisticsError):
    """Authentication failed."""
    pass


class RateLimitError(ChainLogisticsError):
    """Rate limit exceeded."""
    pass


class NotFoundError(ChainLogisticsError):
    """Resource not found."""
    pass


class ValidationError(ChainLogisticsError):
    """Validation error."""
    pass


class ConfigError(ChainLogisticsError):
    """Configuration error."""
    pass


class NetworkError(ChainLogisticsError):
    """Network-related error."""
    pass


class TimeoutError(ChainLogisticsError):
    """Request timeout error."""
    pass

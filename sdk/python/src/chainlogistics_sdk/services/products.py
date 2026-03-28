"""Products service for managing products."""

from typing import Optional, Tuple

from ..client import ChainLogisticsClient
from ..models import (
    NewProduct,
    PaginationMeta,
    Product,
    ProductListQuery,
    UpdateProduct,
)


class ProductsService:
    """Service for managing products."""
    
    def __init__(self, client: ChainLogisticsClient):
        """Initialize the service.
        
        Args:
            client: ChainLogistics client instance
        """
        self.client = client
    
    def list(
        self,
        query: Optional[ProductListQuery] = None,
    ) -> Tuple[list[Product], PaginationMeta]:
        """List products with optional filtering.
        
        Args:
            query: Query parameters for filtering
            
        Returns:
            Tuple of (products list, pagination metadata)
        """
        params = {}
        if query:
            if query.offset is not None:
                params["offset"] = query.offset
            if query.limit is not None:
                params["limit"] = query.limit
            if query.owner_address:
                params["owner_address"] = query.owner_address
            if query.category:
                params["category"] = query.category
            if query.is_active is not None:
                params["is_active"] = query.is_active
            if query.search:
                params["search"] = query.search
        
        data = self.client.get("api/v1/products", params=params)
        
        products = [Product(**item) for item in data["products"]]
        pagination = PaginationMeta(
            total=data["total"],
            offset=data["offset"],
            limit=data["limit"],
        )
        
        return products, pagination
    
    def get(self, product_id: str) -> Product:
        """Get a specific product by ID.
        
        Args:
            product_id: Product ID
            
        Returns:
            Product details
        """
        data = self.client.get(f"api/v1/products/{product_id}")
        return Product(**data)
    
    def create(self, product: NewProduct) -> Product:
        """Create a new product.
        
        Args:
            product: Product data to create
            
        Returns:
            Created product
        """
        data = self.client.post("api/v1/admin/products", data=product.dict())
        return Product(**data)
    
    def update(self, product_id: str, product: UpdateProduct) -> Product:
        """Update an existing product.
        
        Args:
            product_id: Product ID
            product: Updated product data
            
        Returns:
            Updated product
        """
        data = self.client.put(f"api/v1/admin/products/{product_id}", data=product.dict())
        return Product(**data)
    
    def delete(self, product_id: str) -> None:
        """Delete a product.
        
        Args:
            product_id: Product ID
        """
        self.client.delete(f"api/v1/admin/products/{product_id}")
    
    def search(self, query: str, limit: Optional[int] = None) -> list[Product]:
        """Search products by text query.
        
        Args:
            query: Search query
            limit: Maximum number of results
            
        Returns:
            List of matching products
        """
        params = {"search": query}
        if limit is not None:
            params["limit"] = limit
        
        data = self.client.get("api/v1/products", params=params)
        return [Product(**item) for item in data["products"]]
    
    def list_by_owner(
        self,
        owner_address: str,
        offset: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> Tuple[list[Product], PaginationMeta]:
        """Get products by owner address.
        
        Args:
            owner_address: Owner address
            offset: Pagination offset
            limit: Maximum number of results
            
        Returns:
            Tuple of (products list, pagination metadata)
        """
        query = ProductListQuery(
            offset=offset,
            limit=limit,
            owner_address=owner_address,
        )
        return self.list(query)
    
    def list_by_category(
        self,
        category: str,
        offset: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> Tuple[list[Product], PaginationMeta]:
        """Get products by category.
        
        Args:
            category: Product category
            offset: Pagination offset
            limit: Maximum number of results
            
        Returns:
            Tuple of (products list, pagination metadata)
        """
        query = ProductListQuery(
            offset=offset,
            limit=limit,
            category=category,
        )
        return self.list(query)
    
    def list_active(
        self,
        offset: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> Tuple[list[Product], PaginationMeta]:
        """Get only active products.
        
        Args:
            offset: Pagination offset
            limit: Maximum number of results
            
        Returns:
            Tuple of (products list, pagination metadata)
        """
        query = ProductListQuery(
            offset=offset,
            limit=limit,
            is_active=True,
        )
        return self.list(query)

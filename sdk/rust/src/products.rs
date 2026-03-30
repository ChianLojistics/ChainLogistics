use crate::{
    client::HttpClient,
    models::{Product, NewProduct, UpdateProduct, ProductListQuery, PaginationMeta},
    Config, Result,
};

/// Service for managing products
#[derive(Debug, Clone)]
pub struct ProductsService {
    client: HttpClient,
}

impl ProductsService {
    pub(crate) fn new(client: reqwest::Client, config: Config) -> Self {
        Self {
            client: HttpClient::new(client, config),
        }
    }

    /// List products with optional filtering
    pub async fn list(&self, query: Option<ProductListQuery>) -> Result<(Vec<Product>, PaginationMeta)> {
        let mut request = self.client.get("api/v1/products");

        if let Some(q) = query {
            if let Some(offset) = q.offset {
                request = request.query(&[("offset", offset)]);
            }
            if let Some(limit) = q.limit {
                request = request.query(&[("limit", limit)]);
            }
            if let Some(owner) = q.owner_address {
                request = request.query(&[("owner_address", owner)]);
            }
            if let Some(category) = q.category {
                request = request.query(&[("category", category)]);
            }
            if let Some(is_active) = q.is_active {
                request = request.query(&[("is_active", is_active)]);
            }
            if let Some(search) = q.search {
                request = request.query(&[("search", search)]);
            }
        }

        #[derive(serde::Deserialize)]
        struct ProductListResponse {
            products: Vec<Product>,
            total: i64,
            offset: i64,
            limit: i64,
        }

        let response: ProductListResponse = self.client.execute(request).await?;
        let pagination = PaginationMeta {
            total: response.total,
            offset: response.offset,
            limit: response.limit,
        };

        Ok((response.products, pagination))
    }

    /// Get a specific product by ID
    pub async fn get(&self, id: &str) -> Result<Product> {
        let request = self.client.get(&format!("api/v1/products/{}", id));
        self.client.execute(request).await
    }

    /// Create a new product
    pub async fn create(&self, product: &NewProduct) -> Result<Product> {
        let request = self.client.post("api/v1/admin/products");
        self.client.execute_with_body(request, product).await
    }

    /// Update an existing product
    pub async fn update(&self, id: &str, product: &UpdateProduct) -> Result<Product> {
        let request = self.client.put(&format!("api/v1/admin/products/{}", id));
        self.client.execute_with_body(request, product).await
    }

    /// Delete a product
    pub async fn delete(&self, id: &str) -> Result<()> {
        let request = self.client.delete(&format!("api/v1/admin/products/{}", id));
        self.client.execute_no_body(request).await
    }

    /// Search products by text query
    pub async fn search(&self, query: &str, limit: Option<i64>) -> Result<Vec<Product>> {
        let mut request = self.client.get("api/v1/products");
        request = request.query(&[("search", query)]);
        
        if let Some(limit) = limit {
            request = request.query(&[("limit", limit)]);
        }

        #[derive(serde::Deserialize)]
        struct ProductListResponse {
            products: Vec<Product>,
            total: i64,
            offset: i64,
            limit: i64,
        }

        let response: ProductListResponse = self.client.execute(request).await?;
        Ok(response.products)
    }

    /// Get products by owner address
    pub async fn list_by_owner(
        &self,
        owner_address: &str,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<Product>, PaginationMeta)> {
        let query = ProductListQuery {
            offset,
            limit,
            owner_address: Some(owner_address.to_string()),
            category: None,
            is_active: None,
            search: None,
        };
        self.list(Some(query)).await
    }

    /// Get products by category
    pub async fn list_by_category(
        &self,
        category: &str,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<Product>, PaginationMeta)> {
        let query = ProductListQuery {
            offset,
            limit,
            owner_address: None,
            category: Some(category.to_string()),
            is_active: None,
            search: None,
        };
        self.list(Some(query)).await
    }

    /// Get only active products
    pub async fn list_active(
        &self,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<Product>, PaginationMeta)> {
        let query = ProductListQuery {
            offset,
            limit,
            owner_address: None,
            category: None,
            is_active: Some(true),
            search: None,
        };
        self.list(Some(query)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, ChainLogisticsClient};

    #[tokio::test]
    async fn test_product_service() {
        let config = Config::new("test-key").with_base_url("http://localhost:3001");
        let client = ChainLogisticsClient::new(config).unwrap();
        let products = client.products();

        // These would require a running server for actual testing
        // let result = products.list(None).await;
        // assert!(result.is_ok());
    }

    #[test]
    fn test_query_building() {
        let query = ProductListQuery {
            offset: Some(10),
            limit: Some(20),
            owner_address: Some("GABC123".to_string()),
            category: Some("Electronics".to_string()),
            is_active: Some(true),
            search: Some("test".to_string()),
        };

        assert_eq!(query.offset, Some(10));
        assert_eq!(query.limit, Some(20));
        assert_eq!(query.owner_address, Some("GABC123".to_string()));
        assert_eq!(query.category, Some("Electronics".to_string()));
        assert_eq!(query.is_active, Some(true));
        assert_eq!(query.search, Some("test".to_string()));
    }
}

"""use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country_code: String,
}
"""
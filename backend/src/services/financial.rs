use sqlx::PgPool;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub user_id: String,
    pub transaction_type: String,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub blockchain_network: Option<String>,
    pub blockchain_tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub user_id: String,
    pub invoice_number: String,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub due_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancingRequest {
    pub id: String,
    pub user_id: String,
    pub financing_type: String,
    pub amount_requested: String,
    pub amount_approved: Option<String>,
    pub status: String,
    pub interest_rate: Option<String>,
}

pub struct FinancialService {
    pool: PgPool,
}

impl FinancialService {
    pub fn new(pool: PgPool) -> Self {
        FinancialService { pool }
    }

    pub async fn create_transaction(
        &self,
        user_id: &str,
        transaction_type: &str,
        amount: &str,
        currency: &str,
    ) -> Result<Transaction, String> {
        let id = Uuid::new_v4().to_string();

        let result = sqlx::query_as::<_, Transaction>(
            "INSERT INTO transactions (id, user_id, transaction_type, amount, currency, status) 
             VALUES ($1, $2, $3, $4, $5, 'pending') 
             RETURNING id, user_id, transaction_type, amount, currency, status, blockchain_network, blockchain_tx_hash"
        )
        .bind(&id)
        .bind(user_id)
        .bind(transaction_type)
        .bind(amount)
        .bind(currency)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result)
    }

    pub async fn create_invoice(
        &self,
        user_id: &str,
        amount: &str,
        due_date: &str,
    ) -> Result<Invoice, String> {
        let id = Uuid::new_v4().to_string();
        let invoice_number = format!("INV-{}", chrono::Utc::now().timestamp());

        let result = sqlx::query_as::<_, Invoice>(
            "INSERT INTO invoices (id, user_id, invoice_number, amount, currency, status, due_date) 
             VALUES ($1, $2, $3, $4, 'USD', 'draft', $5) 
             RETURNING id, user_id, invoice_number, amount, currency, status, due_date"
        )
        .bind(&id)
        .bind(user_id)
        .bind(&invoice_number)
        .bind(amount)
        .bind(due_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result)
    }

    pub async fn request_financing(
        &self,
        user_id: &str,
        financing_type: &str,
        amount: &str,
    ) -> Result<FinancingRequest, String> {
        let id = Uuid::new_v4().to_string();

        let result = sqlx::query_as::<_, FinancingRequest>(
            "INSERT INTO financing_requests (id, user_id, financing_type, amount_requested, status) 
             VALUES ($1, $2, $3, $4, 'pending') 
             RETURNING id, user_id, financing_type, amount_requested, amount_approved, status, interest_rate"
        )
        .bind(&id)
        .bind(user_id)
        .bind(financing_type)
        .bind(amount)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result)
    }

    pub async fn get_transaction(&self, id: &str) -> Result<Transaction, String> {
        sqlx::query_as::<_, Transaction>(
            "SELECT id, user_id, transaction_type, amount, currency, status, blockchain_network, blockchain_tx_hash 
             FROM transactions WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn list_user_transactions(&self, user_id: &str) -> Result<Vec<Transaction>, String> {
        sqlx::query_as::<_, Transaction>(
            "SELECT id, user_id, transaction_type, amount, currency, status, blockchain_network, blockchain_tx_hash 
             FROM transactions WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn update_transaction_status(
        &self,
        id: &str,
        status: &str,
        blockchain_tx_hash: Option<&str>,
    ) -> Result<Transaction, String> {
        sqlx::query_as::<_, Transaction>(
            "UPDATE transactions SET status = $1, blockchain_tx_hash = $2, updated_at = CURRENT_TIMESTAMP 
             WHERE id = $3 
             RETURNING id, user_id, transaction_type, amount, currency, status, blockchain_network, blockchain_tx_hash"
        )
        .bind(status)
        .bind(blockchain_tx_hash)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
    }
}

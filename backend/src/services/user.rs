use crate::database::UserRepository;
use crate::models::{AppError, NewUser, User, UserRole};
use async_trait::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserService {
    pub(crate) pool: PgPool,
    pub(crate) encryption_key: String,
}

impl UserService {
    pub fn new(pool: PgPool, encryption_key: String) -> Self {
        Self {
            pool,
            encryption_key,
        }
    }

    pub async fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
        hash(password, DEFAULT_COST)
    }

    pub async fn generate_api_key() -> String {
        format!("cl_{}", uuid::Uuid::new_v4().to_string().replace("-", ""))
    }

    fn decrypt_user(&self, user: &mut User) -> Result<(), AppError> {
        if let Ok(decrypted) = crate::utils::crypto::decrypt(&user.email, &self.encryption_key) {
            user.email = decrypted;
        }

        if let Some(addr) = &user.stellar_address {
            if let Ok(decrypted) = crate::utils::crypto::decrypt(addr, &self.encryption_key) {
                user.stellar_address = Some(decrypted);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl UserRepository for UserService {
    async fn create_user(&self, user: NewUser) -> Result<User, sqlx::Error> {
        let encrypted_email = crate::utils::crypto::encrypt(&user.email, &self.encryption_key)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let encrypted_address = if let Some(addr) = &user.stellar_address {
            Some(
                crate::utils::crypto::encrypt(addr, &self.encryption_key)
                    .map_err(|e| sqlx::Error::Protocol(e.to_string()))?,
            )
        } else {
            None
        };

        let mut created = sqlx::query_as::<User, _>(
            r#"
            INSERT INTO users (email, password_hash, stellar_address, role)
            VALUES ($1, $2, $3, $4)
            RETURNING
                id, email, password_hash, stellar_address, role,
                api_key, api_key_hash, is_active, created_at, updated_at, last_login_at
            "#,
        )
        .bind(encrypted_email)
        .bind(user.password_hash)
        .bind(encrypted_address)
        .bind(user.role)
        .fetch_one(&self.pool)
        .await?;

        let _ = self.decrypt_user(&mut created);
        Ok(created)
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let mut user = sqlx::query_as::<User, _>(
            "SELECT id, email, password_hash, stellar_address, role, api_key, api_key_hash, is_active, created_at, updated_at, last_login_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut u) = user {
            let _ = self.decrypt_user(u);
        }
        Ok(user)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let encrypted_email = crate::utils::crypto::encrypt(email, &self.encryption_key)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let mut user = sqlx::query_as::<User, _>(
            "SELECT id, email, password_hash, stellar_address, role, api_key, api_key_hash, is_active, created_at, updated_at, last_login_at FROM users WHERE email = $1",
        )
        .bind(encrypted_email)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut u) = user {
            let _ = self.decrypt_user(u);
        }
        Ok(user)
    }

    async fn get_user_by_stellar_address(
        &self,
        address: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let encrypted_address = crate::utils::crypto::encrypt(address, &self.encryption_key)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let mut user = sqlx::query_as::<User, _>(
            "SELECT id, email, password_hash, stellar_address, role, api_key, api_key_hash, is_active, created_at, updated_at, last_login_at FROM users WHERE stellar_address = $1",
        )
        .bind(encrypted_address)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut u) = user {
            let _ = self.decrypt_user(u);
        }
        Ok(user)
    }

    async fn update_user(&self, id: Uuid, user: User) -> Result<User, sqlx::Error> {
        let encrypted_email = crate::utils::crypto::encrypt(&user.email, &self.encryption_key)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let encrypted_address = if let Some(addr) = &user.stellar_address {
            Some(
                crate::utils::crypto::encrypt(addr, &self.encryption_key)
                    .map_err(|e| sqlx::Error::Protocol(e.to_string()))?,
            )
        } else {
            None
        };

        let mut updated = sqlx::query_as::<User, _>(
            r#"
            UPDATE users SET
                email = $2,
                password_hash = $3,
                stellar_address = $4,
                role = $5,
                api_key = $6,
                api_key_hash = $7,
                is_active = $8
            WHERE id = $1
            RETURNING
                id, email, password_hash, stellar_address, role,
                api_key, api_key_hash, is_active, created_at, updated_at, last_login_at
            "#,
        )
        .bind(id)
        .bind(encrypted_email)
        .bind(user.password_hash)
        .bind(encrypted_address)
        .bind(user.role)
        .bind(user.api_key)
        .bind(user.api_key_hash)
        .bind(user.is_active)
        .fetch_one(&self.pool)
        .await?;

        let _ = self.decrypt_user(&mut updated);
        Ok(updated)
    }

    async fn update_last_login(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

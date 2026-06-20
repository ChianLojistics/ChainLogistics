use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use bcrypt::{hash, DEFAULT_COST};
use crate::database::UserRepository;
use crate::models::{User, NewUser, UserRole, AppError};

pub struct UserService {
    pub(crate) pool: PgPool,
    pub(crate) encryption_key: String,
}

impl UserService {
    pub fn new(pool: PgPool, encryption_key: String) -> Self {
        Self { pool, encryption_key }
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

        let mut created = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, password_hash, stellar_address, role)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            encrypted_email,
            user.password_hash,
            encrypted_address,
            user.role as UserRole
        )
        .fetch_one(&self.pool)
        .await?;

        let _ = self.decrypt_user(&mut created);
        Ok(created)
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let mut user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id
        )
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

        let mut user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            encrypted_email
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut u) = user {
            let _ = self.decrypt_user(u);
        }
        Ok(user)
    }

    async fn get_user_by_stellar_address(&self, address: &str) -> Result<Option<User>, sqlx::Error> {
        let encrypted_address = crate::utils::crypto::encrypt(address, &self.encryption_key)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let mut user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE stellar_address = $1",
            encrypted_address
        )
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

        let mut updated = sqlx::query_as!(
            User,
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
            RETURNING *
            "#,
            id,
            encrypted_email,
            user.password_hash,
            encrypted_address,
            user.role as UserRole,
            user.api_key,
            user.api_key_hash,
            user.is_active
        )
        .fetch_one(&self.pool)
        .await?;

        let _ = self.decrypt_user(&mut updated);
        Ok(updated)
    }

    async fn update_last_login(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET last_login_at = NOW() WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

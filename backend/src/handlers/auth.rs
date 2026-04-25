use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use crate::{AppState, error::AppError, models::{UserRole, NewUser, User}, middleware::auth::Claims, validation::{validate_email, validate_string}};
use bcrypt::verify;
use jsonwebtoken::{encode, Header, EncodingKey};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub stellar_address: Option<String>,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    validate_email(&req.email)?;

    let user = state.user_service.get_user_by_email(&req.email).await?
        .ok_or_else(|| AppError::Unauthorized)?;

    if !user.is_active {
        return Err(AppError::Unauthorized);
    }

    let is_valid = verify(&req.password, &user.password_hash)
        .map_err(|_| AppError::Internal("Password verification failed".to_string()))?;

    if !is_valid {
        return Err(AppError::Unauthorized);
    }

    // Update last login
    let _ = state.user_service.update_last_login(user.id).await;

    // Issue JWT
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration,
        role: user.role.clone(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::Internal("Token issuance failed".to_string()))?;

    Ok(Json(AuthResponse { token, user }))
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<User>, AppError> {
    validate_email(&req.email)?;
    validate_string("password", &req.password, 128)?;

    let password_hash = crate::services::UserService::hash_password(&req.password).await?;

    let new_user = NewUser {
        email: req.email,
        password_hash,
        stellar_address: req.stellar_address,
        role: req.role,
    };

    let user = state.user_service.create_user(new_user).await?;
    Ok(Json(user))
}

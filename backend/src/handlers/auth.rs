use axum::extract::State;
use super::super::{AppState, error::AppError};

pub async fn login(State(_state): State<AppState>) -> Result<(), AppError> {
    Ok(())
}

pub async fn register(State(_state): State<AppState>) -> Result<(), AppError> {
    Ok(())
}

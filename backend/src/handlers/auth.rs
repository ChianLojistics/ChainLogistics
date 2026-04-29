use super::super::{error::AppError, AppState};
use axum::extract::State;

pub async fn login(State(_state): State<AppState>) -> Result<(), AppError> {
    Ok(())
}

pub async fn register(State(_state): State<AppState>) -> Result<(), AppError> {
    Ok(())
}

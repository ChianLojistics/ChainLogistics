use axum::extract::State;
use super::super::{AppState, error::AppError};

pub async fn get_stats(State(_state): State<AppState>) -> Result<(), AppError> {
    Ok(())
}

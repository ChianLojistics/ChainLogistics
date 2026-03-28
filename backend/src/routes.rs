use axum::{Router, routing::{get, post}};
use super::AppState;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/products", get(crate::handlers::product::list_products).post(crate::handlers::product::create_product))
        .route("/products/:id", get(crate::handlers::product::get_product).put(crate::handlers::product::update_product).delete(crate::handlers::product::delete_product))
        .route("/events", get(crate::handlers::event::list_events).post(crate::handlers::event::create_event))
        .route("/events/:id", get(crate::handlers::event::get_event))
        .route("/users", post(crate::handlers::user::create_user))
        .route("/users/me", get(crate::handlers::user::get_current_user))
        .route("/auth/login", post(crate::handlers::auth::login))
        .route("/auth/register", post(crate::handlers::auth::register))
        .route("/stats", get(crate::handlers::stats::get_stats))
}
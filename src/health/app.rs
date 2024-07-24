use super::handlers::health;
use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/health", get(health))
}

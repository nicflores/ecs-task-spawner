use axum::Json;

use super::models::Health;

pub async fn health() -> Json<Health> {
    let health_status = Health {
        status: "ok".to_string(),
    };
    Json(health_status)
}

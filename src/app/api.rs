use crate::spawner::models::TaskSpawner;

use super::handlers::spawn_task;

use axum::{routing::post, Router};

pub fn router<T: TaskSpawner>(state: T) -> Router {
    Router::new()
        .route("/spawn-worker", post(spawn_task::<T>))
        .with_state(state)
}

use crate::ecs::models::EcsTaskRepo;

use super::handlers::{get_task_family, get_tasks, spawn};

use axum::{routing::post, Router};

pub fn router<T: EcsTaskRepo>(state: T) -> Router {
    Router::new()
        .route("/spawn-worker", post(spawn::<T>))
        .route("/task-family", post(get_task_family::<T>))
        .route("/task-tag", post(get_tasks::<T>))
        .with_state(state)
}

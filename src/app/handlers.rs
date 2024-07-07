use axum::{extract::State, Json};

use crate::{error::models::AppError, spawner::models::TaskSpawner};

use super::models::{TaskRequest, TaskResponse};

pub async fn spawn_task<T: TaskSpawner>(
    State(state): State<T>,
    Json(payload): Json<TaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let task_id = state.spawn_task(payload).await?;
    Ok(Json(TaskResponse { task_id }))
}

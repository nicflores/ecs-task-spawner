use axum::{extract::State, Json};

use crate::{
    ecs::models::{EcsTag, EcsTaskDefinition, EcsTaskRepo, TaskFamily, TaskInfo, TaskRequest},
    errors::models::AppError,
};

// pub async fn spawn_task<T: TaskSpawner>(
//     State(state): State<T>,
//     Json(payload): Json<TaskRequest>,
// ) -> Result<Json<TaskResponse>, AppError> {
//     let task_id = state.spawn_task(payload).await?;
//     Ok(Json(TaskResponse { task_id }))
// }

pub async fn spawn<T: EcsTaskRepo>(
    State(state): State<T>,
    Json(task): Json<TaskRequest>,
) -> Result<Json<TaskInfo>, AppError> {
    let taskdef = EcsTaskDefinition::new(task)?;
    let res = state.spawn(taskdef).await?;
    Ok(Json(res))
}

pub async fn get_task_family<T: EcsTaskRepo>(
    State(state): State<T>,
    Json(task_family): Json<TaskFamily>,
) -> Result<Json<Vec<TaskInfo>>, AppError> {
    let res = state.get_task_family(task_family).await?;
    Ok(Json(res))
}

pub async fn get_tasks<T: EcsTaskRepo>(
    State(state): State<T>,
    Json(tag): Json<EcsTag>,
) -> Result<Json<Vec<TaskInfo>>, AppError> {
    let res = state.get_tasks(tag).await?;
    Ok(Json(res))
}

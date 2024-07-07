use crate::{app::models::TaskRequest, error::models::AppError};
use async_trait::async_trait;
use aws_sdk_ecs::Client as EcsClient;

#[async_trait]
pub trait TaskSpawner: Send + Sync + Clone + 'static {
    async fn spawn_task(&self, task_req: TaskRequest) -> Result<String, AppError>;
}

#[derive(Clone)]
pub struct EcsTaskSpawner {
    pub ecs_client: EcsClient,
    pub cluster_name: String,
    pub subnet_id: String,
    pub security_group_id: String,
    pub image: String,
    pub log_group: String,
    pub iam_role_arn: String,
    pub execution_role_arn: String,
}

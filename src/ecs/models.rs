use std::time::{Duration, UNIX_EPOCH};

use crate::errors::models::AppError;
use async_trait::async_trait;
use aws_sdk_ecs::{types::Task, Client as EcsClient};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// #[async_trait]
// pub trait TaskSpawner: Send + Sync + Clone + 'static {
//     async fn spawn_task(&self, task_req: TaskRequest) -> Result<String, AppError>;
// }

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

// New stuff
// To spawna a task we need the following:
// - s3 url
// - soiid
// - clientid
// - worker type

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub data_location: String,
    pub soiid: String,
    pub clientid: String,
    pub vendor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFamily {
    pub task_family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsTag {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct EcsRepo {
    pub client: EcsClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcsEnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct EcsTaskDefinition {
    pub cluster_name: String,
    pub subnet_id: String,
    pub security_group_id: String,
    pub image: String,
    pub log_group: String,
    pub iam_role_arn: String,
    pub execution_role_arn: String,
    pub tags: Vec<EcsTag>,
    pub env_vars: Vec<EcsEnvVar>,
}

impl EcsTaskDefinition {
    pub fn new(tr: TaskRequest) -> Result<Self, AppError> {
        let mut tags = Vec::new();
        tags.push(EcsTag {
            key: "soiid".to_string(),
            value: tr.soiid.clone(),
        });
        tags.push(EcsTag {
            key: "clientid".to_string(),
            value: tr.clientid.clone(),
        });
        tags.push(EcsTag {
            key: "worker_type".to_string(),
            value: tr.vendor.clone(),
        });

        // We can add more env vars to the worker task definition here.
        let mut env_vars = Vec::new();
        env_vars.push(EcsEnvVar {
            name: "APP_DATA_URL".to_string(),
            value: tr.data_location.clone(),
        });

        // NOTE: Add more vendor images here as needed.
        // Given tr.vendor determine which worker image to use.
        let image = match tr.vendor.as_str() {
            "bloomberg" => "public.ecr.aws/soi/bloomberg-worker:latest".to_string(),
            _ => return Err(AppError::UnsupportedVendor(tr.vendor)),
        };

        // TODO: Should we read these from SSM?
        let task_defn = EcsTaskDefinition {
            cluster_name: "default".to_string(),
            subnet_id: "subnet-0c8b6b6b".to_string(),
            security_group_id: "sg-0c8b6b6b".to_string(),
            image,
            log_group: "/ecs/soi-worker".to_string(),
            iam_role_arn: "arn:aws:iam::123456789012:role/ecsTaskExecutionRole".to_string(),
            execution_role_arn: "arn:aws:iam::123456789012:role/ecsTaskExecutionRole".to_string(),
            tags,
            env_vars,
        };

        Ok(task_defn)
    }
}

#[async_trait]
pub trait EcsTaskRepo: Send + Sync + Clone + 'static {
    // Later, try to return more info about the ecs task created.
    async fn spawn(&self, task: EcsTaskDefinition) -> Result<TaskInfo, AppError>;
    // Gets tasks in a given task family. Would be good to return an array of structs that contain task info and metrics.
    async fn get_task_family(&self, task_family: TaskFamily) -> Result<Vec<TaskInfo>, AppError>;
    // Return tasks with a given tag info. Would be good to return a struct contaning task info and metrics.
    async fn get_tasks(&self, tag: EcsTag) -> Result<Vec<TaskInfo>, AppError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub task_arn: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub running_duration: Option<Duration>,
    pub image: String,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub tags: Vec<EcsTag>,
}

impl From<&Task> for TaskInfo {
    fn from(task: &Task) -> Self {
        let tags = task
            .tags()
            .to_vec()
            .iter()
            .map(|tag| EcsTag {
                key: tag.key().unwrap_or_default().to_string(),
                value: tag.value().unwrap_or_default().to_string(),
            })
            .collect();

        let created_at = task.created_at().map(|t| {
            let secs = t.secs().max(0) as u64;
            let nanos = t.subsec_nanos();
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::new(secs, nanos))
        });

        let running_duration = created_at.map(|created_at| {
            Utc::now()
                .signed_duration_since(created_at)
                .to_std()
                .unwrap_or(Duration::ZERO)
        });

        let image: String = task
            .containers()
            .iter()
            .map(|containers| containers.image().unwrap_or_default().to_string())
            .collect();

        TaskInfo {
            task_arn: task.task_arn().unwrap_or_default().to_string(),
            status: task.last_status().unwrap_or_default().to_string(),
            created_at: created_at.unwrap_or_else(Utc::now),
            running_duration,
            image,
            cpu_usage: None,    // Placeholder, will be fetched from CloudWatch
            memory_usage: None, // Placeholder, will be fetched from CloudWatch
            tags,
        }
    }
}

use aws_sdk_ecs::{
    error::{BuildError, SdkError},
    operation::{register_task_definition::RegisterTaskDefinitionError, run_task::RunTaskError},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Task spawn error")]
    TaskSpawnError(String),
    #[error("AWS SDK error")]
    AwsSdkError(#[from] aws_sdk_ecs::Error),
    #[error("Validation error")]
    ValidationError(String),
    #[error("Not found error")]
    NotFoundError(String),
    #[error("Unauthorized error")]
    UnauthorizedError(String),
    #[error("Internal server error")]
    InternalServerError(String),
    #[error("Log configuration build error")]
    LogConfigurationError(#[from] BuildError),
    #[error("Register task definition error")]
    RegisterTaskDefinitionError(#[from] SdkError<RegisterTaskDefinitionError>),
    #[error("Run task error")]
    RunTaskError(#[from] SdkError<RunTaskError>),
}

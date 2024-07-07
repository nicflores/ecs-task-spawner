use super::models::AppError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

impl AppError {
    pub fn error_type(&self) -> &str {
        match self {
            AppError::TaskSpawnError(_) => "TASK_SPAWN_ERROR",
            AppError::AwsSdkError(_) => "AWS_SDK_ERROR",
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::NotFoundError(_) => "NOT_FOUND_ERROR",
            AppError::UnauthorizedError(_) => "UNAUTHORIZED_ERROR",
            AppError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
            AppError::LogConfigurationError(_) => "LOG_CONFIGURATION_ERROR",
            AppError::RegisterTaskDefinitionError(_) => "REGISTER_TASK_DEFINITION_ERROR",
            AppError::RunTaskError(_) => "RUN_TASK_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::TaskSpawnError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::AwsSdkError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NotFoundError(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::UnauthorizedError(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::LogConfigurationError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::RegisterTaskDefinitionError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::RunTaskError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": {
                "type": self.error_type(),
                "message": error_message,
            }
        }));

        (status, body).into_response()
    }
}

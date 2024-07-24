mod auth;
mod errors;
mod health;

use crate::auth::api::auth;

use std::sync::Arc;

use aws_sdk_ecs::Client as EcsClient;
use axum::middleware;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use ecs_task_spawner::app;
use ecs_task_spawner::config::models::AppConfig;
use ecs_task_spawner::ecs::models::EcsRepo;
use ecs_task_spawner::shutdown::shutdown_signal;
use tower::ServiceBuilder;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let cfg = AppConfig::new().unwrap();

    // Setup the auth layer.
    let token = Arc::new(cfg.api_key.clone());
    let auth_layer = ServiceBuilder::new()
        .layer(middleware::from_fn(move |req, next| {
            let token = token.clone();
            async move { auth(req, next, token).await }
        }))
        .into_inner();

    // Initialize ECS client
    let config = aws_config::load_from_env().await;
    let ecs_client = EcsClient::new(&config);

    // TODO: Need to terraform resources that workers will all use:
    // - security group
    // - cloudwatch log group
    // Initialize the task spawner
    // let task_spawner = EcsTaskSpawner::new(
    //     ecs_client,
    //     "cluster-name".to_string(), // Replace with your ECS cluster name
    //     "subnet-id".to_string(),    // Replace with your private subnet ID
    //     "sg-id".to_string(),        // Replace with your security group ID
    //     "ecr-image-id".to_string(), // Replace with your container image
    //     "cloud-watch-log-group".to_string(), // Replace with your CloudWatch log group
    //     "ecs-task-execution-role-arn".to_string(), // Replace with your IAM role ARN
    //     "ecs-task-execution-role-arn".to_string(), // Replace with your execution role ARN
    // );

    let ecs_repo = EcsRepo::new(ecs_client);
    let worker_api = app::api::router(ecs_repo);

    let health_api = health::app::router();

    let app = worker_api
        .layer(auth_layer)
        .merge(health_api)
        .layer(OtelInResponseLayer::default())
        .layer(OtelAxumLayer::default());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

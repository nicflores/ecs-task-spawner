use aws_sdk_ecs::Client as EcsClient;
use ecs_task_spawner::{app, spawner::models::EcsTaskSpawner};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    // Initialize ECS client
    let config = aws_config::load_from_env().await;
    let ecs_client = EcsClient::new(&config);

    // Initialize the task spawner
    let task_spawner = EcsTaskSpawner::new(
        ecs_client,
        "cluster-name".to_string(), // Replace with your ECS cluster name
        "subnet-id".to_string(),    // Replace with your private subnet ID
        "sg-id".to_string(),        // Replace with your security group ID
        "ecr-image-id".to_string(), // Replace with your container image
        "cloud-watch-log-group".to_string(), // Replace with your CloudWatch log group
        "ecs-task-execution-role-arn".to_string(), // Replace with your IAM role ARN
        "ecs-task-execution-role-arn".to_string(), // Replace with your execution role ARN
    );

    //let task_spawner = EcsTaskSpawner::new(client);
    let app = app::api::router(task_spawner);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

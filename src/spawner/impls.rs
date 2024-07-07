use super::models::{EcsTaskSpawner, TaskSpawner};
use crate::{app::models::TaskRequest, error::models::AppError};
use async_trait::async_trait;
use aws_sdk_ecs::{
    types::{
        AssignPublicIp, AwsVpcConfiguration, Compatibility, ContainerDefinition, KeyValuePair,
        LaunchType, LogConfiguration, LogDriver, NetworkConfiguration, NetworkMode,
    },
    Client as EcsClient,
};

impl EcsTaskSpawner {
    pub fn new(
        ecs_client: EcsClient,
        cluster_name: String,
        subnet_id: String,
        security_group_id: String,
        image: String,
        log_group: String,
        iam_role_arn: String,
        execution_role_arn: String,
    ) -> Self {
        EcsTaskSpawner {
            ecs_client,
            cluster_name,
            subnet_id,
            security_group_id,
            image,
            log_group,
            iam_role_arn,
            execution_role_arn,
        }
    }
}

#[async_trait]
impl TaskSpawner for EcsTaskSpawner {
    async fn spawn_task(&self, task_req: TaskRequest) -> Result<String, AppError> {
        let log_configuration = LogConfiguration::builder()
            .log_driver(LogDriver::Awslogs)
            .options("awslogs-group", &self.log_group)
            .options("awslogs-region", "us-east-1")
            .options("awslogs-stream-prefix", "ecs")
            .build()?;

        let container_definition = ContainerDefinition::builder()
            .name("my-container")
            .image(&self.image)
            .cpu(256)
            .memory(512)
            .essential(true)
            .log_configuration(log_configuration)
            .environment(
                KeyValuePair::builder()
                    .name("TASK_DURATION")
                    .value(task_req.duration.to_string())
                    .build(),
            )
            .build();

        let task_definition_response = self
            .ecs_client
            .register_task_definition()
            .family("my-task-family")
            .task_role_arn(&self.iam_role_arn)
            .execution_role_arn(&self.execution_role_arn)
            .network_mode(NetworkMode::Awsvpc)
            .requires_compatibilities(Compatibility::Fargate)
            .cpu("256")
            .memory("512")
            .container_definitions(container_definition)
            .send()
            .await?;

        let task_definition_arn = task_definition_response
            .task_definition
            .unwrap()
            .task_definition_arn
            .unwrap();

        let network_configuration = NetworkConfiguration::builder()
            .awsvpc_configuration(
                AwsVpcConfiguration::builder()
                    .subnets(&self.subnet_id)
                    .assign_public_ip(AssignPublicIp::Disabled)
                    .security_groups(&self.security_group_id)
                    .build()?,
            )
            .build();

        let response = self
            .ecs_client
            .run_task()
            .cluster(&self.cluster_name)
            .launch_type(LaunchType::Fargate)
            .task_definition(task_definition_arn)
            .network_configuration(network_configuration)
            .count(1)
            .send()
            .await?;

        let task_arn = response
            .tasks
            .unwrap()
            .first()
            .unwrap()
            .task_arn
            .as_ref()
            .unwrap()
            .clone();

        Ok(task_arn)
    }
}

use super::models::{EcsRepo, EcsTag, EcsTaskDefinition, EcsTaskRepo, TaskFamily, TaskInfo};
use crate::errors::models::AppError;
use async_trait::async_trait;
use aws_sdk_ecs::{
    types::{
        AssignPublicIp, AwsVpcConfiguration, Compatibility, ContainerDefinition, KeyValuePair,
        LaunchType, LogConfiguration, LogDriver, NetworkConfiguration, NetworkMode, Task,
    },
    Client as EcsClient,
};

impl EcsRepo {
    pub fn new(client: EcsClient) -> Self {
        EcsRepo { client }
    }
}

#[async_trait]
impl EcsTaskRepo for EcsRepo {
    async fn spawn(&self, task: EcsTaskDefinition) -> Result<TaskInfo, AppError> {
        let log_configuration = LogConfiguration::builder()
            .log_driver(LogDriver::Awslogs)
            .options("awslogs-group", task.log_group)
            .options("awslogs-region", "us-east-1")
            .options("awslogs-stream-prefix", "ecs")
            .build()?;

        let mut environment_variables = Vec::new();
        for envvar in task.env_vars.iter() {
            environment_variables.push(
                KeyValuePair::builder()
                    .name(envvar.name.clone())
                    .value(envvar.value.clone())
                    .build(),
            );
        }

        let container_definition = ContainerDefinition::builder()
            .name("my-container")
            .image(task.image)
            .cpu(256)
            .memory(512)
            .essential(true)
            .log_configuration(log_configuration)
            .set_environment(Some(environment_variables))
            .build();

        let task_definition_response = self
            .client
            .register_task_definition()
            .family("my-task-family")
            .task_role_arn(task.iam_role_arn)
            .execution_role_arn(task.execution_role_arn)
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
                    .subnets(task.subnet_id)
                    .assign_public_ip(AssignPublicIp::Disabled)
                    .security_groups(task.security_group_id)
                    .build()?,
            )
            .build();

        let response = self
            .client
            .run_task()
            .cluster(task.cluster_name)
            .launch_type(LaunchType::Fargate)
            .task_definition(task_definition_arn)
            .network_configuration(network_configuration)
            .count(1)
            .send()
            .await?;

        // TODO: handle the case where there are no tasks
        let new_task = response.tasks().first().unwrap();
        let task_info = TaskInfo::from(new_task);

        Ok(task_info)
    }

    async fn get_task_family(&self, task_family: TaskFamily) -> Result<Vec<TaskInfo>, AppError> {
        let list_tasks_response = self
            .client
            .list_tasks()
            .cluster("cluster-name".to_string())
            .send()
            .await?;

        let task_arns = list_tasks_response.task_arns().to_vec();
        if task_arns.is_empty() {
            println!("No tasks found in the cluster.");
            return Err(AppError::CustomError(
                "No tasks found in task family.".to_string(),
            ));
        }

        let describe_tasks_response = self
            .client
            .describe_tasks()
            .cluster("cluster-name".to_string())
            .set_tasks(Some(task_arns.to_vec()))
            .send()
            .await?;

        let tasks = describe_tasks_response.tasks().to_vec();

        let filtered_tasks: Vec<&Task> = tasks
            .iter()
            .filter(|task| {
                if let Some(task_definition_arn) = task.task_definition_arn() {
                    task_definition_arn.contains(task_family.task_family.as_str())
                } else {
                    false
                }
            })
            .collect();

        let task_arns: Vec<TaskInfo> = filtered_tasks
            .iter()
            .map(|task| TaskInfo::from(*task))
            .collect();

        Ok(task_arns)
    }

    // TODO: Retrun an custom struct containing task details and metrics instead of Vec<String>.
    async fn get_tasks(&self, tag: EcsTag) -> Result<Vec<TaskInfo>, AppError> {
        let list_tasks_response = self
            .client
            .list_tasks()
            .cluster("cluster-name".to_string())
            .send()
            .await?;

        let task_arns = list_tasks_response.task_arns().to_vec();
        if task_arns.is_empty() {
            return Err(AppError::CustomError(format!(
                "No tasks exist with tag: {:#?}",
                tag
            )));
        }

        let describe_tasks_response = self
            .client
            .describe_tasks()
            .cluster("cluster-name".to_string())
            .set_tasks(Some(task_arns.to_vec()))
            .send()
            .await?;

        let tasks = describe_tasks_response.tasks().to_vec();

        // warning: we unwrap here
        let filtered_tasks: Vec<TaskInfo> = tasks
            .iter()
            .filter(|task| {
                task.tags()
                    .iter()
                    .any(|t| t.key().unwrap() == tag.key && t.value().unwrap() == tag.value)
            })
            .map(|task| TaskInfo::from(task))
            .collect();

        Ok(filtered_tasks)
    }
}

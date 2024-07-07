use std::sync::Arc;

use async_trait::async_trait;

#[async_trait]
pub trait Task: Send + Sync {
    async fn run(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn get_task_definition(&self) -> String;
}

pub type ArcTask = Arc<dyn Task>;

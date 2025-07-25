use async_trait::async_trait;

#[async_trait]
pub trait Submitter: Send + Sync + 'static {
    async fn submit(
        &self,
        payload_b64: String,
        skip_preflight: bool,
    ) -> anyhow::Result<String>;
    
    /// Enable downcasting for ping functionality
    fn as_any(&self) -> &dyn std::any::Any;
}

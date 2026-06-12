use crate::server::core::routing::DynamicContract;
use crate::server::core::routing::dynamic_context::DynamicContext;
use std::sync::Arc;

#[derive(Clone)]
pub struct DynamicPipeline {
    // 💡 CHANGED: The steps are wrapped in Arc so they can be cloned into the async loop
    pub steps: Vec<Arc<dyn DynamicContract>>,
}

impl DynamicPipeline {
    pub async fn execute(&self, ctx: DynamicContext) -> Result<(), String> {
        for (index, step) in self.steps.iter().enumerate() {
            // 💡 SAFE: Clone the Arc pointer to hand ownership directly to the execution future
            let executable_step = step.clone();

            if let Err(err) = executable_step.execute(ctx.clone()).await {
                return Err(format!("Pipeline interrupted at step {}: {}", index, err));
            }
        }
        Ok(())
    }
}

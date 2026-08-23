use crate::error::{DagrError, Result};
use crate::event_store::RunId;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStepRecord {
    pub step_index: usize,
    pub step_name: String,
    pub completed: bool,
    pub compensated: bool,
}

#[async_trait]
pub trait SagaAction: Send + Sync {
    async fn forward(&self) -> Result<()>;
    async fn compensate(&self) -> Result<()>;
    fn name(&self) -> &str;
}

pub struct SagaCoordinator {
    run_id: RunId,
    actions: Vec<Box<dyn SagaAction>>,
    history: Vec<SagaStepRecord>,
}

impl SagaCoordinator {
    pub fn run_id(&self) -> &RunId {
        &self.run_id
    }

    pub fn new(run_id: RunId) -> Self {
        Self {
            run_id,
            actions: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn add_step(&mut self, action: Box<dyn SagaAction>) {
        self.actions.push(action);
    }

    pub async fn execute_all(&mut self) -> Result<()> {
        let mut executed_count = 0;
        let total_steps = self.actions.len();

        for idx in 0..total_steps {
            let step_res = self.actions[idx].forward().await;
            let step_name = self.actions[idx].name().to_string();

            match step_res {
                Ok(()) => {
                    self.history.push(SagaStepRecord {
                        step_index: idx,
                        step_name,
                        completed: true,
                        compensated: false,
                    });
                    executed_count += 1;
                }
                Err(err) => {
                    // Forward execution failed at step idx. Trigger backward compensation on all previously executed steps!
                    self.compensate_backward(executed_count).await?;
                    return Err(DagrError::Internal(format!(
                        "Saga step {} ({}) failed: {}. Backward compensation completed.",
                        idx, step_name, err
                    )));
                }
            }
        }

        Ok(())
    }

    async fn compensate_backward(&mut self, executed_count: usize) -> Result<()> {
        for idx in (0..executed_count).rev() {
            let action = &self.actions[idx];
            if let Err(comp_err) = action.compensate().await {
                return Err(DagrError::Internal(format!(
                    "Critical failure during saga backward compensation at step {} ({}): {}",
                    idx,
                    action.name(),
                    comp_err
                )));
            }
            if let Some(record) = self.history.get_mut(idx) {
                record.compensated = true;
            }
        }
        Ok(())
    }

    pub fn get_history(&self) -> &[SagaStepRecord] {
        &self.history
    }
}

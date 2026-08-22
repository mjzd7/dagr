use crate::error::{DagrError, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct BudgetContext {
    deadline: Instant,
    tokens_remaining: Arc<AtomicUsize>,
    initial_tokens: usize,
}

impl BudgetContext {
    pub fn new(max_duration: Duration, max_tokens: usize) -> Self {
        Self {
            deadline: Instant::now() + max_duration,
            tokens_remaining: Arc::new(AtomicUsize::new(max_tokens)),
            initial_tokens: max_tokens,
        }
    }

    pub fn remaining_duration(&self) -> Result<Duration> {
        let now = Instant::now();
        if now >= self.deadline {
            Err(DagrError::Internal(
                "Wall-clock execution deadline exceeded".into(),
            ))
        } else {
            Ok(self.deadline - now)
        }
    }

    pub fn deduct_tokens(&self, count: usize) -> Result<usize> {
        loop {
            let current = self.tokens_remaining.load(Ordering::SeqCst);
            if count > current {
                return Err(DagrError::Internal(format!(
                    "Token budget exceeded: requested {} tokens, only {} remaining",
                    count, current
                )));
            }
            let new_val = current - count;
            if self
                .tokens_remaining
                .compare_exchange(current, new_val, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Ok(new_val);
            }
        }
    }

    pub fn tokens_consumed(&self) -> usize {
        self.initial_tokens
            .saturating_sub(self.tokens_remaining.load(Ordering::SeqCst))
    }

    pub fn is_exhausted(&self) -> bool {
        Instant::now() >= self.deadline || self.tokens_remaining.load(Ordering::SeqCst) == 0
    }
}

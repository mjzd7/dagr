use crate::error::{DagrError, Result};
use std::sync::Mutex;
use std::time::Instant;

/// Token-per-minute (TPM) predictive token bucket rate limiter
pub struct TokenBucketRateLimiter {
    capacity_tokens: usize,
    tokens_per_second: f64,
    current_tokens: Mutex<f64>,
    last_refill: Mutex<Instant>,
}

impl TokenBucketRateLimiter {
    pub fn new(tokens_per_minute: usize) -> Self {
        let tps = tokens_per_minute as f64 / 60.0;
        Self {
            capacity_tokens: tokens_per_minute,
            tokens_per_second: tps,
            current_tokens: Mutex::new(tokens_per_minute as f64),
            last_refill: Mutex::new(Instant::now()),
        }
    }

    fn refill(&self) {
        let mut last_refill = self.last_refill.lock().unwrap();
        let mut current_tokens = self.current_tokens.lock().unwrap();
        let now = Instant::now();
        let elapsed_secs = now.duration_since(*last_refill).as_secs_f64();

        if elapsed_secs > 0.0 {
            let tokens_to_add = elapsed_secs * self.tokens_per_second;
            *current_tokens = (*current_tokens + tokens_to_add).min(self.capacity_tokens as f64);
            *last_refill = now;
        }
    }

    /// Try to acquire estimated token cost before dispatch
    pub fn try_acquire(&self, estimated_tokens: usize) -> Result<()> {
        self.refill();
        let mut current_tokens = self.current_tokens.lock().unwrap();

        if *current_tokens >= estimated_tokens as f64 {
            *current_tokens -= estimated_tokens as f64;
            Ok(())
        } else {
            Err(DagrError::Internal(format!(
                "Token rate limit exceeded (TPM): requested ~{} tokens, only ~{:.0} available in bucket",
                estimated_tokens, *current_tokens
            )))
        }
    }

    pub fn available_tokens(&self) -> usize {
        self.refill();
        let current_tokens = self.current_tokens.lock().unwrap();
        *current_tokens as usize
    }
}

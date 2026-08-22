use dagr_core::{DagrError, Result};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

pub struct ToolCircuitBreaker {
    state: Mutex<BreakerState>,
    failure_threshold: usize,
    consecutive_failures: Mutex<usize>,
    open_cooldown: Duration,
    last_tripped_at: Mutex<Option<Instant>>,
}

impl ToolCircuitBreaker {
    pub fn new(failure_threshold: usize, cooldown: Duration) -> Self {
        Self {
            state: Mutex::new(BreakerState::Closed),
            failure_threshold,
            consecutive_failures: Mutex::new(0),
            open_cooldown: cooldown,
            last_tripped_at: Mutex::new(None),
        }
    }

    pub fn default_tool_breaker() -> Self {
        Self::new(3, Duration::from_secs(30))
    }

    pub fn before_call(&self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let last_tripped = self.last_tripped_at.lock().unwrap();

        match *state {
            BreakerState::Closed => Ok(()),
            BreakerState::Open => {
                if let Some(tripped_time) = *last_tripped {
                    if tripped_time.elapsed() >= self.open_cooldown {
                        *state = BreakerState::HalfOpen;
                        Ok(())
                    } else {
                        Err(DagrError::Internal(format!(
                            "Tool Circuit Breaker is OPEN: tripping protective fast-fallback (cooldown: {:.1}s remaining)",
                            (self.open_cooldown - tripped_time.elapsed()).as_secs_f64()
                        )))
                    }
                } else {
                    *state = BreakerState::HalfOpen;
                    Ok(())
                }
            }
            BreakerState::HalfOpen => Ok(()),
        }
    }

    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        let mut failures = self.consecutive_failures.lock().unwrap();
        *failures = 0;
        *state = BreakerState::Closed;
    }

    pub fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        let mut failures = self.consecutive_failures.lock().unwrap();
        let mut last_tripped = self.last_tripped_at.lock().unwrap();

        *failures += 1;
        if *failures >= self.failure_threshold {
            *state = BreakerState::Open;
            *last_tripped = Some(Instant::now());
        }
    }

    pub fn get_state(&self) -> BreakerState {
        *self.state.lock().unwrap()
    }
}

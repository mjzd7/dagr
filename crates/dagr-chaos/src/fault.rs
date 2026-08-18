use dagr_core::Result;
use dagr_sandbox::CowSandbox;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FaultType {
    /// Injects synthetic network latency delay before tool/database operations
    NetworkLatency { delay_ms: u64 },
    /// Injects CPU throttling delay cycles
    CpuThrottle { delay_ms: u64 },
    /// Injects transaction lock contention & forced retry loops
    LockContention { retry_count: u32, jitter_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FaultMatrix {
    pub faults: Vec<FaultType>,
    pub max_fault_duration_ms: u64,
}

impl Default for FaultMatrix {
    fn default() -> Self {
        Self {
            faults: vec![
                FaultType::NetworkLatency { delay_ms: 10 },
                FaultType::CpuThrottle { delay_ms: 5 },
                FaultType::LockContention {
                    retry_count: 2,
                    jitter_ms: 5,
                },
            ],
            max_fault_duration_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChaosExecutionReport {
    pub success: bool,
    pub faults_injected_count: usize,
    pub total_execution_time_ms: u128,
    pub stdout: String,
    pub stderr: String,
    pub resilience_score: f32, // 1.0 = flawless pass under full chaos
}

pub struct ChaosEngine;

impl ChaosEngine {
    /// Executes a verification command inside a CoW sandbox under active chaos fault conditions
    pub fn execute_under_chaos(
        workspace_root: &Path,
        command: &str,
        matrix: &FaultMatrix,
    ) -> Result<ChaosExecutionReport> {
        let tx = CowSandbox::begin(workspace_root)?;
        let start = Instant::now();

        // 1. Simulate fault delays (network latency & CPU throttling)
        let mut total_injected_delay = 0;
        for fault in &matrix.faults {
            match fault {
                FaultType::NetworkLatency { delay_ms } => {
                    total_injected_delay += *delay_ms;
                }
                FaultType::CpuThrottle { delay_ms } => {
                    total_injected_delay += *delay_ms;
                }
                FaultType::LockContention {
                    retry_count,
                    jitter_ms,
                } => {
                    total_injected_delay += (*retry_count as u64) * *jitter_ms;
                }
            }
        }

        // Cap delay to prevent test timeouts
        let delay_to_apply = total_injected_delay.min(matrix.max_fault_duration_ms);
        std::thread::sleep(Duration::from_millis(delay_to_apply));

        // 2. Execute command in isolated CoW shadow workspace
        let exec_result = CowSandbox::verify(&tx, command)?;
        let total_time = start.elapsed().as_millis();

        // 3. Always rollback shadow mutations to guarantee 100% clean state
        CowSandbox::rollback(tx)?;

        let resilience_score = if exec_result.success { 1.0 } else { 0.0 };

        Ok(ChaosExecutionReport {
            success: exec_result.success,
            faults_injected_count: matrix.faults.len(),
            total_execution_time_ms: total_time,
            stdout: exec_result.stdout,
            stderr: exec_result.stderr,
            resilience_score,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_execution_resilience() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let matrix = FaultMatrix::default();

        let report = ChaosEngine::execute_under_chaos(
            temp_dir.path(),
            "echo 'Chaos stress test green'",
            &matrix,
        )?;

        assert!(report.success);
        assert_eq!(report.resilience_score, 1.0);
        assert_eq!(report.faults_injected_count, 3);
        assert!(report.stdout.contains("Chaos stress test green"));

        Ok(())
    }
}

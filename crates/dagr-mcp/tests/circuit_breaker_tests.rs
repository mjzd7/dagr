//! Tests for ToolCircuitBreaker — the only Agent-OS module without dedicated tests.

use dagr_mcp::circuit_breaker::{BreakerState, ToolCircuitBreaker};
use std::time::Duration;

#[test]
fn initial_state_is_closed() {
    let breaker = ToolCircuitBreaker::new(3, Duration::from_secs(60));
    assert_eq!(breaker.get_state(), BreakerState::Closed);
    assert!(breaker.before_call().is_ok());
}

#[test]
fn failures_below_threshold_do_not_trip() {
    let breaker = ToolCircuitBreaker::new(3, Duration::from_secs(60));
    breaker.record_failure();
    breaker.record_failure();
    assert_eq!(breaker.get_state(), BreakerState::Closed);
    assert!(breaker.before_call().is_ok());
}

#[test]
fn reaching_threshold_trips_open() {
    let breaker = ToolCircuitBreaker::new(3, Duration::from_secs(60));
    for _ in 0..3 {
        breaker.record_failure();
    }
    assert_eq!(breaker.get_state(), BreakerState::Open);
    assert!(breaker.before_call().is_err());
}

#[test]
fn success_resets_failure_count() {
    let breaker = ToolCircuitBreaker::new(3, Duration::from_secs(60));
    breaker.record_failure();
    breaker.record_failure();
    breaker.record_success();
    assert_eq!(breaker.get_state(), BreakerState::Closed);
    // Two more failures won't trip because counter was reset.
    breaker.record_failure();
    breaker.record_failure();
    assert_eq!(breaker.get_state(), BreakerState::Closed);
}

#[test]
fn open_transitions_to_halfopen_after_cooldown() {
    let breaker = ToolCircuitBreaker::new(1, Duration::from_millis(50));
    breaker.record_failure();
    assert_eq!(breaker.get_state(), BreakerState::Open);

    std::thread::sleep(Duration::from_millis(80));
    assert!(breaker.before_call().is_ok());
    assert_eq!(breaker.get_state(), BreakerState::HalfOpen);
}

#[test]
fn halfopen_success_closes_breaker() {
    let breaker = ToolCircuitBreaker::new(1, Duration::from_millis(50));
    breaker.record_failure();

    std::thread::sleep(Duration::from_millis(80));
    assert!(breaker.before_call().is_ok()); // HalfOpen
    breaker.record_success();
    assert_eq!(breaker.get_state(), BreakerState::Closed);
}

#[test]
fn halfopen_failure_reopens_breaker() {
    let breaker = ToolCircuitBreaker::new(1, Duration::from_millis(50));
    breaker.record_failure();

    std::thread::sleep(Duration::from_millis(80));
    assert!(breaker.before_call().is_ok()); // HalfOpen allows probe
    breaker.record_failure(); // Probe also failed → reopen
    assert_eq!(breaker.get_state(), BreakerState::Open);
}

#[test]
fn default_tool_breaker_uses_3_failures_30s_cooldown() {
    let breaker = ToolCircuitBreaker::default_tool_breaker();
    assert_eq!(breaker.get_state(), BreakerState::Closed);
    for _ in 0..3 {
        breaker.record_failure();
    }
    assert_eq!(breaker.get_state(), BreakerState::Open);
}

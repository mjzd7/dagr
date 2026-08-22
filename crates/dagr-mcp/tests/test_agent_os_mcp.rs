use dagr_mcp::{BreakerState, ToolCircuitBreaker};
use std::time::Duration;

#[test]
fn test_tool_circuit_breaker_transitions() {
    let breaker = ToolCircuitBreaker::new(3, Duration::from_millis(50));
    assert_eq!(breaker.get_state(), BreakerState::Closed);
    assert!(breaker.before_call().is_ok());

    // 2 failures
    breaker.record_failure();
    breaker.record_failure();
    assert_eq!(breaker.get_state(), BreakerState::Closed);

    // 3rd failure trips breaker to OPEN
    breaker.record_failure();
    assert_eq!(breaker.get_state(), BreakerState::Open);
    assert!(
        breaker.before_call().is_err(),
        "Open breaker must reject calls"
    );

    // Wait for cooldown
    std::thread::sleep(Duration::from_millis(60));

    // Next call transitions to HalfOpen
    assert!(breaker.before_call().is_ok());
    assert_eq!(breaker.get_state(), BreakerState::HalfOpen);

    // Successful call resets to Closed
    breaker.record_success();
    assert_eq!(breaker.get_state(), BreakerState::Closed);
}

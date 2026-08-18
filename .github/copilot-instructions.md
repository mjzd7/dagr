# DAGR Hypervisor Instructions for GitHub Copilot & Coding Agents

1. **Precision Context Protocol**:
   Before modifying any symbol or function, invoke `dagr context <FILE>:<SYMBOL>` to retrieve the minimal AST slice and hoisted type contracts. Do not read the entire file.

2. **Architecture Boundary Invariant**:
   Execute `dagr guard` before submitting code to ensure zero boundary violations.

3. **Sandboxed Verification**:
   Run test suites inside `dagr run "<CMD>"` to guarantee clean rollback on failure.

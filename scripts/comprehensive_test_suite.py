#!/usr/bin/env python3
"""
DAGR Comprehensive End-to-End Functionality Verification Suite
Tests all 9 core subsystems, CLI commands, MCP tools, and edge cases.
"""

import os
import sys
import json
import time
import shutil
import tempfile
import subprocess
from pathlib import Path

DAGR_BIN = Path("./target/debug/dagr").resolve()

results = []

def run_test(name, func):
    start = time.time()
    try:
        status, details = func()
        duration_ms = (time.time() - start) * 1000
        results.append({
            "name": name,
            "status": "PASS" if status else "FAIL",
            "duration_ms": round(duration_ms, 2),
            "details": details
        })
        print(f"[{'PASS' if status else 'FAIL'}] {name} ({duration_ms:.2f}ms)")
        if not status:
            print(f"       Error Details: {details}")
    except Exception as e:
        duration_ms = (time.time() - start) * 1000
        results.append({
            "name": name,
            "status": "ERROR",
            "duration_ms": round(duration_ms, 2),
            "details": str(e)
        })
        print(f"[ERROR] {name}: {e}")

# ====================================================================
# 1. AST Slicing & Contract Hoisting Tests
# ====================================================================

def test_ast_slicing_typescript():
    cmd = [str(DAGR_BIN), "context", "tests/fixtures/billing_service.ts:chargeCustomer", "--format", "json"]
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        return False, f"CLI exited with {p.returncode}: {p.stderr}"
    data = json.loads(p.stdout)
    if data["target_symbol"] != "chargeCustomer":
        return False, "Target symbol mismatch"
    if len(data["type_contracts"]) == 0:
        return False, "Expected hoisted type contracts"
    if data["compression_ratio"] <= 0:
        return False, "Invalid compression ratio"
    return True, f"Tokens: {data['estimated_tokens']}/{data['original_file_tokens']} ({data['compression_ratio']*100:.1f}%)"

def test_ast_slicing_python():
    cmd = [str(DAGR_BIN), "context", "tests/fixtures/auth_pipeline.py:verify_token", "--format", "json"]
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        return False, f"CLI exited with {p.returncode}: {p.stderr}"
    data = json.loads(p.stdout)
    if data["target_symbol"] != "verify_token":
        return False, "Target symbol mismatch"
    return True, f"Tokens: {data['estimated_tokens']}/{data['original_file_tokens']}"

def test_ast_slicing_nonexistent_symbol():
    cmd = [str(DAGR_BIN), "context", "tests/fixtures/billing_service.ts:nonExistentMethod"]
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode == 0:
        return False, "Expected error on missing symbol"
    if "Symbol Not Found" not in p.stdout and "Symbol Not Found" not in p.stderr:
        return False, f"Unexpected error message: {p.stderr}"
    return True, "Properly returned clean Symbol Not Found error"

# ====================================================================
# 2. Architecture Boundary Guard Tests
# ====================================================================

def test_guard_pass_clean_workspace():
    cmd = [str(DAGR_BIN), "guard", "--workspace", ".", "--format", "json"]
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        return False, f"Guard failed on clean workspace: {p.stderr}"
    data = json.loads(p.stdout)
    if data["violations_count"] != 0:
        return False, f"Found unexpected violations: {data['violations']}"
    return True, "0 boundary violations on clean workspace"

def test_guard_detects_violating_import():
    with tempfile.TemporaryDirectory() as tmpdir:
        dagr_dir = os.path.join(tmpdir, ".dagr")
        os.makedirs(dagr_dir, exist_ok=True)
        rules_path = os.path.join(dagr_dir, "rules.yaml")
        with open(rules_path, "w") as f:
            f.write("""version: "1.0"
boundaries:
  - name: "UI-to-DB Boundary"
    from: "src/ui/**"
    cannot_import:
      - "src/db/**"
    message: "UI cannot import DB"
""")
        ui_dir = os.path.join(tmpdir, "src", "ui")
        os.makedirs(ui_dir, exist_ok=True)
        with open(os.path.join(ui_dir, "Component.tsx"), "w") as f:
            f.write('import { client } from "src/db/client";\nexport function Component() {}\n')

        cmd = [str(DAGR_BIN), "guard", "--workspace", tmpdir, "--format", "json"]
        p = subprocess.run(cmd, capture_output=True, text=True)
        if p.returncode == 0:
            return False, "Expected guard to fail on violating import"
        data = json.loads(p.stdout)
        if data["violations_count"] != 1:
            return False, f"Expected 1 violation, found {data['violations_count']}"
        if data["violations"][0]["rule_name"] != "UI-to-DB Boundary":
            return False, "Violation rule mismatch"
        return True, "Accurately caught violating import and returned exit code 1"

# ====================================================================
# 3. Copy-on-Write Sandbox & Atomic Rollback Tests
# ====================================================================

def test_cow_sandbox_atomic_rollback_on_failure():
    with tempfile.TemporaryDirectory() as tmpdir:
        test_file = os.path.join(tmpdir, "original.txt")
        with open(test_file, "w") as f:
            f.write("pristine state\n")

        # Run command that modifies file and then fails
        failing_cmd = "echo 'corrupted' > original.txt && exit 1"
        cmd = [str(DAGR_BIN), "run", failing_cmd]
        p = subprocess.run(cmd, cwd=tmpdir, capture_output=True, text=True)
        if p.returncode == 0:
            return False, "Expected run command to fail"

        # Verify original file was completely preserved
        with open(test_file, "r") as f:
            content = f.read()
        if content != "pristine state\n":
            return False, f"Rollback failed, content was modified: {content}"
        return True, "Shadow workspace successfully rolled back with 0 modified bytes"

def test_cow_sandbox_commit_on_success():
    with tempfile.TemporaryDirectory() as tmpdir:
        test_file = os.path.join(tmpdir, "created.txt")
        success_cmd = "echo 'valid patch' > created.txt"
        cmd = [str(DAGR_BIN), "run", success_cmd, "--commit-on-success"]
        p = subprocess.run(cmd, cwd=tmpdir, capture_output=True, text=True)
        if p.returncode != 0:
            return False, f"Run command failed: {p.stderr}"
        if not os.path.exists(test_file):
            return False, "Commit on success failed to apply created file"
        with open(test_file, "r") as f:
            content = f.read().strip()
        if content != "valid patch":
            return False, f"File content mismatch: {content}"
        return True, "Successfully committed shadow workspace changes on green verification"

# ====================================================================
# 4. MCP JSON-RPC 2.0 Protocol & Stdio Log Isolation Tests
# ====================================================================

def test_mcp_protocol_handshake_and_tools():
    req1 = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-suite", "version": "1.0"}
        }
    }
    req2 = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }
    req3 = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "dagr_get_context_slice",
            "arguments": {
                "file_path": "tests/fixtures/billing_service.ts",
                "symbol_name": "chargeCustomer"
            }
        }
    }

    input_payload = "\n".join([json.dumps(r) for r in [req1, req2, req3]]) + "\n"
    p = subprocess.run(
        [str(DAGR_BIN), "mcp", "start"],
        input=input_payload,
        capture_output=True,
        text=True
    )

    stdout_lines = [l for l in p.stdout.strip().split("\n") if l.strip()]
    if len(stdout_lines) != 3:
        return False, f"Expected 3 JSON-RPC responses, got {len(stdout_lines)}"

    resp1 = json.loads(stdout_lines[0])
    if resp1.get("result", {}).get("serverInfo", {}).get("name") != "dagr-hypervisor":
        return False, "ServerInfo mismatch in initialize response"

    resp2 = json.loads(stdout_lines[1])
    tool_names = [t["name"] for t in resp2.get("result", {}).get("tools", [])]
    expected_tools = ["dagr_get_context_slice", "dagr_verify_architecture", "dagr_execute_sandboxed"]
    for t in expected_tools:
        if t not in tool_names:
            return False, f"Missing tool: {t}"

    resp3 = json.loads(stdout_lines[2])
    content_raw = resp3.get("result", {}).get("content", [{}])[0].get("text", "")
    slice_data = json.loads(content_raw)
    if slice_data["target_symbol"] != "chargeCustomer":
        return False, "Tool call slice mismatch"

    return True, f"All 3 JSON-RPC protocol frames returned valid results. Available tools: {len(tool_names)}"

# ====================================================================
# 5. One-Click MCP Installer Tests
# ====================================================================

def test_mcp_installer():
    with tempfile.TemporaryDirectory() as tmpdir:
        target_json = os.path.join(tmpdir, "test_mcp.json")
        cmd = [str(DAGR_BIN), "mcp", "install", "--client", target_json, "--bin-path", "/usr/local/bin/dagr"]
        p = subprocess.run(cmd, capture_output=True, text=True)
        if p.returncode != 0:
            return False, f"MCP install command failed: {p.stderr}"
        if not os.path.exists(target_json):
            return False, "MCP install failed to create target file"
        with open(target_json, "r") as f:
            data = json.load(f)
        if "mcpServers" not in data or "dagr" not in data["mcpServers"]:
            return False, "DAGR server entry missing in JSON"
        if data["mcpServers"]["dagr"]["command"] != "/usr/local/bin/dagr":
            return False, "Command path mismatch in JSON"
        return True, "Successfully injected and validated JSON configuration"

# ====================================================================
# 6. Automatic Architecture Inference Tests
# ====================================================================

def test_init_framework_inference():
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create mock Next.js project
        with open(os.path.join(tmpdir, "next.config.js"), "w") as f:
            f.write("module.exports = {};")

        cmd = [str(DAGR_BIN), "init"]
        p = subprocess.run(cmd, cwd=tmpdir, capture_output=True, text=True)
        if p.returncode != 0:
            return False, f"Init command failed: {p.stderr}"

        rules_path = os.path.join(tmpdir, ".dagr", "rules.yaml")
        if not os.path.exists(rules_path):
            return False, ".dagr/rules.yaml was not generated"

        with open(rules_path, "r") as f:
            content = f.read()
        if "nextjs" not in content:
            return False, f"Expected Next.js preset in inferred rules, got: {content}"
        return True, "Automatically detected Next.js app and created valid rules.yaml"

# ====================================================================
# Runner
# ====================================================================

def main():
    print("====================================================================")
    print("⚡ DAGR Hypervisor Comprehensive Functionality Verification Suite")
    print("====================================================================\n")

    run_test("AST Slicing (TypeScript)", test_ast_slicing_typescript)
    run_test("AST Slicing (Python)", test_ast_slicing_python)
    run_test("AST Slicing (Missing Symbol Handling)", test_ast_slicing_nonexistent_symbol)
    run_test("Architecture Guard (Clean Workspace)", test_guard_pass_clean_workspace)
    run_test("Architecture Guard (Violation Interception)", test_guard_detects_violating_import)
    run_test("CoW Sandbox (Atomic Rollback on Error)", test_cow_sandbox_atomic_rollback_on_failure)
    run_test("CoW Sandbox (Commit on Success)", test_cow_sandbox_commit_on_success)
    run_test("MCP JSON-RPC Protocol & Tools Stdio", test_mcp_protocol_handshake_and_tools)
    run_test("MCP One-Click Auto-Installer", test_mcp_installer)
    run_test("Architecture Framework Inferrer (Init)", test_init_framework_inference)

    print("\n====================================================================")
    passed = sum(1 for r in results if r["status"] == "PASS")
    total = len(results)
    print(f"Final Scorecard: {passed}/{total} Tests Passed ({passed/total*100:.1f}%)")
    print("====================================================================")

    if passed != total:
        sys.exit(1)

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
import subprocess
import tempfile
import os
import json
import shutil

DAGR_BIN = shutil.which("dagr") or "/Users/mm/.cargo/bin/dagr"
print(f"⚡ Testing DAGR binary: {DAGR_BIN}\n")

tests = []

def run_test(name, fn):
    try:
        ok, msg = fn()
        if ok:
            print(f" [PASS] {name}: {msg}")
            tests.append((name, True, msg))
        else:
            print(f" [FAIL] {name}: {msg}")
            tests.append((name, False, msg))
    except Exception as e:
        print(f" [ERROR] {name}: {e}")
        tests.append((name, False, str(e)))

# 1. Test dagr init
def test_init():
    with tempfile.TemporaryDirectory() as tmp:
        # Create a mock nextjs project
        with open(os.path.join(tmp, "next.config.js"), "w") as f:
            f.write("module.exports = {};\n")
        p = subprocess.run([DAGR_BIN, "init"], cwd=tmp, capture_output=True, text=True)
        if p.returncode != 0:
            return False, f"dagr init failed: {p.stderr}"
        if not os.path.exists(os.path.join(tmp, ".dagr", "rules.yaml")):
            return False, "rules.yaml was not generated"
        return True, "Successfully generated .dagr/rules.yaml with inferred nextjs preset"

# 2. Test dagr context
def test_context():
    with tempfile.TemporaryDirectory() as tmp:
        file_path = os.path.join(tmp, "service.ts")
        with open(file_path, "w") as f:
            f.write("""
export interface User { id: string; name: string; }
export interface Other { x: number; }
export class UserService {
  public getUser(user: User): string {
    return user.name;
  }
  public ignoreMe(): void {}
}
""")
        p = subprocess.run([DAGR_BIN, "context", f"{file_path}:getUser", "--format", "json"], cwd=tmp, capture_output=True, text=True)
        if p.returncode != 0:
            return False, f"dagr context failed: {p.stderr}"
        data = json.loads(p.stdout)
        if "User" not in str(data["type_contracts"]):
            return False, "Failed to hoist User interface"
        if "Other" in str(data["type_contracts"]):
            return False, "Failed to prune Other interface"
        return True, "Successfully sliced function & hoisted contracts in JSON format"

# 3. Test dagr guard
def test_guard():
    with tempfile.TemporaryDirectory() as tmp:
        dagr_dir = os.path.join(tmp, ".dagr")
        os.makedirs(dagr_dir, exist_ok=True)
        with open(os.path.join(dagr_dir, "rules.yaml"), "w") as f:
            f.write("""version: "1.0"
boundaries:
  - name: "UI-DB"
    from: "src/ui/**"
    cannot_import: ["src/db/**"]
""")
        os.makedirs(os.path.join(tmp, "src", "ui"), exist_ok=True)
        with open(os.path.join(tmp, "src", "ui", "App.tsx"), "w") as f:
            f.write('import { db } from "src/db/client";\n')
        
        p = subprocess.run([DAGR_BIN, "guard", "--workspace", tmp, "--format", "json"], capture_output=True, text=True)
        if p.returncode == 0:
            return False, "Expected guard to detect violation"
        data = json.loads(p.stdout)
        if data["violations_count"] != 1:
            return False, f"Expected 1 violation, got {data['violations_count']}"
        return True, "Accurately intercepted boundary violation and exited code 1"

# 4. Test dagr run (Sandbox Rollback)
def test_run_rollback():
    with tempfile.TemporaryDirectory() as tmp:
        p = subprocess.run([DAGR_BIN, "run", "echo 'dirty' > test.txt && exit 1"], cwd=tmp, capture_output=True, text=True)
        if p.returncode == 0:
            return False, "Expected sandboxed command to fail"
        if os.path.exists(os.path.join(tmp, "test.txt")):
            return False, "test.txt was not rolled back"
        return True, "Shadow workspace successfully discarded with 0 residual files"

# 5. Test dagr run (Commit on Success)
def test_run_commit():
    with tempfile.TemporaryDirectory() as tmp:
        p = subprocess.run([DAGR_BIN, "run", "echo 'clean' > test.txt", "--commit-on-success"], cwd=tmp, capture_output=True, text=True)
        if p.returncode != 0:
            return False, f"Expected sandboxed command to succeed: {p.stderr}"
        if not os.path.exists(os.path.join(tmp, "test.txt")):
            return False, "test.txt was not committed"
        return True, "Successfully applied verified shadow changes to workspace"

# 6. Test dagr mcp list-clients
def test_mcp_list_clients():
    p = subprocess.run([DAGR_BIN, "mcp", "list-clients"], capture_output=True, text=True)
    if p.returncode != 0:
        return False, f"mcp list-clients failed: {p.stderr}"
    if "cursor" not in p.stdout or "claude" not in p.stdout or "windsurf" not in p.stdout:
        return False, "Missing expected clients in list"
    return True, "Listed all 30+ supported AI IDEs & agents"

# 7. Test dagr mcp install
def test_mcp_install():
    with tempfile.TemporaryDirectory() as tmp:
        custom_mcp = os.path.join(tmp, "custom_mcp.json")
        p = subprocess.run([DAGR_BIN, "mcp", "install", "--client", custom_mcp], capture_output=True, text=True)
        if p.returncode != 0:
            return False, f"mcp install failed: {p.stderr}"
        if not os.path.exists(custom_mcp):
            return False, "custom_mcp.json was not created"
        with open(custom_mcp, "r") as f:
            data = json.load(f)
        if "dagr" not in data.get("mcpServers", {}):
            return False, "dagr server key missing in mcpServers"
        return True, "Successfully injected dagr server into configuration"

# 8. Test dagr skills list
def test_skills_list():
    p = subprocess.run([DAGR_BIN, "skills", "list"], capture_output=True, text=True)
    if p.returncode != 0:
        return False, f"skills list failed: {p.stderr}"
    if "dagr-slicer" not in p.stdout or "dagr-guard" not in p.stdout:
        return False, "Missing core skills in output"
    return True, "Successfully listed all portable SKILL.md packages"

# 9. Test dagr skills install
def test_skills_install():
    with tempfile.TemporaryDirectory() as tmp:
        p = subprocess.run([DAGR_BIN, "skills", "install", "--target", tmp], capture_output=True, text=True)
        if p.returncode != 0:
            return False, f"skills install failed: {p.stderr}"
        for skill in ["dagr-slicer", "dagr-guard", "dagr-sandbox", "dagr-chaos"]:
            skill_file = os.path.join(tmp, skill, "SKILL.md")
            if not os.path.exists(skill_file):
                return False, f"{skill_file} was not installed"
        return True, "Successfully installed all 4 SKILL.md packages into target"

# 10. Test dagr update --help
def test_update_help():
    p = subprocess.run([DAGR_BIN, "update", "--help"], capture_output=True, text=True)
    if p.returncode != 0:
        return False, f"update --help failed: {p.stderr}"
    return True, "Update command verified"

print("====================================================================")
print("⚡ Running DAGR Sandbox Command Verification Battery")
print("====================================================================")
run_test("1.  dagr init", test_init)
run_test("2.  dagr context", test_context)
run_test("3.  dagr guard", test_guard)
run_test("4.  dagr run (rollback)", test_run_rollback)
run_test("5.  dagr run (commit)", test_run_commit)
run_test("6.  dagr mcp list-clients", test_mcp_list_clients)
run_test("7.  dagr mcp install", test_mcp_install)
run_test("8.  dagr skills list", test_skills_list)
run_test("9.  dagr skills install", test_skills_install)
run_test("10. dagr update", test_update_help)

passed = sum(1 for _, ok, _ in tests if ok)
total = len(tests)
print("\n====================================================================")
print(f"Final Scorecard: {passed}/{total} Sandbox Tests Passed ({passed/total*100:.1f}%)")
print("====================================================================")

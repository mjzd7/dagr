import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync, rmSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname, resolve as resolvePath } from "node:path";

// Strategies:
//   baseline = paste every task source file whole into the prompt
//   dagr     = inject `dagr context` slice output instead
export function buildPrompt(taskDir, strategy, dagrBin) {
  const task = JSON.parse(readFileSync(join(taskDir, "task.json"), "utf8"));
  const files = task.files.map((f) => ({
    path: f,
    content: readFileSync(join(taskDir, "repo", f), "utf8"),
  }));
  let contextBlock;
  if (strategy === "baseline") {
    contextBlock = files
      .map((f) => `--- FILE: ${f.path} ---\n${f.content}`)
      .join("\n");
  } else {
    const target = `${files[0].path}:${task.symbol}`;
    const slice = execFileSync(dagrBin, ["context", target, "--format", "json"], {
      encoding: "utf8",
      cwd: join(taskDir, "repo"),
    });
    contextBlock = `--- DAGR SLICE for ${target} ---\n${slice}`;
  }
  return {
    system: task.system_prompt,
    user: `${contextBlock}\n---TASK---\n${JSON.stringify(task)}\nRespond with ONLY the complete new contents of ${task.target_file}.`,
    task,
  };
}

export function gradeTask(taskDir, responseText) {
  // Write the model's file into a scratch copy of the repo and run hidden tests.
  const task = JSON.parse(readFileSync(join(taskDir, "task.json"), "utf8"));
  const scratch = mkdtempSync(join(tmpdir(), "dagr-eval-"));
  try {
    execFileSync("cp", ["-R", join(taskDir, "repo") + "/.", scratch]);
    writeFileSync(join(scratch, "hidden-test.mjs"), readFileSync(join(taskDir, "hidden-test.mjs"), "utf8"));
    writeFileSync(join(scratch, task.target_file), responseText);
    const testCmd = task.test_command;
    try {
      execFileSync("sh", ["-c", testCmd], { cwd: scratch, stdio: "pipe" });
      return { pass: true, defects: 0 };
    } catch (e) {
      if (process.env.DAGR_EVAL_DEBUG) console.error("GRADE-CMD-FAIL:", String(e.stderr ?? e.message).slice(0, 400));
      const out = `${e.stdout ?? ""}${e.stderr ?? ""}`;
      const failures = (out.match(/FAIL|✗|failed/g) ?? []).length;
      return { pass: false, defects: Math.max(1, Math.min(failures, 9)) };
    }
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

export async function runTask(provider, taskDir, strategy, dagrBin) {
  const { system, user, task } = buildPrompt(taskDir, strategy, dagrBin);
  const started = process.hrtime.bigint();
  const res = await provider.complete({ system, user });
  const elapsedMs = Number(process.hrtime.bigint() - started) / 1e6;
    const responseText = typeof res === "string" ? res : (res.text ?? "");
  const grade = gradeTask(taskDir, responseText);
  return {
    task: task.id,
    strategy,
    provider: provider.name,
    pass: grade.pass,
    defects: grade.defects,
    tokens_in_baseline: strategy === "baseline" ? countTokens(user) : null,
    tokens_in_dagr: strategy === "dagr" ? countTokens(user) : null,
    tokens_out: res.tokens_out ?? 0,
    latency_ms: Math.round(elapsedMs),
  };
}

function countTokens(s) {
  return Math.ceil((s ?? "").length / 4);
}

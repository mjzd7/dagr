#!/usr/bin/env node
// DAGR Pilot Eval — outcome-quality benchmark harness.
//
//   node evals/run.mjs --provider mock
//   ANTHROPIC_API_KEY=... node evals/run.mjs --provider anthropic --tasks task-001-fix-function
//
// Compares BASELINE (whole files pasted) vs DAGR (context slice injected)
// on identical tasks, scoring PASS/FAIL against hidden tests. Zero deps.
import { readdirSync, readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { join, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { makeProvider } from "./lib/provider.mjs";
import { runTask } from "./lib/harness.mjs";

const args = process.argv.slice(2);
const arg = (name, dflt) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 ? args[i + 1] : dflt;
};
const providerName = arg("provider", "mock");
const onlyTask = arg("tasks", null);
const modelName = arg("model", null);
const dagrBin = resolve(arg("dagr-bin") ?? "target/debug/dagr");

const root = dirname(fileURLToPath(import.meta.url));
const tasksDir = join(root, "tasks");
const tasks = readdirSync(tasksDir)
  .filter((d) => d.startsWith("task-"))
  .filter((d) => !onlyTask || d === onlyTask || d.includes(onlyTask))
  .sort();

if (!tasks.length) { console.error("no tasks matched"); process.exit(2); }
const provider = makeProvider(providerName);
const results = [];
for (const t of tasks) {
  for (const strategy of ["baseline", "dagr"]) {
    try {
      results.push(await runTask(provider, join(tasksDir, t), strategy, dagrBin, modelName));
    } catch (e) {
      results.push({ task: t, strategy, provider: providerName, model: modelName ?? "default", pass: false, defects: 9, error: String(e.message ?? e).slice(0, 200), latency_ms: 0 });
    }
  }
}

const byStrategy = {};
for (const r of results) {
  byStrategy[r.strategy] ??= { runs: 0, passes: 0, defects: 0 };
  byStrategy[r.strategy].runs++;
  byStrategy[r.strategy].passes += r.pass ? 1 : 0;
  byStrategy[r.strategy].defects += r.defects;
}
const summary = { generated_at_unix: Math.floor(Date.now() / 1000), provider: provider.name, model: modelName ?? "default", byStrategy, results };
console.log(JSON.stringify(summary, null, 2));

const outDir = resolve(root, "results");
mkdirSync(outDir, { recursive: true });
writeFileSync(join(outDir, "latest.json"), JSON.stringify(summary, null, 2));

const mockMode = providerName === "mock";
console.error(
  `\nDAGR Pilot Eval (${mockMode ? "MOCK — mechanics check only" : `provider=${providerName}`})`
);
for (const [s, v] of Object.entries(byStrategy)) {
  console.error(`  ${s.padEnd(9)} ${v.passes}/${v.runs} pass, defects=${v.defects}`);
}

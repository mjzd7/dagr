use dagr_core::Result;
use std::path::PathBuf;

pub struct SkillDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub content: &'static str,
}

pub const DAGR_SKILLS: &[SkillDefinition] = &[
    SkillDefinition {
        name: "dagr-slicer",
        description: "Surgical AST context slicing and contract hoisting hypervisor. Use whenever inspecting, analyzing, or preparing to modify a function, method, or class, to avoid loading full files and slash token consumption by >95%. Also use when user mentions token reduction, context slicing, or AST extraction.",
        content: r#"---
name: dagr-slicer
description: Surgical AST context slicing and contract hoisting hypervisor. Use whenever inspecting, analyzing, or preparing to modify a function, method, or class, to avoid loading full files and slash token consumption by >95%. Also use when user mentions token reduction, context slicing, or AST extraction.
---

# `dagr-slicer` Agent Skill

## 🎯 When to Use
- Whenever preparing to edit, review, or write a function, method, or class.
- When you need to understand a symbol's contract without blowing your token budget on a 2,000-line file.
- When the user asks you to inspect or refactor a specific function.

## 🚀 How to Execute
Call the CLI command or MCP tool:
```bash
dagr context <FILE_PATH>:<SYMBOL_NAME> --format json
```
Or MCP tool:
`dagr_get_context_slice(file_path="...", symbol_name="...")`

## 📋 Returned Context
1. **Hoisted Type Contracts**: Exact interfaces and classes referenced by the symbol.
2. **Minimal Implementation Slice**: Only the relevant body lines.
3. **Token Footprint**: Precise token count and compression ratio.
"#,
    },
    SkillDefinition {
        name: "dagr-guard",
        description: "In-memory architectural boundary and layer import linter (<0.1ms). Use whenever validating code changes, writing imports, creating PRs, or verifying clean architecture boundaries (e.g. UI cannot import DB/ORM). Also use when user mentions boundary rules, layer violations, or architecture guard.",
        content: r#"---
name: dagr-guard
description: In-memory architectural boundary and layer import linter (<0.1ms). Use whenever validating code changes, writing imports, creating PRs, or verifying clean architecture boundaries (e.g. UI cannot import DB/ORM). Also use when user mentions boundary rules, layer violations, or architecture guard.
---

# `dagr-guard` Agent Skill

## 🎯 When to Use
- Before submitting code changes, creating a commit, or opening a PR.
- To verify that presentation layers (UI) do not directly import database/ORM layers.
- To sanitize user comments against indirect prompt injections.

## 🚀 How to Execute
Call the CLI command or MCP tool:
```bash
dagr guard --format json
```
Or MCP tool:
`dagr_verify_architecture(source_file="...", proposed_imports=[...])`
"#,
    },
    SkillDefinition {
        name: "dagr-sandbox",
        description: "Copy-on-Write (CoW) shadow workspace runner. Use whenever executing tests, refactors, or potentially destructive commands with instant 10ms atomic rollback on failure. Also use when user mentions sandboxed run, safe trial, or atomic rollback.",
        content: r#"---
name: dagr-sandbox
description: Copy-on-Write (CoW) shadow workspace runner. Use whenever executing tests, refactors, or potentially destructive commands with instant 10ms atomic rollback on failure. Also use when user mentions sandboxed run, safe trial, or atomic rollback.
---

# `dagr-sandbox` Agent Skill

## 🎯 When to Use
- When executing tests, builds, or scripts that might fail or leave dirty artifacts.
- When performing speculative refactors where you want guaranteed 0-side-effect rollback on failure.

## 🚀 How to Execute
Call the CLI command or MCP tool:
```bash
dagr run "<TEST_COMMAND>" [--commit-on-success]
```
Or MCP tool:
`dagr_execute_sandboxed(command="...")`
"#,
    },
    SkillDefinition {
        name: "dagr-chaos",
        description: "Ephemeral chaos fault injection and cryptographic Proof-of-Correctness generator. Use whenever stress-testing PRs under latency, CPU throttling, or lock contention.",
        content: r#"---
name: dagr-chaos
description: Ephemeral chaos fault injection and cryptographic Proof-of-Correctness generator. Use whenever stress-testing PRs under latency, CPU throttling, or lock contention.
---

# `dagr-chaos` Agent Skill

## 🎯 When to Use
- To stress-test PRs under adverse conditions (latency, thread lock contention, packet jitter).
- To generate unforgeable Blake3 Proof-of-Correctness audit badges.
"#,
    },
];

pub struct SkillsInstaller;

impl SkillsInstaller {
    /// Resolves target skill installation paths based on target environment
    pub fn get_target_directories(target: &str) -> Vec<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let mut dirs = Vec::new();

        match target.to_lowercase().as_str() {
            "antigravity" | "gemini" => {
                dirs.push(home.join(".gemini").join("config").join("skills"));
            }
            "cursor" => {
                dirs.push(home.join(".cursor").join("skills"));
                dirs.push(PathBuf::from(".cursor").join("skills"));
            }
            "claude" => {
                dirs.push(home.join(".claude").join("skills"));
            }
            "workspace" => {
                dirs.push(PathBuf::from(".agents").join("skills"));
            }
            "all" => {
                dirs.push(home.join(".gemini").join("config").join("skills"));
                dirs.push(home.join(".cursor").join("skills"));
                dirs.push(PathBuf::from(".agents").join("skills"));
                dirs.push(PathBuf::from(".cursor").join("skills"));
            }
            _ => {
                dirs.push(PathBuf::from(target));
            }
        }

        dirs
    }

    /// Installs all DAGR skill packages into target directories
    pub fn install_skills(target: &str) -> Result<Vec<PathBuf>> {
        let target_dirs = Self::get_target_directories(target);
        let mut installed_paths = Vec::new();

        for base_dir in target_dirs {
            for skill in DAGR_SKILLS {
                let skill_dir = base_dir.join(skill.name);
                std::fs::create_dir_all(&skill_dir)?;

                let skill_file = skill_dir.join("SKILL.md");
                std::fs::write(&skill_file, skill.content.trim_start())?;
                installed_paths.push(skill_file);
            }
        }

        Ok(installed_paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_skills_in_temp_dir() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let installed = SkillsInstaller::install_skills(temp.path().to_str().unwrap())?;

        assert_eq!(installed.len(), DAGR_SKILLS.len());
        for skill in DAGR_SKILLS {
            let skill_file = temp.path().join(skill.name).join("SKILL.md");
            assert!(skill_file.exists());
            let content = std::fs::read_to_string(skill_file)?;
            assert!(content.contains(skill.name));
        }

        Ok(())
    }
}

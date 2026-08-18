use dagr_core::{DagrError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundaryRule {
    pub name: String,
    pub from: String,
    pub cannot_import: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LimitsConfig {
    pub max_file_lines: Option<usize>,
    pub max_function_lines: Option<usize>,
    pub disallow_eval: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecurityConfig {
    pub sanitize_prompt_injections: bool,
    pub strip_control_tokens: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            sanitize_prompt_injections: true,
            strip_control_tokens: vec![
                "<|im_start|>".into(),
                "<|im_end|>".into(),
                "SYSTEM:".into(),
                "SYSTEM PROMPT:".into(),
                "[INST]".into(),
                "[/INST]".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleConfig {
    pub version: String,
    #[serde(default)]
    pub project_name: Option<String>,
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub boundaries: Vec<BoundaryRule>,
    #[serde(default)]
    pub limits: LimitsConfig,
    #[serde(default)]
    pub security: SecurityConfig,
}

impl RuleConfig {
    /// Loads from `.dagr/rules.yaml` or falls back to standard preset
    pub fn load_or_default(workspace_root: &Path) -> Result<Self> {
        let rule_path = workspace_root.join(".dagr").join("rules.yaml");
        if !rule_path.exists() {
            return Ok(Self::clean_architecture_preset());
        }

        let content = std::fs::read_to_string(&rule_path)?;
        let mut config: RuleConfig = serde_yaml::from_str(&content)
            .map_err(|e| DagrError::Config(format!("Invalid .dagr/rules.yaml: {}", e)))?;

        // Apply preset boundaries if specified and boundaries list is empty
        if let Some(ref preset) = config.preset {
            if config.boundaries.is_empty() {
                config.boundaries = Self::get_preset_boundaries(preset);
            }
        }

        Ok(config)
    }

    pub fn clean_architecture_preset() -> Self {
        Self {
            version: "1.0".into(),
            project_name: Some("default-project".into()),
            preset: Some("clean-architecture".into()),
            boundaries: Self::get_preset_boundaries("clean-architecture"),
            limits: LimitsConfig {
                max_file_lines: Some(500),
                max_function_lines: Some(60),
                disallow_eval: Some(true),
            },
            security: SecurityConfig::default(),
        }
    }

    pub fn nextjs_preset() -> Self {
        Self {
            version: "1.0".into(),
            project_name: Some("nextjs-app".into()),
            preset: Some("nextjs-app".into()),
            boundaries: Self::get_preset_boundaries("nextjs-app"),
            limits: LimitsConfig::default(),
            security: SecurityConfig::default(),
        }
    }

    pub fn get_preset_boundaries(preset_name: &str) -> Vec<BoundaryRule> {
        match preset_name {
            "nextjs" | "nextjs-app" => vec![
                BoundaryRule {
                    name: "UI Isolation".into(),
                    from: "src/components/**".into(),
                    cannot_import: vec![
                        "src/db/**".into(),
                        "@prisma/client".into(),
                        "typeorm".into(),
                        "mongoose".into(),
                    ],
                    message: "UI components must not import database clients directly. Use API routes or services.".into(),
                },
                BoundaryRule {
                    name: "Client-Server Boundary".into(),
                    from: "src/app/**/page.tsx".into(),
                    cannot_import: vec!["@/server/secrets".into()],
                    message: "Client pages must not import server secrets.".into(),
                },
            ],
            _ => vec![
                BoundaryRule {
                    name: "UI-to-DB Boundary".into(),
                    from: "src/ui/**".into(),
                    cannot_import: vec!["src/db/**".into(), "src/infra/**".into()],
                    message: "Presentation layer cannot directly depend on infrastructure or database layers.".into(),
                },
                BoundaryRule {
                    name: "Domain Purity".into(),
                    from: "src/domain/**".into(),
                    cannot_import: vec!["express".into(), "fastify".into(), "src/controllers/**".into()],
                    message: "Domain entities must be pure and decoupled from HTTP presentation frameworks.".into(),
                },
            ],
        }
    }
}

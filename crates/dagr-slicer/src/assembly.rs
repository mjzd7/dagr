use crate::page_fault::HoistedSymbolContract;
use dagr_core::{count_tokens, MinimalContextSlice};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledPromptPayload {
    pub final_prompt: String,
    pub top_section_tokens: usize,
    pub middle_section_tokens: usize,
    pub bottom_section_tokens: usize,
    pub total_tokens: usize,
}

pub struct PositionAwareAssembler;

impl PositionAwareAssembler {
    pub fn new() -> Self {
        Self
    }

    /// Assembles context following the U-shaped attention curve:
    /// - TOP (High Attention): System prompt, architectural boundary rules, hoisted type contracts
    /// - MIDDLE (Low Attention): Dialogue turns and conversational history
    /// - BOTTOM (Highest Attention): User instruction, active AST slice, compiler errors
    pub fn assemble(
        &self,
        system_prompt: &str,
        architectural_rules: &[String],
        hoisted_contracts: &[HoistedSymbolContract],
        dialogue_history: &[String],
        active_slice: &MinimalContextSlice,
        user_instruction: &str,
    ) -> AssembledPromptPayload {
        let mut top_parts = Vec::new();
        top_parts.push(system_prompt.to_string());

        if !architectural_rules.is_empty() {
            top_parts.push("=== ARCHITECTURAL BOUNDARY RULES ===".to_string());
            for rule in architectural_rules {
                top_parts.push(format!("- {}", rule));
            }
        }

        if !hoisted_contracts.is_empty() {
            top_parts.push("=== HOISTED SYMBOL TYPE CONTRACTS ===".to_string());
            for contract in hoisted_contracts {
                top_parts.push(contract.signature_slice.clone());
            }
        }
        let top_str = top_parts.join("\n\n");

        let middle_str = if dialogue_history.is_empty() {
            String::new()
        } else {
            format!(
                "=== CONVERSATION HISTORY ===\n{}",
                dialogue_history.join("\n")
            )
        };

        let mut bottom_parts = Vec::new();
        bottom_parts.push("=== TARGET AST SLICE (PRIMARY CODE CONTEXT) ===".to_string());
        bottom_parts.push(format!(
            "// File: {}\n// Target: {}",
            active_slice.file_path.display(),
            active_slice.target_symbol
        ));

        let formatted_code = active_slice
            .sparse_code_lines
            .iter()
            .map(|(line_no, line_content)| format!("{:4} | {}", line_no, line_content))
            .collect::<Vec<_>>()
            .join("\n");
        bottom_parts.push(formatted_code);
        bottom_parts.push(format!("=== CURRENT USER TASK ===\n{}", user_instruction));
        let bottom_str = bottom_parts.join("\n\n");

        let top_tokens = count_tokens(&top_str);
        let middle_tokens = count_tokens(&middle_str);
        let bottom_tokens = count_tokens(&bottom_str);

        let final_prompt = format!("{}\n\n{}\n\n{}", top_str, middle_str, bottom_str);
        let total_tokens = top_tokens + middle_tokens + bottom_tokens;

        AssembledPromptPayload {
            final_prompt,
            top_section_tokens: top_tokens,
            middle_section_tokens: middle_tokens,
            bottom_section_tokens: bottom_tokens,
            total_tokens,
        }
    }
}

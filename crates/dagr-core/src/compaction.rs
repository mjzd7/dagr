use crate::error::Result;
use crate::token::count_tokens;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTurn {
    pub role: String,
    pub content: String,
    pub token_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub system_prompt: String,
    pub turns: Vec<DialogueTurn>,
    pub max_capacity_tokens: usize,
    pub watermark_threshold: f64,
}

impl ContextWindow {
    pub fn new(
        system_prompt: String,
        max_capacity_tokens: usize,
        watermark_threshold: f64,
    ) -> Self {
        Self {
            system_prompt,
            turns: Vec::new(),
            max_capacity_tokens,
            watermark_threshold,
        }
    }

    pub fn total_tokens(&self) -> usize {
        let sys_tokens = count_tokens(&self.system_prompt);
        let turns_tokens: usize = self.turns.iter().map(|t| t.token_count).sum();
        sys_tokens + turns_tokens
    }

    pub fn utilization_ratio(&self) -> f64 {
        if self.max_capacity_tokens == 0 {
            return 0.0;
        }
        self.total_tokens() as f64 / self.max_capacity_tokens as f64
    }

    pub fn should_compact(&self) -> bool {
        self.utilization_ratio() >= self.watermark_threshold
    }

    pub fn add_turn(&mut self, role: &str, content: &str) -> Result<()> {
        let tokens = count_tokens(content);
        self.turns.push(DialogueTurn {
            role: role.to_string(),
            content: content.to_string(),
            token_count: tokens,
        });
        Ok(())
    }

    /// Compacts older turns [0..k] into a synthesized turn, retaining recent turns
    pub fn compact_window(&self, retain_recent_turns: usize) -> ContextWindow {
        if self.turns.len() <= retain_recent_turns {
            return self.clone();
        }

        let split_idx = self.turns.len().saturating_sub(retain_recent_turns);
        let older_turns = &self.turns[..split_idx];
        let recent_turns = &self.turns[split_idx..];

        let mut summary_lines = Vec::new();
        for t in older_turns {
            summary_lines.push(format!("[{}] {}", t.role, t.content));
        }
        let summarized_content = format!(
            "<!-- COMPACTED HISTORICAL CONTEXT ({} turns) -->\n{}",
            older_turns.len(),
            summary_lines.join("\n")
        );
        let summary_tokens = count_tokens(&summarized_content);

        let mut new_turns = Vec::new();
        new_turns.push(DialogueTurn {
            role: "system:compacted_history".into(),
            content: summarized_content,
            token_count: summary_tokens,
        });
        new_turns.extend_from_slice(recent_turns);

        ContextWindow {
            system_prompt: self.system_prompt.clone(),
            turns: new_turns,
            max_capacity_tokens: self.max_capacity_tokens,
            watermark_threshold: self.watermark_threshold,
        }
    }
}

pub struct AsyncCompactionTracker {
    is_compacting: Arc<AtomicBool>,
}

impl Default for AsyncCompactionTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncCompactionTracker {
    pub fn new() -> Self {
        Self {
            is_compacting: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn try_start_compaction(&self) -> bool {
        self.is_compacting
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    pub fn finish_compaction(&self) {
        self.is_compacting.store(false, Ordering::SeqCst);
    }
}

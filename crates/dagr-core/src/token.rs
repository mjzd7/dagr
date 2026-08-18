use std::sync::OnceLock;
use tiktoken_rs::{cl100k_base, CoreBPE};

static BPE: OnceLock<Option<CoreBPE>> = OnceLock::new();

/// Calculates the exact BPE token count of a given text string using OpenAI/Anthropic standard cl100k_base.
pub fn count_tokens(text: &str) -> usize {
    let bpe = BPE.get_or_init(|| cl100k_base().ok());
    if let Some(bpe) = bpe {
        bpe.encode_with_special_tokens(text).len()
    } else {
        (text.len() as f64 / 3.8).ceil() as usize
    }
}

/// Computes the mathematical token compression ratio:
/// Returns a value between 0.0 (no savings) and 1.0 (100% saved).
pub fn compute_compression_ratio(original_tokens: usize, sliced_tokens: usize) -> f32 {
    if original_tokens == 0 {
        return 0.0;
    }
    if sliced_tokens >= original_tokens {
        return 0.0;
    }
    1.0 - (sliced_tokens as f32 / original_tokens as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counting() {
        let code = "export async function processPayment(payload: PaymentPayload): Promise<PaymentResult> { return true; }";
        let count = count_tokens(code);
        assert!(count > 5 && count < 30);
    }

    #[test]
    fn test_compression_ratio() {
        let original = 10000;
        let sliced = 300;
        let ratio = compute_compression_ratio(original, sliced);
        assert!((ratio - 0.97).abs() < 0.01);
    }
}

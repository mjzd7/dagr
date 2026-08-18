//! Fuzzy identifier tokenization, Jaro-Winkler similarity, and multi-tier relevance ranking

/// Splits identifiers across camelCase, PascalCase, snake_case, kebab-case, and dot notation boundaries
pub fn tokenize_identifier(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = input.chars().collect();

    for i in 0..chars.len() {
        let ch = chars[i];

        if ch == '_'
            || ch == '-'
            || ch == '.'
            || ch == '/'
            || ch == '\\'
            || ch == ':'
            || ch.is_whitespace()
        {
            if !current.is_empty() {
                tokens.push(current.to_lowercase());
                current.clear();
            }
            continue;
        }

        // Detect camelCase / PascalCase transitions: e.g. "calculateMonthly" -> "calculate", "Monthly"
        if ch.is_uppercase() {
            let prev_is_lower = i > 0 && chars[i - 1].is_lowercase();
            let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();

            if (prev_is_lower || (next_is_lower && current.len() > 1)) && !current.is_empty() {
                tokens.push(current.to_lowercase());
                current.clear();
            }
        }

        current.push(ch);
    }

    if !current.is_empty() {
        tokens.push(current.to_lowercase());
    }

    tokens
}

/// Computes the Jaro similarity score between two strings (0.0 to 1.0)
pub fn jaro_similarity(s1: &str, s2: &str) -> f64 {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    if len1 == 0 && len2 == 0 {
        return 1.0;
    }
    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    let match_distance = (len1.max(len2) / 2).saturating_sub(1);

    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];

    let mut matches = 0;
    let mut transpositions = 0;

    for i in 0..len1 {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(len2);

        for j in start..end {
            if s2_matches[j] || s1_chars[i] != s2_chars[j] {
                continue;
            }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let m = matches as f64;
    ((m / len1 as f64) + (m / len2 as f64) + ((m - (transpositions as f64 / 2.0)) / m)) / 3.0
}

/// Computes the Jaro-Winkler similarity score with standard prefix scaling
pub fn jaro_winkler(s1: &str, s2: &str) -> f64 {
    let jaro = jaro_similarity(s1, s2);
    if jaro < 0.7 {
        return jaro;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let max_prefix = 4.min(s1_chars.len()).min(s2_chars.len());
    let mut prefix_len = 0;

    for i in 0..max_prefix {
        if s1_chars[i] == s2_chars[i] {
            prefix_len += 1;
        } else {
            break;
        }
    }

    let scaling_factor = 0.1;
    jaro + (prefix_len as f64 * scaling_factor * (1.0 - jaro))
}

/// Computes a composite match score (0 to 100) between a user query and a candidate symbol
pub fn compute_symbol_match_score(
    query: &str,
    symbol_name: &str,
    file_path: &str,
    docstring: Option<&str>,
) -> usize {
    let query_clean = query.trim().to_lowercase();
    let symbol_clean = symbol_name.trim().to_lowercase();

    // 1. Exact match (100)
    if query_clean == symbol_clean {
        return 100;
    }

    // 2. Direct substring / prefix / suffix (85-95)
    if symbol_clean.starts_with(&query_clean) {
        return 95;
    }
    if symbol_clean.contains(&query_clean) {
        return 85;
    }
    if query_clean.contains(&symbol_clean) {
        return 80;
    }

    // 3. Token set intersection (e.g. "monthly discount" vs "calculateMonthlyDiscounts")
    let query_tokens = tokenize_identifier(query);
    let symbol_tokens = tokenize_identifier(symbol_name);

    if !query_tokens.is_empty() && !symbol_tokens.is_empty() {
        let mut matched_tokens = 0;
        for q_tok in &query_tokens {
            if symbol_tokens.iter().any(|s_tok| {
                s_tok == q_tok
                    || s_tok.starts_with(q_tok)
                    || q_tok.starts_with(s_tok)
                    || jaro_winkler(q_tok, s_tok) >= 0.82
            }) {
                matched_tokens += 1;
            }
        }

        if matched_tokens == query_tokens.len() {
            return 90;
        } else if matched_tokens > 0 {
            let ratio = matched_tokens as f64 / query_tokens.len() as f64;
            let token_score = (ratio * 80.0) as usize;
            if token_score >= 50 {
                return token_score;
            }
        }
    }

    // 4. Jaro-Winkler fuzzy metric
    let jw = jaro_winkler(&query_clean, &symbol_clean);
    if jw >= 0.80 {
        return (jw * 85.0) as usize;
    }

    // 5. Docstring / file path relevance
    if let Some(doc) = docstring {
        if doc.to_lowercase().contains(&query_clean) {
            return 70;
        }
    }
    if file_path.to_lowercase().contains(&query_clean) {
        return 50;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_identifier_casing() {
        assert_eq!(
            tokenize_identifier("calculateMonthlyDiscounts"),
            vec!["calculate", "monthly", "discounts"]
        );
        assert_eq!(
            tokenize_identifier("calc_monthly_discount_v2"),
            vec!["calc", "monthly", "discount", "v2"]
        );
        assert_eq!(
            tokenize_identifier("LocalIndexStore::search_symbols"),
            vec!["local", "index", "store", "search", "symbols"]
        );
    }

    #[test]
    fn test_jaro_winkler_similarity() {
        let score = jaro_winkler("calculate", "calculate");
        assert_eq!(score, 1.0);

        let typo_score = jaro_winkler("calclate", "calculate");
        assert!(typo_score > 0.9);

        let diff_score = jaro_winkler("calculate", "zzzzzz");
        assert!(diff_score < 0.2);
    }

    #[test]
    fn test_compute_symbol_match_score() {
        assert_eq!(
            compute_symbol_match_score("calculate", "calculate", "math.rs", None),
            100
        );
        assert_eq!(
            compute_symbol_match_score("calculate", "calculateMonthly", "math.rs", None),
            95
        );
        assert!(
            compute_symbol_match_score(
                "monthly discount",
                "calculateMonthlyDiscounts",
                "math.rs",
                None
            ) >= 80
        );
    }
}

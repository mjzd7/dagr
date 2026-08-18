use dagr_core::{DagrError, Language, Result};
use tree_sitter::{Language as TsLanguage, Parser, Tree};

pub struct AstParser {
    parser: Parser,
    pub language: Language,
}

impl AstParser {
    pub fn new(language: Language) -> Result<Self> {
        let mut parser = Parser::new();
        let ts_lang: TsLanguage = match language {
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            Language::Go => tree_sitter_go::LANGUAGE.into(),
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::Unknown => {
                return Err(DagrError::UnsupportedLanguage("Unknown or unsupported language extension".into()));
            }
        };

        parser
            .set_language(&ts_lang)
            .map_err(|e| DagrError::ParserInit(e.to_string()))?;

        Ok(Self { parser, language })
    }

    /// Parses source code into a Tree-sitter syntax tree
    pub fn parse(&mut self, source_code: &str, old_tree: Option<&Tree>) -> Result<Tree> {
        self.parser
            .parse(source_code, old_tree)
            .ok_or_else(|| DagrError::ParseFailure("Tree-sitter parser failed to produce AST".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_parsing() -> Result<()> {
        let mut parser = AstParser::new(Language::TypeScript)?;
        let code = "function add(a: number, b: number): number { return a + b; }";
        let tree = parser.parse(code, None)?;
        assert_eq!(tree.root_node().kind(), "program");
        Ok(())
    }

    #[test]
    fn test_python_parsing() -> Result<()> {
        let mut parser = AstParser::new(Language::Python)?;
        let code = "def calculate_discount(price: float, rate: float) -> float:\n    return price * (1 - rate)\n";
        let tree = parser.parse(code, None)?;
        assert_eq!(tree.root_node().kind(), "module");
        Ok(())
    }
}

pub mod error;
pub mod extractor;
pub mod lexer;
pub mod normalizer;
pub mod parser;
pub mod types;

use error::{ParseError, ParseWarning};
use serde::Serialize;
use types::{Fragment, ParseNode};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    pub fragments: Vec<NormalizedFragment>,
    pub warnings: Vec<ParseWarning>,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedFragment {
    pub index: usize,
    pub source: Fragment,
    pub tree: ParseNode,
    pub json: String,
}

pub fn parse(input: &str) -> ParseResult {
    let mut fragments = Vec::new();
    let mut errors = Vec::new();

    let extracted = extractor::extract(input);

    for (i, ext) in extracted.into_iter().enumerate() {
        let mut parser = parser::Parser::new(&ext.raw);
        match parser.parse() {
            Ok(tree) => {
                let json = normalizer::normalize(&tree);
                fragments.push(NormalizedFragment {
                    index: i,
                    source: ext,
                    tree,
                    json,
                });
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }

    ParseResult {
        fragments,
        warnings: vec![],
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_json_round_trip() {
        let input = r#"{"a": 1, "b": [true, null, "test"]}"#;
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.fragments.len(), 1);
        let expected = r#"{"a":1,"b":[true,null,"test"]}"#;
        assert_eq!(result.fragments[0].json, expected);
    }

    #[test]
    fn test_mixed_separators() {
        let input = r#"{a: 1, b = 2, c => 3, d -> 4}"#;
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        let expected = r#"{"a":1,"b":2,"c":3,"d":4}"#;
        assert_eq!(result.fragments[0].json, expected);
    }

    #[test]
    fn test_mixed_quoting() {
        let input = r#"{ "a": 1, 'b': 2, c: 3 }"#;
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        let expected = r#"{"a":1,"b":2,"c":3}"#;
        assert_eq!(result.fragments[0].json, expected);
    }

    #[test]
    fn test_unquoted_boundary_heuristic() {
        let input = r#"{name: FlexJSON "Neon, test: a}"#;
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        let expected = r#"{"name":"FlexJSON \"Neon","test":"a"}"#;
        assert_eq!(result.fragments[0].json, expected);
    }

    #[test]
    fn test_embedded_commas() {
        let input = r#"{x: a,b,c, y: z}"#;
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        let expected = r#"{"x":"a,b,c","y":"z"}"#;
        assert_eq!(result.fragments[0].json, expected);
    }

    #[test]
    fn test_multi_fragment() {
        let input = "Here is some text {a: 1} and another [2, 3]";
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.fragments.len(), 2);
        assert_eq!(result.fragments[0].json, r#"{"a":1}"#);
        assert_eq!(result.fragments[1].json, r#"[2,3]"#);
    }
}

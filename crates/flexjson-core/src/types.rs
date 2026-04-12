use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum QuoteStyle {
    Double,
    Single,
    None,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind")]
pub enum ParseNode {
    #[serde(rename = "object")]
    Object { children: Vec<MemberNode>, raw: String },
    #[serde(rename = "array")]
    Array { elements: Vec<ParseNode>, raw: String },
    #[serde(rename = "string")]
    String { value: String, #[serde(rename = "quoteStyle")] quote_style: QuoteStyle, raw: String },
    #[serde(rename = "number")]
    Number { value: f64, raw: String },
    #[serde(rename = "boolean")]
    Boolean { value: bool, raw: String },
    #[serde(rename = "null")]
    Null { raw: String },
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MemberNode {
    pub key: ParseNode,
    pub separator: String,
    pub value: ParseNode,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fragment {
    pub raw: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub start_line: usize,
    pub start_col: usize,
    #[serde(rename = "type")]
    pub fragment_type: String, // "object" | "array"
}

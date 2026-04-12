use crate::types::{ParseNode, QuoteStyle};

pub fn normalize(node: &ParseNode) -> String {
    match node {
        ParseNode::Object { children, .. } => {
            let mut out = String::new();
            out.push('{');
            for (i, child) in children.iter().enumerate() {
                out.push_str(&normalize(&child.key));
                out.push(':');
                out.push_str(&normalize(&child.value));
                if i < children.len() - 1 {
                    out.push(',');
                }
            }
            out.push('}');
            out
        }
        ParseNode::Array { elements, .. } => {
            let mut out = String::new();
            out.push('[');
            for (i, el) in elements.iter().enumerate() {
                out.push_str(&normalize(el));
                if i < elements.len() - 1 {
                    out.push(',');
                }
            }
            out.push(']');
            out
        }
        ParseNode::String {
            value, quote_style, ..
        } => {
            if *quote_style == QuoteStyle::None {
                normalize_unquoted(value)
            } else {
                serde_json::to_string(&unescape_string(value)).unwrap()
            }
        }
        ParseNode::Number { value, raw } => {
            if value.fract() == 0.0 && raw.contains('.') == false {
                format!("{}", *value as i64)
            } else {
                serde_json::to_string(value).unwrap()
            }
        }
        ParseNode::Boolean { value, .. } => {
            if *value {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        ParseNode::Null { .. } => "null".to_string(),
    }
}

fn unescape_string(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('\'') => out.push('\''),
                Some('b') => out.push('\x08'),
                Some('f') => out.push('\x0C'),
                Some('u') => {
                    let mut hex = String::new();
                    for _ in 0..4 {
                        if let Some(hc) = chars.next() {
                            hex.push(hc);
                        }
                    }
                    if let Ok(cp) = u32::from_str_radix(&hex, 16) {
                        if let Some(cc) = std::char::from_u32(cp) {
                            out.push(cc);
                        }
                    }
                }
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn normalize_unquoted(raw: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' || ch == '\'' {
            let mut has_close = false;
            let mut j = i + 1;
            while j < chars.len() {
                if chars[j] == ch {
                    has_close = true;
                    break;
                }
                j += 1;
            }

            if has_close {
                let q = ch;
                i += 1;
                while i < chars.len() && chars[i] != q {
                    if chars[i] == '"' {
                        result.push_str("\\\"");
                    } else if chars[i] == '\\' && i + 1 < chars.len() {
                        result.push('\\');
                        result.push(chars[i + 1]);
                        i += 1;
                    } else {
                        escape_char_to(chars[i], &mut result);
                    }
                    i += 1;
                }
                i += 1; // skip closing quote
            } else {
                escape_char_to(ch, &mut result);
                i += 1;
            }
        } else {
            escape_char_to(ch, &mut result);
            i += 1;
        }
    }
    format!("\"{}\"", result)
}

fn escape_char_to(c: char, out: &mut String) {
    match c {
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        '\\' => out.push_str("\\\\"),
        '\x08' => out.push_str("\\b"),
        '\x0C' => out.push_str("\\f"),
        '"' => out.push_str("\\\""), // safety measure, though handled above
        _ => out.push(c),
    }
}

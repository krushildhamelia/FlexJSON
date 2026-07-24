use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\r]+")]
#[logos(skip r"//[^\n]*\n?")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[regex(r":|=>|->|=", priority = 5)]
    Separator,

    #[regex(r#""([^"\\]|\\.)*""#)]
    StringDouble,

    #[regex(r#"'([^'\\]|\\.)*'"#)]
    StringSingle,

    #[regex(r#"`([^`\\]|\\.)*`"#)]
    StringBacktick,

    #[regex(
        r"[+-]?([0-9]+(_[0-9]+)*)(\.[0-9]+(_[0-9]+)*)?([eE][+-]?[0-9]+(_[0-9]+)*)?",
        priority = 4
    )]
    #[regex(r"0x[0-9a-fA-F]+(_[0-9a-fA-F]+)*", priority = 4)]
    #[regex(r"[+-]?\.[0-9]+(_[0-9]+)*([eE][+-]?[0-9]+(_[0-9]+)*)?", priority = 4)]
    #[regex(r"[+-]?[0-9]+(_[0-9]+)*\.", priority = 5)]
    Number,

    #[regex("(?i:true|false|yes|no|on|off)", priority = 10)]
    Boolean,

    #[regex("(?i:null|nil|none|undefined|nan|[+-]?infinity)", priority = 10)]
    Null,

    // First char must not be structural, whitespace, quote, comment start.
    #[regex(r#"[^\{\}\[\]:,\s"'/=]+"#, lex_unquoted, priority = 1)]
    Unquoted(String),

    // ISO 8601 Date regex
    #[regex(r"[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9](T[0-9][0-9]:[0-9][0-9]:[0-9][0-9](\.[0-9]+)?(Z|[+-][0-9][0-9]:?[0-9][0-9])?)?", priority = 6)]
    Date,

    Unknown,
}

fn lex_unquoted<'a>(lex: &mut Lexer<'a, Token>) -> Option<String> {
    let remainder = lex.remainder();
    let bytes = remainder.as_bytes();
    let mut i = 0;
    let mut in_quote: Option<u8> = None;

    while i < bytes.len() {
        let b = bytes[i];

        if let Some(q) = in_quote {
            if b == q {
                let mut backslashes = 0;
                let mut j = i.saturating_sub(1);
                while j > 0 && bytes[j] == b'\\' {
                    backslashes += 1;
                    j -= 1;
                }
                if bytes[0] == b'\\' && j == 0 {
                    backslashes += 1;
                }
                if backslashes % 2 == 0 {
                    in_quote = None;
                }
            }
            i += 1;
            continue;
        }

        if b == b'"' || b == b'\'' || b == b'`' {
            let mut has_close = false;
            let mut j = i + 1;
            while j < bytes.len() {
                if bytes[j] == b {
                    has_close = true;
                    break;
                }
                j += 1;
            }
            if has_close {
                in_quote = Some(b);
            }
            i += 1;
            continue;
        }

        if b == b'/' && i + 1 < bytes.len() && (bytes[i + 1] == b'/' || bytes[i + 1] == b'*') {
            break;
        }

        if b == b'}' || b == b']' || b == b'\n' || b == b'\r' || b == b':' || b == b'=' || b == b'>' {
            break;
        }
        
        if b == b'-' && i + 1 < bytes.len() && bytes[i + 1] == b'>' {
            break;
        }

        if b == b',' {
            if is_value_boundary(&remainder[i..]) {
                break;
            }
        }

        i += 1;
    }

    lex.bump(i);
    let matched = lex.slice();
    Some(matched.trim_end().to_string())
}

fn is_value_boundary(input: &str) -> bool {
    if !input.starts_with(',') {
        return false;
    }
    
    let rest = &input[1..];
    let mut chars = rest.char_indices().peekable();
    
    // Skip whitespace
    while let Some(&(_, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
    
    let start_idx = match chars.peek() {
        Some(&(idx, _)) => idx,
        None => return false,
    };
    
    // Determine token type
    let first_char = chars.peek().unwrap().1;
    let token_end_idx = if first_char == '"' || first_char == '\'' || first_char == '`' {
        let quote = first_char;
        chars.next(); // consume opening quote
        let mut escaped = false;
        let mut closed = false;
        let mut end_idx = None;
        while let Some(&(idx, c)) = chars.peek() {
            chars.next();
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote {
                closed = true;
                end_idx = Some(idx + 1);
                break;
            }
        }
        if !closed {
            return false;
        }
        end_idx.unwrap()
    } else {
        // Unquoted key (or number, boolean, null)
        let mut end_idx = start_idx;
        while let Some(&(idx, c)) = chars.peek() {
            // Check if it starts a separator -> or =>
            if c == '-' || c == '=' {
                let mut temp_chars = chars.clone();
                temp_chars.next();
                if let Some((_, '>')) = temp_chars.peek() {
                    break;
                }
            }
            
            if c.is_whitespace() 
                || c == '{' || c == '}' 
                || c == '[' || c == ']' 
                || c == ':' || c == '=' 
                || c == ',' || c == '"' 
                || c == '\'' || c == '`' || c == '/' 
            {
                break;
            }
            chars.next();
            end_idx = idx + c.len_utf8();
        }
        if end_idx == start_idx {
            return false; // empty key
        }
        end_idx
    };
    
    // Skip whitespace after the key
    let after_key = &rest[token_end_idx..];
    let mut after_chars = after_key.chars().peekable();
    while let Some(&c) = after_chars.peek() {
        if c.is_whitespace() {
            after_chars.next();
        } else {
            break;
        }
    }
    
    // Check if the remaining string starts with a separator: : or = or => or ->
    let remaining: String = after_chars.collect();
    remaining.starts_with(':') 
        || remaining.starts_with('=') 
        || remaining.starts_with("=>") 
        || remaining.starts_with("->")
}

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

    #[regex(r#""([^"\\]|\\["\\/bfnrt]|\\u[0-9a-fA-F]{4})*""#)]
    StringDouble,

    #[regex(r#"'([^'\\]|\\['\\/bfnrt]|\\u[0-9a-fA-F]{4})*'"#)]
    StringSingle,

    #[regex(
        r"[+-]?([0-9]+(_[0-9]+)*)(\.[0-9]+(_[0-9]+)*)?([eE][+-]?[0-9]+(_[0-9]+)*)?",
        priority = 4
    )]
    #[regex(r"0x[0-9a-fA-F]+(_[0-9a-fA-F]+)*", priority = 4)]
    Number,

    #[regex("(?i:true|false|yes|no|on|off)", priority = 10)]
    Boolean,

    #[regex("(?i:null|nil|none|undefined)", priority = 10)]
    Null,

    // First char must not be structural, whitespace, quote, comment start.
    #[regex(r#"[^\{\}\[\]:,\s"'/=]+"#, lex_unquoted, priority = 1)]
    Unquoted(String),

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

        if b == b'"' || b == b'\'' {
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
    let mut lex = Token::lexer(&input[1..]);
    if let Some(Ok(tok)) = lex.next() {
        match tok {
            Token::StringDouble
            | Token::StringSingle
            | Token::Number
            | Token::Boolean
            | Token::Null
            | Token::Unquoted(_) => {
                let mut lex2 = Token::lexer(lex.remainder());
                matches!(lex2.next(), Some(Ok(Token::Separator)))
            }
            _ => false,
        }
    } else {
        false
    }
}

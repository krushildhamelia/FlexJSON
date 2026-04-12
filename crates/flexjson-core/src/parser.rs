use crate::error::ParseError;
use crate::lexer::Token;
use crate::types::{MemberNode, ParseNode, QuoteStyle};
use logos::{Lexer, Logos};

pub struct Parser<'a> {
    lex: Lexer<'a, Token>,
    current: Option<Token>,
    input: &'a str,
    last_end: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lex = Token::lexer(input);
        let current = lex.next().and_then(Result::ok);
        Self {
            lex,
            current,
            input,
            last_end: 0,
        }
    }

    fn advance(&mut self) {
        self.last_end = self.lex.span().end;
        self.current = match self.lex.next() {
            Some(Ok(t)) => Some(t),
            Some(Err(_)) => Some(Token::Unknown),
            None => None,
        };
    }

    pub fn parse(&mut self) -> Result<ParseNode, ParseError> {
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<ParseNode, ParseError> {
        let tok = match &self.current {
            Some(t) => t.clone(),
            None => return Err(ParseError::UnexpectedToken("EOF".to_string())),
        };

        match tok {
            Token::LBrace => self.parse_object(),
            Token::LBracket => self.parse_array(),
            Token::StringDouble => {
                let raw = self.lex.slice().to_string();
                let value = raw[1..raw.len() - 1].to_string();
                self.advance();
                Ok(ParseNode::String {
                    value,
                    quote_style: QuoteStyle::Double,
                    raw,
                })
            }
            Token::StringSingle => {
                let raw = self.lex.slice().to_string();
                let value = raw[1..raw.len() - 1].to_string();
                self.advance();
                Ok(ParseNode::String {
                    value,
                    quote_style: QuoteStyle::Single,
                    raw,
                })
            }
            Token::Unquoted(val) => {
                let raw = self.lex.slice().to_string();
                self.advance();
                Ok(ParseNode::String {
                    value: val,
                    quote_style: QuoteStyle::None,
                    raw,
                })
            }
            Token::Number => {
                let raw = self.lex.slice().to_string();
                let val_str = raw.replace("_", "").to_lowercase();
                let value = if val_str.starts_with("0x") {
                    i64::from_str_radix(&val_str[2..], 16).unwrap_or(0) as f64
                } else if val_str.ends_with(".") {
                    val_str[..val_str.len() - 1]
                        .parse::<f64>()
                        .unwrap_or(0.0)
                } else {
                    val_str.parse::<f64>().unwrap_or(0.0)
                };
                self.advance();
                Ok(ParseNode::Number { value, raw })
            }
            Token::Boolean => {
                let raw = self.lex.slice().to_string();
                let r_lower = raw.to_lowercase();
                let value = r_lower == "true" || r_lower == "yes" || r_lower == "on";
                self.advance();
                Ok(ParseNode::Boolean { value, raw })
            }
            Token::Null => {
                let raw = self.lex.slice().to_string();
                self.advance();
                Ok(ParseNode::Null { raw })
            }
            _ => Err(ParseError::UnexpectedToken(format!("{:?}", tok))),
        }
    }

    fn parse_object(&mut self) -> Result<ParseNode, ParseError> {
        let start_pos = self.lex.span().start;
        self.advance(); // consume {

        let mut children = Vec::new();

        while let Some(tok) = &self.current {
            if *tok == Token::RBrace {
                break;
            }

            let key_node = match self.parse_value() {
                Ok(n) => n,
                Err(_) => {
                    // Try to skip to next member
                    self.advance();
                    continue;
                }
            };

            let separator = if let Some(Token::Separator) = self.current {
                let sep = self.lex.slice().to_string();
                self.advance();
                sep
            } else {
                ":".to_string() // implicit separator
            };

            let value_node = match self.parse_value() {
                Ok(n) => n,
                Err(_) => ParseNode::Null {
                    raw: "null".to_string(),
                },
            };

            children.push(MemberNode {
                key: key_node,
                separator,
                value: value_node,
                raw: String::new(), // Will fill in below if needed, or omit.
            });

            if let Some(Token::Comma) = self.current {
                self.advance();
            }
        }

        if let Some(Token::RBrace) = self.current {
            self.advance();
        }

        let end_pos = self.last_end;
        let raw = if start_pos < end_pos && end_pos <= self.input.len() {
            self.input[start_pos..end_pos].to_string()
        } else {
            "".to_string()
        };

        Ok(ParseNode::Object { children, raw })
    }

    fn parse_array(&mut self) -> Result<ParseNode, ParseError> {
        let start_pos = self.lex.span().start;
        self.advance(); // consume [

        let mut elements = Vec::new();

        while let Some(tok) = &self.current {
            if *tok == Token::RBracket {
                break;
            }

            if let Ok(val) = self.parse_value() {
                elements.push(val);
            } else {
                self.advance();
                continue;
            }

            if let Some(Token::Comma) = self.current {
                self.advance();
            }
        }

        if let Some(Token::RBracket) = self.current {
            self.advance();
        }

        let end_pos = self.last_end;
        let raw = if start_pos < end_pos && end_pos <= self.input.len() {
            self.input[start_pos..end_pos].to_string()
        } else {
            "".to_string()
        };

        Ok(ParseNode::Array { elements, raw })
    }
}

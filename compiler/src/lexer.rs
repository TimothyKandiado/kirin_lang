use std::fmt;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TokenKind {
    None,
    // Literals
    NumberLiteral,
    StringLiteral,

    Identifier,

    // Types
    I64,
    F64,
    Bool,
    Void,
    Any,
    Str,

    // Unary,
    Not,
    Neg,

    // Binary
    Plus,
    Minus,
    Slash,
    Star,
    Mod,
    Caret,

    Equal,

    // Cmp
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    EqualEqual,
    NotEqual,

    And,
    Or,

    // Keywords
    Fn,
    Return,
    If,
    Else,
    For,
    Pub,
    Native,
    True,
    False,
    Package,

    // Brackets
    ParenLeft,
    ParenRight,
    BraceLeft,
    BraceRight,
    SquareLeft,
    SquareRight,

    // Delimiters
    NewLine,
    Colon,
    Comma,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub line: usize,
    pub column: usize,
}

impl Token<'_> {
    pub fn none() -> Self {
        Token {
            kind: TokenKind::None,
            lexeme: "",
            line: 0,
            column: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanError {
    pub line: usize,
    pub column: usize,
    pub context: String,
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ScanError at line {}, column {}: {}",
            self.line, self.column, self.context
        )
    }
}

pub fn parse_tokens(source: &str) -> Result<Vec<Token<'_>>, Vec<ScanError>> {
    let parser = Parser {
        current: 0,
        column: 0,
        line: 1,
        start: 0,
        source,
        errors: Vec::new(),
        tokens: Vec::new(),
    };

    parser.scan_tokens()
}

struct Parser<'a> {
    current: usize,
    start: usize,
    source: &'a str,
    line: usize,
    column: usize,
    errors: Vec<ScanError>,
    tokens: Vec<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn scan_tokens(mut self) -> Result<Vec<Token<'a>>, Vec<ScanError>> {
        while !self.is_at_end() {
            self.scan_token();
        }

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        self.emit_current_simple_token(TokenKind::Eof);

        Ok(self.tokens)
    }

    fn scan_token(&mut self) {
        self.skip_white_space();

        self.start = self.current;

        if self.is_at_end() {
            return;
        }

        let current_char = self.advance();

        match current_char {
            '+' => {
                self.emit_current_simple_token(TokenKind::Plus);
            }
            '-' => {
                self.emit_current_simple_token(TokenKind::Minus);
            }
            '/' => {
                self.emit_current_simple_token(TokenKind::Slash);
            }
            '*' => {
                self.emit_current_simple_token(TokenKind::Star);
            }
            '%' => {
                self.emit_current_simple_token(TokenKind::Mod);
            }
            '^' => {
                self.emit_current_simple_token(TokenKind::Caret);
            }

            '<' => {
                if self.match_char('=') {
                    self.emit_current_simple_token(TokenKind::LessEqual);
                } else {
                    self.emit_current_simple_token(TokenKind::Less);
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.emit_current_simple_token(TokenKind::GreaterEqual);
                } else {
                    self.emit_current_simple_token(TokenKind::Greater);
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.emit_current_simple_token(TokenKind::EqualEqual);
                } else {
                    self.emit_current_simple_token(TokenKind::Equal);
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.emit_current_simple_token(TokenKind::NotEqual);
                } else {
                    self.emit_current_simple_token(TokenKind::Not);
                }
            }

            '(' => {
                self.emit_current_simple_token(TokenKind::ParenLeft);
            }
            ')' => {
                self.emit_current_simple_token(TokenKind::ParenRight);
            }

            '{' => {
                self.emit_current_simple_token(TokenKind::BraceLeft);
            }
            '}' => {
                self.emit_current_simple_token(TokenKind::BraceRight);
            }
            '[' => {
                self.emit_current_simple_token(TokenKind::SquareLeft);
            }
            ']' => {
                self.emit_current_simple_token(TokenKind::SquareRight);
            }
            ':' => self.emit_current_simple_token(TokenKind::Colon),
            ',' => self.emit_current_simple_token(TokenKind::Comma),

            '"' => self.scan_string(),

            x if x.is_ascii_digit() => self.scan_number(),

            x if is_identifier_start(x) => self.scan_identifier(),

            _ => self.emit_current_error(format!("Unexpected character '{}'", current_char)),
        }
    }

    fn scan_number(&mut self) {
        let line = self.line;
        let column = self.column - 1;

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        let next = self.peek();
        // if next character is a decimal point consume all remaining digits
        if next == '.' {
            self.advance();
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let segment = &self.source[self.start..self.current];

        self.emit_token(TokenKind::NumberLiteral, segment, line, column);
    }

    fn scan_identifier(&mut self) {
        let line = self.line;
        let column = self.column - 1;

        while !self.is_at_end() && is_identifier_rest(self.peek()) {
            self.advance();
        }

        let segment = &self.source[self.start..self.current];

        match segment {
            "fn" => self.emit_token(TokenKind::Fn, "", line, column),
            "if" => self.emit_token(TokenKind::If, "", line, column),
            "else" => self.emit_token(TokenKind::Else, "", line, column),
            "for" => self.emit_token(TokenKind::For, "", line, column),
            "i64" => self.emit_token(TokenKind::I64, "", line, column),
            "f64" => self.emit_token(TokenKind::F64, "", line, column),
            "void" => self.emit_token(TokenKind::Void, "", line, column),
            "any" => self.emit_token(TokenKind::Any, "", line, column),
            "string" => self.emit_token(TokenKind::Str, "", line, column),
            "bool" => self.emit_token(TokenKind::Bool, "", line, column),
            "true" => self.emit_token(TokenKind::True, "", line, column),
            "false" => self.emit_token(TokenKind::False, "", line, column),
            "pub" => self.emit_token(TokenKind::Pub, "", line, column),
            "native" => self.emit_token(TokenKind::Native, "", line, column),
            "package" => self.emit_token(TokenKind::Package, "", line, column),
            "return" => self.emit_token(TokenKind::Return, "", line, column),

            _ => self.emit_token(TokenKind::Identifier, segment, line, column),
        }
    }

    fn scan_string(&mut self) {
        let line = self.line;
        let column = self.column;

        self.start = self.current;

        while !self.is_at_end() && self.peek() != '"' {
            let c = self.advance();

            if c == '\\' {
                _ = self.advance();
            }
        }

        let segment = &self.source[self.start..self.current];
        self.consume('"', "expected '\"' to end string literal".to_string());

        self.emit_token(TokenKind::StringLiteral, segment, line, column);
    }

    fn skip_white_space(&mut self) {
        let mut new_line_emitted = false;

        while !self.is_at_end() {
            let c = self.peek();

            match c {
                '\r' | ' ' => _ = self.advance(),
                '\n' => {
                    if !new_line_emitted {
                        self.emit_token(TokenKind::NewLine, "", self.line, self.column);
                        new_line_emitted = true;
                    }

                    self.line += 1;
                    self.column = 0;
                    _ = self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while !self.is_at_end() && self.peek() != '\n' {
                            _ = self.advance();
                        }
                        _ = self.advance();
                        self.line += 1;
                        self.column = 0;
                    } else {
                        break;
                    }
                }

                _ => {
                    break;
                }
            }
        }
    }

    fn emit_token(&mut self, kind: TokenKind, lexeme: &'a str, line: usize, column: usize) {
        let token = Token {
            kind,
            lexeme,
            line,
            column,
        };

        self.tokens.push(token);
    }

    fn emit_error(&mut self, message: String, line: usize, column: usize) {
        let error = ScanError {
            context: message,
            line,
            column,
        };

        self.errors.push(error);
    }

    fn emit_current_error(&mut self, message: String) {
        self.emit_error(message, self.line, self.column - 1);
    }

    fn emit_current_simple_token(&mut self, kind: TokenKind) {
        self.emit_token(kind, "", self.line, self.column - 1);
    }

    fn consume(&mut self, c: char, message: String) {
        if self.peek() != c {
            self.emit_current_error(message);
            return;
        }

        _ = self.advance();
    }

    pub fn match_char(&mut self, c: char) -> bool {
        if self.peek() != c {
            return false;
        }

        _ = self.advance();
        true
    }

    pub fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source.chars().nth(self.current).unwrap()
    }

    pub fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.source.chars().nth(self.current + 1).unwrap()
    }

    pub fn advance(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.current += 1;
        self.column += 1;

        self.source.chars().nth(self.current - 1).unwrap()
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

fn is_identifier_start(character: char) -> bool {
    character.is_alphabetic() || character == '_'
}

fn is_identifier_rest(character: char) -> bool {
    is_identifier_start(character) || character.is_ascii_digit()
}

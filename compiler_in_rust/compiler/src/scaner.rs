#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    TokenLeftParen,
    TokenRightParen,
    TokenLeftBrace,
    TokenRightBrace,
    TokenComma,
    TokenDot,
    TokenMinus,
    TokenPlus,
    TokenSemicolon,
    TokenSlash,
    TokenStar,
    TokenBang,
    TokenBangEqual,
    TokenEqual,
    TokenEqualEqual,
    TokenGreater,
    TokenGreaterEqual,
    TokenLess,
    TokenLessEqual,
    TokenIdentifier,
    TokenString,
    TokenNumber,
    TokenAnd,
    TokenClass,
    TokenElse,
    TokenFalse,
    TokenFor,
    TokenFun,
    TokenIf,
    TokenNil,
    TokenOr,
    TokenPrint,
    TokenReturn,
    TokenSuper,
    TokenThis,
    TokenTrue,
    TokenVar,
    TokenWhile,
    TokenError,
    TokenSynthetic,
    TokenEof,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}
#[derive(Clone, Debug)]
pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    fn make_token(&self, token_type: TokenType) -> Token {
        let lexeme = &self.source[self.start..self.current].to_owned();
        Token {
            token_type,
            lexeme: lexeme.to_string(),
            line: self.line,
        }
    }
    fn error_token(&self, message: String) -> Token {
        Token {
            token_type: TokenType::TokenError,
            lexeme: message,
            line: self.line,
        }
    }
    fn advance(&mut self) -> char {
        if self.is_at_end() {
            let error_message = format!(
                "Advanced past end. current:{current} len:{len}",
                current = self.current,
                len = self.source.len()
            );
            panic!("{error_message}");
        }

        let ch = self.source[self.current..]
            .chars()
            .next()
            .expect("advance(): non-empty slice should have a char");

        self.current += ch.len_utf8();
        ch
    }
    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current..].chars().next().unwrap_or('\0')
    }
    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        let mut it = self.source[self.current..].chars();
        it.next();
        it.next().unwrap_or('\0')
    }
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            let ch = self.source[self.current..]
                .chars()
                .next()
                .expect("match_char(): non-empty slice should have a char");
            self.current += ch.len_utf8();
            true
        }
    }
    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => {
                    return;
                }
            }
        }
    }
    fn check_keyword(
        &self,
        start_offset: usize,
        expected_len: usize,
        rest: &str,
        kind: TokenType,
    ) -> TokenType {
        let start = self.start + start_offset;
        let end = start + expected_len;

        if end > self.source.len() {
            return TokenType::TokenIdentifier;
        }

        if self.current - self.start != start_offset + expected_len {
            return TokenType::TokenIdentifier;
        }

        if &self.source[start..end] == rest {
            kind
        } else {
            TokenType::TokenIdentifier
        }
    }
    fn identifier_type(&self) -> TokenType {
        let bytes = self.source.as_bytes();
        if self.start >= bytes.len() {
            return TokenType::TokenIdentifier;
        }

        match bytes[self.start] {
            b'a' => self.check_keyword(1, 2, "nd", TokenType::TokenAnd),
            b'c' => self.check_keyword(1, 4, "lass", TokenType::TokenClass),
            b'e' => self.check_keyword(1, 3, "lse", TokenType::TokenElse),
            b'f' => {
                if self.current - self.start > 1 && self.start + 1 < bytes.len() {
                    match bytes[self.start + 1] {
                        b'a' => self.check_keyword(2, 3, "lse", TokenType::TokenFalse),
                        b'o' => self.check_keyword(2, 1, "r", TokenType::TokenFor),
                        b'u' => self.check_keyword(2, 1, "n", TokenType::TokenFun),
                        _ => TokenType::TokenIdentifier,
                    }
                } else {
                    TokenType::TokenIdentifier
                }
            }
            b'i' => self.check_keyword(1, 1, "f", TokenType::TokenIf),
            b'n' => self.check_keyword(1, 2, "il", TokenType::TokenNil),
            b'o' => self.check_keyword(1, 1, "r", TokenType::TokenOr),
            b'p' => self.check_keyword(1, 4, "rint", TokenType::TokenPrint),
            b'r' => self.check_keyword(1, 5, "eturn", TokenType::TokenReturn),
            b's' => self.check_keyword(1, 4, "uper", TokenType::TokenSuper),
            b't' => {
                if self.current - self.start > 1 && self.start + 1 < bytes.len() {
                    match bytes[self.start + 1] {
                        b'h' => self.check_keyword(2, 2, "is", TokenType::TokenThis),
                        b'r' => self.check_keyword(2, 2, "ue", TokenType::TokenTrue),
                        _ => TokenType::TokenIdentifier,
                    }
                } else {
                    TokenType::TokenIdentifier
                }
            }
            b'v' => self.check_keyword(1, 2, "ar", TokenType::TokenVar),
            b'w' => self.check_keyword(1, 4, "hile", TokenType::TokenWhile),
            _ => TokenType::TokenIdentifier,
        }
    }
    fn identifier(&mut self) -> Token {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        self.make_token(self.identifier_type())
    }
    fn number(&mut self) -> Token {
        while self.peek().is_numeric() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_numeric() {
            self.advance();
            while self.peek().is_numeric() {
                self.advance();
            }
        }

        self.make_token(TokenType::TokenNumber)
    }
    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return self.error_token("Unterminated string.".to_owned());
        }
        self.advance();
        self.make_token(TokenType::TokenString)
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenType::TokenEof);
        }
        let c = self.advance();
        if c.is_alphabetic() || c == '_' {
            self.identifier()
        } else if c.is_numeric() {
            return self.number();
        } else {
            match c {
                '(' => self.make_token(TokenType::TokenLeftParen),
                ')' => self.make_token(TokenType::TokenRightParen),
                '{' => self.make_token(TokenType::TokenLeftBrace),
                '}' => self.make_token(TokenType::TokenRightBrace),
                ',' => self.make_token(TokenType::TokenComma),
                '.' => self.make_token(TokenType::TokenDot),
                '-' => self.make_token(TokenType::TokenMinus),
                '+' => self.make_token(TokenType::TokenPlus),
                ';' => self.make_token(TokenType::TokenSemicolon),
                '/' => self.make_token(TokenType::TokenSlash),
                '*' => self.make_token(TokenType::TokenStar),
                '!' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::TokenBangEqual)
                    } else {
                        self.make_token(TokenType::TokenBang)
                    }
                }
                '=' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::TokenEqualEqual)
                    } else {
                        self.make_token(TokenType::TokenEqual)
                    }
                }
                '<' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::TokenLessEqual)
                    } else {
                        self.make_token(TokenType::TokenLess)
                    }
                }
                '>' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::TokenGreaterEqual)
                    } else {
                        self.make_token(TokenType::TokenGreater)
                    }
                }
                '"' => self.string(),

                _ => self.error_token("Unexpected character.".to_owned()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(source: String) -> Vec<Token> {
        let mut scanner = Scanner::new(source);
        let mut tokens = Vec::new();

        loop {
            let token = scanner.scan_token();
            tokens.push(token.clone());
            if token.token_type == TokenType::TokenEof {
                break;
            }
        }
        tokens
    }

    fn token_types(tokens: &[Token]) -> Vec<TokenType> {
        tokens.iter().map(|t| t.token_type.clone()).collect()
    }

    fn lexemes(tokens: &[Token]) -> Vec<String> {
        tokens.iter().map(|t| t.lexeme.clone()).collect()
    }

    #[test]
    fn test_single_character_tokens() {
        let source = "{ } ( ) , . - + ; / *".to_owned();
        let tokens = scan(source);

        let expected = vec![
            TokenType::TokenLeftBrace,
            TokenType::TokenRightBrace,
            TokenType::TokenLeftParen,
            TokenType::TokenRightParen,
            TokenType::TokenComma,
            TokenType::TokenDot,
            TokenType::TokenMinus,
            TokenType::TokenPlus,
            TokenType::TokenSemicolon,
            TokenType::TokenSlash,
            TokenType::TokenStar,
            TokenType::TokenEof,
        ];

        assert_eq!(token_types(&tokens), expected);
    }

    #[test]
    fn test_one_or_two_char_tokens() {
        let source = "! != = == > >= < <=".to_owned();
        let tokens = scan(source);

        let expected = vec![
            TokenType::TokenBang,
            TokenType::TokenBangEqual,
            TokenType::TokenEqual,
            TokenType::TokenEqualEqual,
            TokenType::TokenGreater,
            TokenType::TokenGreaterEqual,
            TokenType::TokenLess,
            TokenType::TokenLessEqual,
            TokenType::TokenEof,
        ];

        assert_eq!(token_types(&tokens), expected);
    }

    #[test]
    fn test_keywords_and_identifiers() {
        let source =
            "and class else false for fun if nil or print return super this true var while foo bar"
                .to_owned();
        let tokens = scan(source);

        let expected = vec![
            TokenType::TokenAnd,
            TokenType::TokenClass,
            TokenType::TokenElse,
            TokenType::TokenFalse,
            TokenType::TokenFor,
            TokenType::TokenFun,
            TokenType::TokenIf,
            TokenType::TokenNil,
            TokenType::TokenOr,
            TokenType::TokenPrint,
            TokenType::TokenReturn,
            TokenType::TokenSuper,
            TokenType::TokenThis,
            TokenType::TokenTrue,
            TokenType::TokenVar,
            TokenType::TokenWhile,
            TokenType::TokenIdentifier, // foo
            TokenType::TokenIdentifier, // bar
            TokenType::TokenEof,
        ];

        assert_eq!(token_types(&tokens), expected);

        let lex = lexemes(&tokens);
        assert_eq!(lex[lex.len() - 3], "foo");
        assert_eq!(lex[lex.len() - 2], "bar");
    }

    #[test]
    fn test_numbers() {
        let source = "123 45.67 0.123 .".to_owned();
        let tokens = scan(source);

        let expected = vec![
            TokenType::TokenNumber,
            TokenType::TokenNumber,
            TokenType::TokenNumber,
            TokenType::TokenDot,
            TokenType::TokenEof,
        ];

        assert_eq!(token_types(&tokens), expected);
    }

    #[test]
    fn test_strings() {
        let source = r#""hello" "" "multi\nline" "unterminated"#.to_owned();
        let tokens = scan(source);

        let expected = vec![
            TokenType::TokenString,
            TokenType::TokenString,
            TokenType::TokenString,
            TokenType::TokenError, // unterminated
            TokenType::TokenEof,
        ];

        assert_eq!(token_types(&tokens), expected);

        assert_eq!(tokens[0].lexeme, "\"hello\"");
        assert_eq!(tokens[1].lexeme, "\"\"");
        assert_eq!(tokens[3].lexeme, "Unterminated string.");
    }

    #[test]
    fn test_comments_and_whitespace() {
        let source = "
            var x = 5; // this is a comment
            // another comment
            print x;  \t  \r
        "
        .to_owned();

        let tokens = scan(source);

        let expected = vec![
            TokenType::TokenVar,
            TokenType::TokenIdentifier,
            TokenType::TokenEqual,
            TokenType::TokenNumber,
            TokenType::TokenSemicolon,
            TokenType::TokenPrint,
            TokenType::TokenIdentifier,
            TokenType::TokenSemicolon,
            TokenType::TokenEof,
        ];

        assert_eq!(token_types(&tokens), expected);
    }

    #[test]
    fn test_line_numbers() {
        let source = "
            var a = 1;
            print a;
            // comment
            print \"hello\";
        "
        .to_owned();

        let mut scanner = Scanner::new(source);
        let tokens: Vec<Token> = std::iter::from_fn(|| {
            let token = scanner.scan_token();
            if token.token_type == TokenType::TokenEof {
                None
            } else {
                Some(token)
            }
        })
        .collect();
        let lines: Vec<usize> = tokens.iter().map(|t| t.line).collect();

        // Expected line numbers (1-indexed)
        let expected_lines = vec![2, 2, 2, 2, 2, 3, 3, 3, 5, 5, 5];

        assert_eq!(lines, expected_lines);
    }

    #[test]
    fn test_unexpected_character() {
        let source = "@ # $".to_owned();
        let tokens = scan(source);

        let types: Vec<TokenType> = token_types(&tokens);
        assert!(types.contains(&TokenType::TokenError));
        assert_eq!(types[types.len() - 2], TokenType::TokenError); // last non-EOF
    }

    #[test]
    fn test_full_lox_example() {
        let source = r#"
            var breakfast = "bagels";
            print breakfast;
            if (breakfast == "bagels") print "yes!";
        "#
        .to_owned();

        let tokens = scan(source);
        let types = token_types(&tokens);

        assert!(types.contains(&TokenType::TokenVar));
        assert!(types.contains(&TokenType::TokenString));
        assert!(types.contains(&TokenType::TokenPrint));
        assert!(types.contains(&TokenType::TokenIf));
        assert!(types.contains(&TokenType::TokenEqualEqual));
        assert_eq!(*types.last().unwrap(), TokenType::TokenEof);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    TOKEN_LEFT_PAREN,
    TOKEN_RIGHT_PAREN,
    TOKEN_LEFT_BRACE,
    TOKEN_RIGHT_BRACE,
    TOKEN_COMMA,
    TOKEN_DOT,
    TOKEN_MINUS,
    TOKEN_PLUS,
    TOKEN_SEMICOLON,
    TOKEN_SLASH,
    TOKEN_STAR,
    TOKEN_BANG,
    TOKEN_BANG_EQUAL,
    TOKEN_EQUAL,
    TOKEN_EQUAL_EQUAL,
    TOKEN_GREATER,
    TOKEN_GREATER_EQUAL,
    TOKEN_LESS,
    TOKEN_LESS_EQUAL,
    TOKEN_IDENTIFIER,
    TOKEN_STRING,
    TOKEN_NUMBER,
    TOKEN_AND,
    TOKEN_CLASS,
    TOKEN_ELSE,
    TOKEN_FALSE,
    TOKEN_FOR,
    TOKEN_FUN,
    TOKEN_IF,
    TOKEN_NIL,
    TOKEN_OR,
    TOKEN_PRINT,
    TOKEN_RETURN,
    TOKEN_SUPER,
    TOKEN_THIS,
    TOKEN_TRUE,
    TOKEN_VAR,
    TOKEN_WHILE,
    TOKEN_ERROR,
    TOKEN_EOF,
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
            token_type: TokenType::TOKEN_ERROR,
            lexeme: message,
            line: self.line,
        }
    }
    fn advance(&mut self) -> char {
        let error_message = format!(
            "Advanced past end. current:{current} len:{len}",
            current = self.current,
            len = self.source.len()
        );
        let ch = self.source.chars().nth(self.current).expect(&error_message);
        self.current += 1;
        ch
    }
    fn peek(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        let error_message = format!(
            "Peek past end. current:{current} len:{len}",
            current = self.current,
            len = self.source.len()
        );
        let ch = self.source.chars().nth(self.current).expect(&error_message);
        ch
    }
    fn peek_next(&mut self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        let error_message = format!(
            "Peek past end. current + 1:{current} len:{len}",
            current = self.current + 1,
            len = self.source.len()
        );
        let ch = self
            .source
            .chars()
            .nth(self.current + 1)
            .expect(&error_message);
        ch
    }
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.current += 1;
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
        let lexeme =
            &self.source[self.start + start_offset..self.start + start_offset + expected_len];

        if self.current - self.start == start_offset + expected_len && lexeme == rest {
            kind
        } else {
            TokenType::TOKEN_IDENTIFIER
        }
    }
    fn identifier_type(&self) -> TokenType {
        match self.source.chars().nth(self.start) {
            Some('a') => self.check_keyword(1, 2, "nd", TokenType::TOKEN_AND),
            Some('c') => self.check_keyword(1, 4, "lass", TokenType::TOKEN_CLASS),
            Some('e') => self.check_keyword(1, 3, "lse", TokenType::TOKEN_ELSE),
            Some('f') => {
                if self.current - self.start > 1 {
                    match self.source.chars().nth(self.start + 1) {
                        Some('a') => self.check_keyword(2, 3, "lse", TokenType::TOKEN_FALSE),
                        Some('o') => self.check_keyword(2, 1, "r", TokenType::TOKEN_FOR),
                        Some('u') => self.check_keyword(2, 1, "n", TokenType::TOKEN_FUN),
                        _ => TokenType::TOKEN_IDENTIFIER,
                    }
                } else {
                    TokenType::TOKEN_IDENTIFIER
                }
            }
            Some('i') => self.check_keyword(1, 1, "f", TokenType::TOKEN_IF),
            Some('n') => self.check_keyword(1, 2, "il", TokenType::TOKEN_NIL),
            Some('o') => self.check_keyword(1, 1, "r", TokenType::TOKEN_OR),
            Some('p') => self.check_keyword(1, 4, "rint", TokenType::TOKEN_PRINT),
            Some('r') => self.check_keyword(1, 5, "eturn", TokenType::TOKEN_RETURN),
            Some('s') => self.check_keyword(1, 4, "uper", TokenType::TOKEN_SUPER),
            Some('t') => {
                if self.current - self.start > 1 {
                    match self.source.chars().nth(self.start + 1) {
                        Some('h') => self.check_keyword(2, 2, "is", TokenType::TOKEN_THIS),
                        Some('r') => self.check_keyword(2, 2, "ue", TokenType::TOKEN_TRUE),
                        _ => TokenType::TOKEN_IDENTIFIER,
                    }
                } else {
                    TokenType::TOKEN_IDENTIFIER
                }
            }
            Some('v') => self.check_keyword(1, 2, "ar", TokenType::TOKEN_VAR),
            Some('w') => self.check_keyword(1, 4, "hile", TokenType::TOKEN_WHILE),
            _ => TokenType::TOKEN_IDENTIFIER,
        }
    }
    fn identifier(&mut self) -> Token {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        return self.make_token(self.identifier_type());
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

        self.make_token(TokenType::TOKEN_NUMBER)
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
        self.make_token(TokenType::TOKEN_STRING)
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenType::TOKEN_EOF);
        }
        let c = self.advance();
        if c.is_alphabetic() || c == '_' {
            return self.identifier();
        } else if c.is_numeric() {
            return self.number();
        } else {
            match c {
                '(' => self.make_token(TokenType::TOKEN_LEFT_PAREN),
                ')' => self.make_token(TokenType::TOKEN_RIGHT_PAREN),
                '{' => self.make_token(TokenType::TOKEN_LEFT_BRACE),
                '}' => self.make_token(TokenType::TOKEN_RIGHT_BRACE),
                ',' => self.make_token(TokenType::TOKEN_COMMA),
                '.' => self.make_token(TokenType::TOKEN_DOT),
                '-' => self.make_token(TokenType::TOKEN_MINUS),
                '+' => self.make_token(TokenType::TOKEN_PLUS),
                ';' => self.make_token(TokenType::TOKEN_SEMICOLON),
                '/' => self.make_token(TokenType::TOKEN_SLASH),
                '*' => self.make_token(TokenType::TOKEN_STAR),
                '!' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::TOKEN_BANG_EQUAL)
                    } else {
                        self.make_token(TokenType::TOKEN_BANG)
                    }
                }
                '=' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::TOKEN_EQUAL_EQUAL)
                    } else {
                        self.make_token(TokenType::TOKEN_EQUAL)
                    }
                }
                '<' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::TOKEN_LESS_EQUAL)
                    } else {
                        self.make_token(TokenType::TOKEN_LESS)
                    }
                }
                '>' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::TOKEN_GREATER_EQUAL)
                    } else {
                        self.make_token(TokenType::TOKEN_GREATER)
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
            if token.token_type == TokenType::TOKEN_EOF {
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
            TokenType::TOKEN_LEFT_BRACE,
            TokenType::TOKEN_RIGHT_BRACE,
            TokenType::TOKEN_LEFT_PAREN,
            TokenType::TOKEN_RIGHT_PAREN,
            TokenType::TOKEN_COMMA,
            TokenType::TOKEN_DOT,
            TokenType::TOKEN_MINUS,
            TokenType::TOKEN_PLUS,
            TokenType::TOKEN_SEMICOLON,
            TokenType::TOKEN_SLASH,
            TokenType::TOKEN_STAR,
            TokenType::TOKEN_EOF,
        ];

        assert_eq!(token_types(&tokens), expected);
    }

    #[test]
    fn test_one_or_two_char_tokens() {
        let source = "! != = == > >= < <=".to_owned();
        let tokens = scan(source);

        let expected = vec![
            TokenType::TOKEN_BANG,
            TokenType::TOKEN_BANG_EQUAL,
            TokenType::TOKEN_EQUAL,
            TokenType::TOKEN_EQUAL_EQUAL,
            TokenType::TOKEN_GREATER,
            TokenType::TOKEN_GREATER_EQUAL,
            TokenType::TOKEN_LESS,
            TokenType::TOKEN_LESS_EQUAL,
            TokenType::TOKEN_EOF,
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
            TokenType::TOKEN_AND,
            TokenType::TOKEN_CLASS,
            TokenType::TOKEN_ELSE,
            TokenType::TOKEN_FALSE,
            TokenType::TOKEN_FOR,
            TokenType::TOKEN_FUN,
            TokenType::TOKEN_IF,
            TokenType::TOKEN_NIL,
            TokenType::TOKEN_OR,
            TokenType::TOKEN_PRINT,
            TokenType::TOKEN_RETURN,
            TokenType::TOKEN_SUPER,
            TokenType::TOKEN_THIS,
            TokenType::TOKEN_TRUE,
            TokenType::TOKEN_VAR,
            TokenType::TOKEN_WHILE,
            TokenType::TOKEN_IDENTIFIER, // foo
            TokenType::TOKEN_IDENTIFIER, // bar
            TokenType::TOKEN_EOF,
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
            TokenType::TOKEN_NUMBER,
            TokenType::TOKEN_NUMBER,
            TokenType::TOKEN_NUMBER,
            TokenType::TOKEN_DOT,
            TokenType::TOKEN_EOF,
        ];

        assert_eq!(token_types(&tokens), expected);
    }

    #[test]
    fn test_strings() {
        let source = r#""hello" "" "multi\nline" "unterminated"#.to_owned();
        let tokens = scan(source);

        let expected = vec![
            TokenType::TOKEN_STRING,
            TokenType::TOKEN_STRING,
            TokenType::TOKEN_STRING,
            TokenType::TOKEN_ERROR, // unterminated
            TokenType::TOKEN_EOF,
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
            TokenType::TOKEN_VAR,
            TokenType::TOKEN_IDENTIFIER,
            TokenType::TOKEN_EQUAL,
            TokenType::TOKEN_NUMBER,
            TokenType::TOKEN_SEMICOLON,
            TokenType::TOKEN_PRINT,
            TokenType::TOKEN_IDENTIFIER,
            TokenType::TOKEN_SEMICOLON,
            TokenType::TOKEN_EOF,
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
            if token.token_type == TokenType::TOKEN_EOF {
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
        assert!(types.contains(&TokenType::TOKEN_ERROR));
        assert_eq!(types[types.len() - 2], TokenType::TOKEN_ERROR); // last non-EOF
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

        assert!(types.contains(&TokenType::TOKEN_VAR));
        assert!(types.contains(&TokenType::TOKEN_STRING));
        assert!(types.contains(&TokenType::TOKEN_PRINT));
        assert!(types.contains(&TokenType::TOKEN_IF));
        assert!(types.contains(&TokenType::TOKEN_EQUAL_EQUAL));
        assert_eq!(*types.last().unwrap(), TokenType::TOKEN_EOF);
    }
}

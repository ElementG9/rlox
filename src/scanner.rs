use std::collections::HashMap;
use crate::Error;

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    Identifier, String, Number,

    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    // End of file.
    Eof
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Token {
    r#type: TokenType,
    lexeme: String,
    literal: String,
    line: usize,
}
impl Token {
    pub fn new(r#type: TokenType, lexeme: String, literal: String, line: usize) -> Token {
        Token {
            r#type,
            lexeme,
            literal,
            line,
        }
    }
    pub fn to_string(&self) -> String {
        format!("{:?} {}", self.r#type, self.literal)
    }
}

#[allow(dead_code)]
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<String, TokenType>
}
impl Scanner {
    pub fn new(source: String) -> Scanner {
        let mut keywords = HashMap::new();
        keywords.insert("and".to_owned(), TokenType::And);
        keywords.insert("or".to_owned(), TokenType::Or);
        keywords.insert("if".to_owned(), TokenType::If);
        keywords.insert("else".to_owned(), TokenType::Else);
        keywords.insert("for".to_owned(), TokenType::For);
        keywords.insert("while".to_owned(), TokenType::While);
        keywords.insert("var".to_owned(), TokenType::Var);
        keywords.insert("class".to_owned(), TokenType::Class);
        keywords.insert("fun".to_owned(), TokenType::Fun);
        keywords.insert("return".to_owned(), TokenType::Return);
        keywords.insert("print".to_owned(), TokenType::Print);
        keywords.insert("super".to_owned(), TokenType::Super);
        keywords.insert("this".to_owned(), TokenType::This);
        keywords.insert("true".to_owned(), TokenType::True);
        keywords.insert("false".to_owned(), TokenType::False);
        keywords.insert("nil".to_owned(), TokenType::Nil);
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords,
        }
    }
    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, Error> {
        while !self.is_at_end() {
          // We are at the beginning of the next lexeme.
          self.start = self.current;
          self.scan_token()?;
        }
        self.tokens.push(Token::new(TokenType::Eof, "".to_owned(), "".to_owned(), self.line));
        Ok(&self.tokens)
    }
    fn scan_token(&mut self) -> Result<(), Error>{
        let c = self.advance();
        match c {
          '(' => {
              self.add_token(TokenType::LeftParen);
          },
          ')' => {
              self.add_token(TokenType::RightParen);
          },
          '{' => {
              self.add_token(TokenType::LeftBrace);
          },
          '}' => {
              self.add_token(TokenType::RightBrace);
          },
          ',' => {
              self.add_token(TokenType::Comma);
          },
          '.' => {
              self.add_token(TokenType::Dot);
          },
          '-' => {
              self.add_token(TokenType::Minus);
          },
          '+' => {
              self.add_token(TokenType::Plus);
          },
          ';' => {
              self.add_token(TokenType::Semicolon);
          },
          '*' => {
              self.add_token(TokenType::Star);
          },
          '!' => {
              let r#type = if self.advance_if_equal('=') {
                  TokenType::BangEqual
              } else {
                  TokenType::Bang
              };
              self.add_token(r#type);
          },
          '=' => {
              let r#type = if self.advance_if_equal('=') {
                  TokenType::EqualEqual
              } else {
                  TokenType::Equal
              };
              self.add_token(r#type);
          },
          '<' => {
              let r#type = if self.advance_if_equal('=') {
                  TokenType::LessEqual
              } else {
                  TokenType::Less
              };
              self.add_token(r#type);
          },
          '>' => {
              let r#type = if self.advance_if_equal('=') {
                  TokenType::GreaterEqual
              } else {
                  TokenType::Greater
              };
              self.add_token(r#type);
          },
          '/' => { // More complicated because comments use //
              if self.advance_if_equal('/') {
                  while self.peek() != '\n' && !self.is_at_end() {
                      self.advance();
                  }
              } else if self.advance_if_equal('*') { // Block comment
                  let mut nesting_layer = 1;
                  loop {
                      if self.is_at_end() {
                          let mut s = String::from("Unbalanced block comment");
                          if nesting_layer > 0 {
                              s.push_str(": missing closing */");
                          }
                          return Err(Error::new(s, self.line));
                      } else if self.peek() == '/' && self.peek_next() == '*' {
                          self.advance(); // Consume nested /
                          self.advance(); // Consume nested *
                          nesting_layer += 1;
                      } else if self.peek() == '*' && self.peek_next() == '/' {
                          if nesting_layer == 1 {
                              self.advance(); // Consume closing *
                              self.advance(); // Consume closing /
                              break;
                          } else {
                              nesting_layer -= 1;
                          }
                      } else {
                          self.advance();
                      }
                  }
              } else {
                  self.add_token(TokenType::Slash);
              }
          },
          ' ' | '\r' | '\t' => {}, // Ignore whitespace.
          '"' => {
              self.read_string()?;
          },
          '\n' => {
              self.line += 1;
          },
          _ => {
              if is_digit(c) {
                  self.read_number()?;
              } else if is_alpha(c) {
                  self.read_identifier()?;
              } else {
                  return Err(Error::new("Unexpected character.".to_owned(), String::new(), self.line));
              }
          }
      };
      Ok(())
    }
    fn read_number(&mut self) -> Result<(), Error> {
        while is_digit(self.peek()) {
            self.advance();
        }
        // Look for a fractional part.
        if self.peek() == '.' && is_digit(self.peek_next()) {
          // Consume the "."
          self.advance();

          while is_digit(self.peek()) {
              self.advance();
          }
        }
        self.add_token_literal(TokenType::Number, substring(&self.source, self.start, self.current));
        Ok(())
    }
    fn read_identifier(&mut self) -> Result<(), Error>{
        while is_alphanumeric(self.peek()) {
            self.advance();
        }
        let text = substring(&self.source, self.start, self.current);
        let t = match self.keywords.get(&text) {
            Some(t) => *t,
            None => TokenType::Identifier
        };
        if t == TokenType::Identifier {
            self.add_token_literal(t, text);
        } else {
            self.add_token(t);
        }
        Ok(())
    }
    fn read_string(&mut self) -> Result<(), Error> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        // Unterminated string.
        if self.is_at_end() {
            Err(Error::new("Unterminated string".to_owned(), String::new(), self.line))
        } else {
            // The closing ".
            self.advance();
            self.add_token_literal(TokenType::String, substring(&self.source, self.start + 1, self.current - 1));
            Ok(())
        }
    }
    fn add_token(&mut self, r#type: TokenType) {
        self.add_token_literal(r#type, "".to_owned());
    }
    fn add_token_literal(&mut self, r#type: TokenType, literal: String) {
        self.tokens.push(Token::new(r#type, substring(&self.source, self.start, self.current), literal, self.line));
    }
    fn is_at_end(&self) -> bool {
        self.current >= str_len(&self.source)
    }
    fn advance(&mut self) -> char {
        self.current += 1;
        char_at(&self.source, self.current - 1)
    }
    fn advance_if_equal(&mut self, c: char) -> bool {
        let p = self.peek();
        if p == '\0' || p != c {
            false
        } else {
            self.advance();
            true
        }
    }
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            char_at(&self.source, self.current)
        }
    }
    fn peek_next(&self) -> char {
        if self.current + 1 >= str_len(&self.source) {
            '\0'
        } else {
            char_at(&self.source, self.current + 1)
        }
    }
}
// Helper functions
fn str_len(s: &String) -> usize {
    s.chars().count()
}
fn char_at(s: &String, pos: usize) -> char {
    if pos >= str_len(&s) {
        '\0'
    } else {
        s.chars().skip(pos).next().unwrap()
    }
}
fn substring(s: &String, start: usize, mut end: usize) -> String {
    if start > end || start >= str_len(&s) {
        "".to_owned()
    } else {
        if end > str_len(&s) {
            end = str_len(&s);
        }
        let text: String = s.chars().skip(start).take(end - start).collect();
        text
    }
}
fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}
fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') ||
    (c >= 'A' && c <= 'Z') ||
    c == '_'
}
fn is_alphanumeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Helper function tests.
    #[test]
    fn test_str_len() {
        let s1 = String::from("Hello world!");
        assert_eq!(str_len(&s1), 12);
        let s2 = String::from("this is a test");
        assert_eq!(str_len(&s2), 14);
    }
    #[test]
    fn test_char_at() {
        let s = String::from("this is a test");
        assert_eq!(char_at(&s, 1), 'h'); // valid
        assert_eq!(char_at(&s, 5), 'i'); // valid
        assert_eq!(char_at(&s, 8), 'a'); // valid
        assert_eq!(char_at(&s, 20), '\0'); // pos > s.len()
    }
    #[test]
    fn test_substring() {
        let s = String::from("this is a test");
        assert_eq!(substring(&s, 0, 4), "this".to_owned()); // valid
        assert_eq!(substring(&s, 6, 12), "s a te".to_owned()); // valid
        assert_eq!(substring(&s, 8, 20), "a test".to_owned()); // end > s.len()
        assert_eq!(substring(&s, 25, 27), "".to_owned()); // start > s.len()
        assert_eq!(substring(&s, 5, 3), "".to_owned()); // start > end
    }
    #[test]
    fn test_is_digit() {
        assert!(is_digit('0'));
        assert!(is_digit('4'));
        assert!(is_digit('9'));
        assert!(!is_digit('a'));
        assert!(!is_digit('$'));
    }
    #[test]
    fn test_is_alpha() {
        assert!(is_alpha('s'));
        assert!(is_alpha('f'));
        assert!(is_alpha('a'));
        assert!(is_alpha('_'));
        assert!(!is_alpha('^'));
        assert!(!is_alpha('9'));
    }
    #[test]
    fn test_is_alphanumeric() {
        assert!(is_alphanumeric('4'));
        assert!(is_alphanumeric('b'));
        assert!(is_alphanumeric('x'));
        assert!(is_alphanumeric('_'));
        assert!(!is_alphanumeric('%'));
        assert!(!is_alphanumeric('+'));
    }

    // Token tests
    #[test]
    fn test_token_constructor() {
        let t = Token::new(
            TokenType::Equal,
            "asdf".to_owned(),
            "==".to_owned(),
            23
        );
        assert_eq!(t.r#type, TokenType::Equal);
        assert_eq!(t.lexeme, "asdf".to_owned());
        assert_eq!(t.literal, "==".to_owned());
        assert_eq!(t.line, 23);
    }
    #[test]
    fn test_token_to_string() {
        let t = Token::new(
            TokenType::Equal,
            "asdf".to_owned(),
            "==".to_owned(),
            23
        );
        assert_eq!(t.to_string(), String::from("Equal =="));
    }

    // Scanner tests
    #[test]
    fn test_scanner_constructor() {
        let s = Scanner::new("var x = 5;".to_owned());
        assert_eq!(s.source, "var x = 5;".to_owned());
        assert_eq!(s.tokens.len(), 0);
        assert_eq!(s.start, 0);
        assert_eq!(s.current, 0);
        assert_eq!(s.line, 1);
        assert_eq!(*s.keywords.get("nil").unwrap(), TokenType::Nil);
    }
    fn test_scanner_scanning() {
        let mut s = Scanner::new("var x = 5;".to_owned());
        let mut t = s.scan_tokens();
        match t {
            Ok(tokens) => {
                assert_eq!(tokens[0].to_string(), "Var ".to_owned());
                assert_eq!(tokens[1].to_string(), "Identifier x".to_owned());
                assert_eq!(tokens[2].to_string(), "Equal ".to_owned());
                assert_eq!(tokens[3].to_string(), "Number 5".to_owned());
                assert_eq!(tokens[4].to_string(), "Semicolon ".to_owned());
            },
            Err(err) => {
                assert!(false);
            }
        }
    }
}

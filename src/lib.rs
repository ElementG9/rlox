use std::fs::File;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;

pub fn run_prompt() {
    println!("Running rlox prompt");
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_bytes_read) => {
                input.pop(); // Remove trailing newline
                std::io::stdout().flush().unwrap();
                run(&input);
            }
            Err(err) => println!("error: {}", err),
        }
    }
}
pub fn run_file(filename: &str) {
    println!("Running rlox file {}", filename);
    if !Path::new(&filename).exists() {
        panic!(format!("Error running rlox file: {} does not exist", filename));
    }
    let mut file = File::open(&filename)
        .expect(&format!("Error running rlox file: Could not open {}", filename));
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Error running rlox file: Could not copy to string");
    run(&contents);
}

pub fn run(source: &str) {
    let mut had_error = false;
    let mut scanner = Scanner::new(String::from(source));
    let tokens = match scanner.scan_tokens() {
        Ok(val) => val,
        Err(err) => {
            std::process::exit(1);
        }
    };
    for t in tokens {
        println!("{}", t.to_string());
    }
    if had_error {
        std::process::exit(1);
    }
}
fn error(line: usize, message: String) {
    report(line, String::new(), message);
}
fn report(line: usize, r#where: String, message: String) {
    eprintln!("[line {}] Error {}: {}", line, r#where, message);
}

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
enum TokenType {
    // Single-character tokens.
    LEFT_PAREN, RIGHT_PAREN, LEFT_BRACE, RIGHT_BRACE,
    COMMA, DOT, MINUS, PLUS, SEMICOLON, SLASH, STAR,

    // One or two character tokens.
    BANG, BANG_EQUAL,
    EQUAL, EQUAL_EQUAL,
    GREATER, GREATER_EQUAL,
    LESS, LESS_EQUAL,

    // Literals.
    IDENTIFIER, STRING, NUMBER,

    // Keywords.
    AND, CLASS, ELSE, FALSE, FUN, FOR, IF, NIL, OR,
    PRINT, RETURN, SUPER, THIS, TRUE, VAR, WHILE,

    EOF
}
struct Token {
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
        format!("{:?} {} {}", self.r#type, self.lexeme, self.literal)
    }
}
struct Scanner {
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
        keywords.insert("and".to_owned(), TokenType::AND);
        keywords.insert("class".to_owned(), TokenType::CLASS);
        keywords.insert("else".to_owned(), TokenType::ELSE);
        keywords.insert("false".to_owned(), TokenType::FALSE);
        keywords.insert("for".to_owned(), TokenType::FOR);
        keywords.insert("fun".to_owned(), TokenType::FUN);
        keywords.insert("if".to_owned(), TokenType::IF);
        keywords.insert("nil".to_owned(), TokenType::NIL);
        keywords.insert("or".to_owned(), TokenType::OR);
        keywords.insert("print".to_owned(), TokenType::PRINT);
        keywords.insert("return".to_owned(), TokenType::RETURN);
        keywords.insert("super".to_owned(), TokenType::SUPER);
        keywords.insert("this".to_owned(), TokenType::THIS);
        keywords.insert("true".to_owned(), TokenType::TRUE);
        keywords.insert("var".to_owned(), TokenType::VAR);
        keywords.insert("while".to_owned(), TokenType::WHILE);
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
            keywords,
        }
    }
    fn scan_tokens(&mut self) -> Result<&Vec<Token>, String> {
        while !self.is_at_end() {
          // We are at the beginning of the next lexeme.
          self.start = self.current;
          self.scan_token();
        }
        self.tokens.push(Token::new(TokenType::EOF, "".to_owned(), "".to_owned(), self.line));
        Ok(&self.tokens)
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.source.chars().count()
    }
    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
          '(' => {
              self.add_token(TokenType::LEFT_PAREN);
          },
          ')' => {
              self.add_token(TokenType::RIGHT_PAREN);
          },
          '{' => {
              self.add_token(TokenType::LEFT_BRACE);
          },
          '}' => {
              self.add_token(TokenType::RIGHT_BRACE);
          },
          ',' => {
              self.add_token(TokenType::COMMA);
          },
          '.' => {
              self.add_token(TokenType::DOT);
          },
          '-' => {
              self.add_token(TokenType::MINUS);
          },
          '+' => {
              self.add_token(TokenType::PLUS);
          },
          ';' => {
              self.add_token(TokenType::SEMICOLON);
          },
          '*' => {
              self.add_token(TokenType::STAR);
          },
          '!' => {
              let r#type = if self.advance_if_equal('=') {
                  TokenType::BANG_EQUAL
              } else {
                  TokenType::BANG
              };
              self.add_token(r#type);
          },
          '=' => {
              let r#type = if self.advance_if_equal('=') {
                  TokenType::EQUAL_EQUAL
              } else {
                  TokenType::EQUAL
              };
              self.add_token(r#type);
          },
          '<' => {
              let r#type = if self.advance_if_equal('=') {
                  TokenType::LESS_EQUAL
              } else {
                  TokenType::LESS
              };
              self.add_token(r#type);
          },
          '>' => {
              let r#type = if self.advance_if_equal('=') {
                  TokenType::GREATER_EQUAL
              } else {
                  TokenType::GREATER
              };
              self.add_token(r#type);
          },
          '/' => { // More complicated because comments use //
              if self.advance_if_equal('/') {
                  while self.peek() != '\n' && !self.is_at_end() {
                      self.advance();
                  }
              } else {
                  self.add_token(TokenType::SLASH);
              }
          },
          ' ' | '\r' | '\t' => {}, // Ignore whitespace.
          '"' => {
              self.read_string();
          },
          '\n' => {
              self.line += 1;
          },
          _ => {
              if self.is_digit(c) {
                  self.read_number();
              } else if self.is_alpha(c) {
                  self.read_identifier();
              } else {
                  error(self.line, "Unexpected character.".to_owned());
              }
          }
      };
    }
    fn advance(&mut self) -> char {
        self.current += 1;
        char_at(&self.source, self.current - 1)
    }
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            char_at(&self.source, self.current)
        }
    }
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.chars().count() {
            '\0'
        } else {
            char_at(&self.source, self.current + 1)
        }
    }
    fn advance_if_equal(&mut self, c: char) -> bool {
        let p = self.peek();
        if p == '\0' || p == c {
            false
        } else {
            self.current += 1;
            true
        }
    }
    fn add_token(&mut self, r#type: TokenType) {
        self.add_token_literal(r#type, "".to_owned());
    }
    fn add_token_literal(&mut self, r#type: TokenType, literal: String) {
        self.tokens.push(Token::new(r#type, substring(&self.source, self.start, self.current), literal, self.line));
    }
    fn read_string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        // Unterminated string.
        if self.is_at_end() {
            error(self.line, "Unterminated string.".to_owned());
        } else {
            // The closing ".
            self.advance();
            self.add_token_literal(TokenType::STRING, substring(&self.source, self.start + 1, self.current - 1));
        }
    }
    fn read_number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }
        // Look for a fractional part.
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
          // Consume the "."
          self.advance();

          while self.is_digit(self.peek()) {
              self.advance();
          }
        }
        self.add_token_literal(TokenType::NUMBER, substring(&self.source, self.start, self.current));
    }
    fn read_identifier(&mut self) {
        while self.is_alphanumeric(self.peek()) {
            self.advance();
        }
        let text = substring(&self.source, self.start, self.current);
        match self.keywords.get(&text) {
            Some(t) => {
                self.add_token(*t);
            },
            None => {
                self.add_token(TokenType::IDENTIFIER);
            }
        }
    }
    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }
    fn is_alpha(&self, c: char) -> bool {
        (c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') ||
        c == '_'
    }
    fn is_alphanumeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }
}
fn char_at(s: &String, pos: usize) -> char {
    if pos >= s.chars().count() {
        '\0'
    } else {
        let char_vec: Vec<char> = s.chars().collect();
        char_vec[pos]
    }
}
fn substring(s: &String, start: usize, end: usize) -> String {
    let len = s.chars().count();
    if start > end || start >= len || end >= len {
        "".to_owned()
    } else {
        let text: String = s.chars().skip(start).take(end).collect();
        text
    }
}

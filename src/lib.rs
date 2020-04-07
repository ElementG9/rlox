pub mod scanner;
use scanner::*;

pub fn run(source: &str) -> Result<(), Error> {
    let mut scanner = Scanner::new(String::from(source));
    let tokens = scanner.scan_tokens()?;
    for t in tokens {
        println!("{}", t.to_string());
    }
    Ok(())
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub line: usize,
}
impl Error {
    pub fn new(message: String, line: usize) -> Error {
        Error {
            message,
            line
        }
    }
    pub fn to_string(&self) -> String {
        format!("Error: [line {}] {}", self.line, self.message)
    }
    pub fn report(&self) {
        eprintln!("{}", self.to_string());
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub use crate::common::*;
pub use crate::token::*;

pub trait Tokenizer {
    fn next_token(self: &mut Self) -> Result<Token, Error>;
    fn peek_token(self: &Self) -> Result<Token, Error>;
    fn peek_kind(self: &Self) -> Result<TokenKind, Error>;

    fn expect_token(self: &mut Self, kind: TokenKind) -> Result<Token, Error> {
        let actual_token = self.peek_token()?;
        if actual_token.kind != kind {
            Err(Error {
                location: actual_token.location.clone(),
                message: format!(
                    "Unexpected token '{:?}', expected '{:?}'",
                    actual_token.kind, kind,
                ),
            })
        } else {
            self.next_token()
        }
    }
}

struct TokenArray {
    pub filepath: String,
    pub tokens: Vec<Token>,
    pub position: usize,
}

impl TokenArray {
    pub fn get_end_of_file_token(self: &TokenArray) -> Token {
        Token {
            kind: TokenKind::EndOfFile,
            location: if self.tokens.len() > 0 {
                let last_token = self.tokens.last().unwrap();
                SourceLocation {
                    filepath: self.filepath.clone(),
                    position: last_token.location.position + last_token.length,
                    line: last_token.location.line,
                    column: last_token.location.column + last_token.length,
                }
            } else {
                SourceLocation {
                    filepath: self.filepath.clone(),
                    position: 0,
                    line: 1,
                    column: 1,
                }
            },
            length: 0,
            data: TokenData::None,
        }
    }
}

impl Tokenizer for TokenArray {
    fn next_token(self: &mut Self) -> Result<Token, Error> {
        self.position += 1;
        if self.position < self.tokens.len() {
            Ok(self.tokens[self.position - 1].clone())
        } else {
            Ok(self.get_end_of_file_token())
        }
    }

    fn peek_token(self: &Self) -> Result<Token, Error> {
        if self.position < self.tokens.len() {
            Ok(self.tokens[self.position].clone())
        } else {
            Ok(self.get_end_of_file_token())
        }
    }

    fn peek_kind(self: &Self) -> Result<TokenKind, Error> {
        if self.position < self.tokens.len() {
            Ok(self.tokens[self.position].kind.clone())
        } else {
            Ok(TokenKind::EndOfFile)
        }
    }
}

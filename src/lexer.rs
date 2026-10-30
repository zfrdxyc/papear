use crate::{
    common::Span,
    error::{
        ProgramError,
        ProgramErrorKind::{self, UnexpectedCharacter},
    },
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind<'a> {
    Word(&'a str),
    NumberPID(usize),
    Comma,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}

impl<'a> Token<'a> {
    fn new(kind: TokenKind<'a>, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    source: &'a str,
    current: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, current: 0 }
    }
    pub fn get_tokens(&mut self) -> Result<Vec<Token<'a>>, ProgramError> {
        let mut tokens: Vec<Token<'a>> = vec![];
        while let Some(ch) = self.peek() {
            match ch {
                ',' => tokens.push(Token::new(
                    TokenKind::Comma,
                    Span::new(self.current, self.current + 1),
                )),
                '"' => {
                    tokens.push(self.parse_quoted_identifier()?);
                    continue;
                }
                ch if ch.is_ascii_alphabetic() => {
                    tokens.push(self.parse_identifier()?);
                    continue;
                }
                ch if ch.is_whitespace() => {}
                _ => {
                    return Err(ProgramError::new(
                        UnexpectedCharacter(ch),
                        Span::new(self.current, self.current + ch.len_utf8()),
                    ));
                }
            }
            self.next();
        }
        Ok(tokens)
    }

    fn parse_identifier(&mut self) -> Result<Token<'a>, ProgramError> {
        let start = self.current;
        while let Some(ch) = self.peek()
            && !ch.is_whitespace()
            && ch != ','
        {
            self.next();
        }
        let span = Span::new(start, self.current);
        let word = &self.source[start..self.current];
        if word == "pid" {
            self.parse_after_pid_kw(span)
        } else {
            Ok(Token::new(TokenKind::Word(word), span))
        }
    }
    fn parse_after_pid_kw(&mut self, kw_span: Span) -> Result<Token<'a>, ProgramError> {
        while let Some(ch) = self.peek() {
            match ch {
                ch if ch.is_whitespace() => {}
                ch if ch.is_ascii_digit() => return self.parse_num(),
                _ => {
                    return Err(ProgramError::new(
                        UnexpectedCharacter(ch),
                        Span::new(self.current, self.current),
                    ));
                }
            }
            self.next();
        }
        Err(ProgramError::new(ProgramErrorKind::ExpectedPid, kw_span))
    }
    // valid numbers (?
    fn parse_num(&mut self) -> Result<Token<'a>, ProgramError> {
        // skip ws
        while let Some(ch) = self.peek()
            && ch.is_whitespace()
        {
            self.next();
        }
        let start = self.current;
        while let Some(ch) = self.peek()
            && !ch.is_whitespace()
            && ch != ','
        {
            self.next();
        }
        let num_span = Span::new(start, self.current);
        let word_num = &self.source[start..self.current];
        let num = word_num.parse::<usize>().map_err(|_| {
            ProgramError::new(ProgramErrorKind::InvalidPid(word_num.to_owned()), num_span)
        })?;
        Ok(Token::new(TokenKind::NumberPID(num), num_span))
    }
    // "quoted name"
    fn parse_quoted_identifier(&mut self) -> Result<Token<'a>, ProgramError> {
        // `"` caller doesnt consume it.
        self.next();
        let start = self.current;
        while let Some(ch) = self.peek()
            && ch != '"'
        {
            self.next();
        }
        let span_w_quote = Span::new(start - 1, self.current + 1);
        let identifier = &self.source[start..self.current]; // doesnt count the quote
        if !matches!(self.next(), Some(ch) if ch == '"') {
            return Err(ProgramError::new(
                ProgramErrorKind::UnclosedQuote,
                Span::new(start - 1, self.current), // not sure if this is ok
            ));
        }
        Ok(Token::new(TokenKind::Word(identifier), span_w_quote))
    }
    fn peek(&self) -> Option<char> {
        self.source[self.current..].chars().next()
    }
    fn next(&mut self) -> Option<char> {
        let mut chars = self.source[self.current..].chars();
        let ch = chars.next();
        if let Some(c) = ch {
            self.current += c.len_utf8();
        }
        ch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eq_tokens(orig: &[Token], expected: &[TokenKind]) -> bool {
        if orig.len() != expected.len() {
            return false;
        }
        for (a, b) in orig.iter().zip(expected.iter()) {
            if &a.kind != b {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_words() -> Result<(), ProgramError> {
        let mut lexer = Lexer::new("proc1, proc2");
        assert!(eq_tokens(
            &lexer.get_tokens()?,
            &vec![
                TokenKind::Word("proc1"),
                TokenKind::Comma,
                TokenKind::Word("proc2")
            ]
        ));
        Ok(())
    }

    #[test]
    fn test_pids() -> Result<(), ProgramError> {
        let mut lexer = Lexer::new("pid 123, pid 67");
        assert!(eq_tokens(
            &lexer.get_tokens()?,
            &vec![
                TokenKind::NumberPID(123),
                TokenKind::Comma,
                TokenKind::NumberPID(67)
            ]
        ));
        Ok(())
    }

    #[test]
    fn test_quotes() -> Result<(), ProgramError> {
        let mut lexer = Lexer::new(r##" "67", "😵‍💫😴" "##);
        assert!(eq_tokens(
            &lexer.get_tokens()?,
            &vec![
                TokenKind::Word("67"),
                TokenKind::Comma,
                TokenKind::Word("😵‍💫😴")
            ]
        ));
        Ok(())
    }

    #[test]
    fn test_all() -> Result<(), ProgramError> {
        let mut lexer = Lexer::new(r##"proc2, pid 6767, pro, "67", "😵‍💫😴" "##);
        assert!(eq_tokens(
            &lexer.get_tokens()?,
            &vec![
                TokenKind::Word("proc2"),
                TokenKind::Comma,
                TokenKind::NumberPID(6767),
                TokenKind::Comma,
                TokenKind::Word("pro"),
                TokenKind::Comma,
                TokenKind::Word("67"),
                TokenKind::Comma,
                TokenKind::Word("😵‍💫😴")
            ]
        ));
        Ok(())
    }

    // errors
    #[test]
    fn unexpected_char() -> Result<(), ProgramError> {
        let mut lexer = Lexer::new("🙂");
        assert_eq!(
            lexer.get_tokens().unwrap_err().kind,
            ProgramErrorKind::UnexpectedCharacter('🙂')
        );
        Ok(())
    }
    #[test]
    fn expected_pid() -> Result<(), ProgramError> {
        let mut lexer = Lexer::new("pid");
        assert_eq!(
            lexer.get_tokens().unwrap_err().kind,
            ProgramErrorKind::ExpectedPid
        );
        Ok(())
    }
    #[test]
    fn unexpected_char_pid() -> Result<(), ProgramError> {
        let mut lexer = Lexer::new("pid, 67");
        assert_eq!(
            lexer.get_tokens().unwrap_err().kind,
            ProgramErrorKind::UnexpectedCharacter(',')
        );
        Ok(())
    }
    #[test]
    fn invalid_pid() -> Result<(), ProgramError> {
        let mut lexer = Lexer::new("pid 67s");
        assert_eq!(
            lexer.get_tokens().unwrap_err().kind,
            ProgramErrorKind::InvalidPid("67s".to_owned())
        );
        Ok(())
    }
    #[test]
    fn unclosed_quote() -> Result<(), ProgramError> {
        let mut lexer = Lexer::new("\"proc\", \"proc2");
        assert_eq!(
            lexer.get_tokens().unwrap_err().kind,
            ProgramErrorKind::UnclosedQuote
        );
        Ok(())
    }
}

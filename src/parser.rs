use std::{collections::HashSet, rc::Rc};

use crate::{
    error::{
        ProgramError,
        ProgramErrorKind::{self},
    },
    lexer::{Lexer, Token, TokenKind},
    processes::Processes,
};

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    tokens: Rc<[Token<'a>]>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn from_source(source: &'a str) -> Result<Self, ProgramError> {
        let mut lexer = Lexer::new(source);
        Ok(Self {
            tokens: Rc::from(lexer.get_tokens()?.into_boxed_slice()),
            current: 0,
        })
    }
    pub fn get_processes(&mut self) -> Result<Processes, ProgramError> {
        let mut procs = Processes {
            pids: HashSet::new(),
            names: HashSet::new(),
        };
        if let Some(tk) = self.tokens.last()
            && tk.kind == TokenKind::Comma
        {
            return Err(ProgramError::new(
                ProgramErrorKind::UnexpectedCharacter(','),
                tk.span,
            ));
        }

        self.expect_token(
            &TokenKind::Word("al"),
            ProgramError::new(
                ProgramErrorKind::ExpectedOpening,
                self.tokens.first().map_or_default(|tk| tk.span),
            ),
        )?;
        let total_element_count = self
            .tokens
            .iter()
            .filter(|&tk| {
                std::mem::discriminant(&tk.kind) != std::mem::discriminant(&TokenKind::Comma)
            })
            .count()
            - 1; // doesnt include the first "al"

        loop {
            match self.next() {
                Some(Token {
                    kind: TokenKind::NumberPID(pid),
                    span: _,
                }) => {
                    procs.pids.insert(*pid);
                }
                Some(Token {
                    kind: TokenKind::Word(name),
                    span: _,
                }) => {
                    procs.names.insert(name.to_string());
                }
                Some(Token {
                    kind: TokenKind::Comma,
                    span: _,
                }) => {}
                None => break,
            }
            if total_element_count > 1 && self.remaining_len() <= 3 {
                let last_tk_span = self.tokens.last().unwrap().span; // safe to unwrap
                self.expect_token(
                    &TokenKind::Word("y"),
                    ProgramError::new(ProgramErrorKind::ExpectedEnding, last_tk_span),
                )?;
                self.expect_token(
                    &TokenKind::Word("al"),
                    ProgramError::new(ProgramErrorKind::ExpectedEnding, last_tk_span),
                )?;
                match self.next() {
                    Some(Token {
                        kind: TokenKind::NumberPID(pid),
                        span: _,
                    }) => {
                        procs.pids.insert(*pid);
                    }
                    Some(Token {
                        kind: TokenKind::Word(name),
                        span: _,
                    }) => {
                        procs.names.insert(name.to_string());
                    }
                    _ => break,
                }
            } else if self.is_at_the_end() {
                break;
            } else {
                self.expect_token(
                    &TokenKind::Comma,
                    ProgramError::new(ProgramErrorKind::ExpectedComma, self.peek().unwrap().span), // safe to unwrap, not at the end
                )?;
            }
        }

        Ok(procs)
    }
    fn remaining_len(&self) -> usize {
        self.tokens.len() - self.current
    }
    // consume the expected token, otherwise fails.
    fn expect_token(
        &mut self,
        expected: &TokenKind<'a>,
        if_fail: ProgramError,
    ) -> Result<(), ProgramError> {
        if matches!(self.peek(), Some(t) if t.kind == *expected) {
            self.next();
            Ok(())
        } else {
            Err(if_fail)
        }
    }
    fn is_at_the_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.current)
    }
    fn next(&mut self) -> Option<&Token<'a>> {
        if self.is_at_the_end() {
            return None;
        }
        self.current += 1;
        self.tokens.get(self.current - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] // ???
    fn test() -> Result<(), ProgramError> {
        let mut parser = Parser::from_source("al proc1, proc2 y al pid 67")?;
        let pids = {
            let mut hashset = HashSet::new();
            hashset.insert(67usize);
            hashset
        };
        let names = {
            let mut hashset = HashSet::new();
            hashset.insert("proc1".to_owned());
            hashset.insert("proc2".to_owned());
            hashset
        };
        assert_eq!(parser.get_processes()?, Processes { pids, names });
        Ok(())
    }
    #[test]
    fn test2() -> Result<(), ProgramError> {
        let mut parser = Parser::from_source("al proc1 y al pid 67")?;
        let pids = {
            let mut hashset = HashSet::new();
            hashset.insert(67usize);
            hashset
        };
        let names = {
            let mut hashset = HashSet::new();
            hashset.insert("proc1".to_owned());
            hashset
        };
        assert_eq!(parser.get_processes()?, Processes { pids, names });
        Ok(())
    }
    #[test]
    fn test_bad_opening() -> Result<(), ProgramError> {
        let mut parser = Parser::from_source("proc1, proc2 y al pid 67")?;
        assert_eq!(
            parser.get_processes().unwrap_err().kind,
            ProgramErrorKind::ExpectedOpening
        );
        Ok(())
    }
    #[test]
    fn test_bad_ending() -> Result<(), ProgramError> {
        let mut parser = Parser::from_source("al proc1, proc2, pid 67")?;
        assert_eq!(
            parser.get_processes().unwrap_err().kind,
            ProgramErrorKind::ExpectedEnding
        );
        Ok(())
    }
}

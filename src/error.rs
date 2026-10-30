use std::io::Write;

use crate::common::Span;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProgramErrorKind {
    ExpectedComma,
    ExpectedPid,
    InvalidPid(String),
    ExpectedOpening,
    ExpectedEnding,
    UnexpectedCharacter(char),
    UnclosedQuote,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProgramError {
    pub kind: ProgramErrorKind,
    pub span: Span,
}

struct ErrorInfo {
    name: &'static str,
    description: Option<String>,
}
trait IntoErrorInfo {
    fn into_error_info(&self) -> ErrorInfo;
}
impl IntoErrorInfo for ProgramErrorKind {
    fn into_error_info(&self) -> ErrorInfo {
        match &self {
            ProgramErrorKind::ExpectedComma => ErrorInfo {
                name: "Se esperaba una coma.",
                description: None,
            },
            ProgramErrorKind::ExpectedPid => ErrorInfo {
                name: "Se esperaba un pid.",
                description: None,
            },
            ProgramErrorKind::InvalidPid(s) => ErrorInfo {
                name: "PID Invalido.",
                description: Some(format!("`{s}` no es un número valido")),
            },
            ProgramErrorKind::ExpectedOpening => ErrorInfo {
                name: "Se esperaba `al` antes del primer programa.",
                description: None,
            },
            ProgramErrorKind::ExpectedEnding => ErrorInfo {
                name: "Se esperaba `y al` antes del ultimo programa.",
                description: None,
            },
            ProgramErrorKind::UnexpectedCharacter(ch) => ErrorInfo {
                name: "Carácter inesperado o invalido.",
                description: Some(format!("`{ch}` no es un caracter valido o esperado")),
            },
            ProgramErrorKind::UnclosedQuote => ErrorInfo {
                name: "Se esperaba `\"` para terminar el nombre.",
                description: None,
            },
        }
    }
}

impl ProgramError {
    pub fn new(kind: ProgramErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
    pub fn show_error(&self, source: &str) {
        let e_info = self.kind.into_error_info();
        let character_count_substr = self.calculate_error_char_count(source);
        let capped = character_count_substr.clamp(0, 60);
        let (substr, has_been_capped) = {
            if capped == character_count_substr {
                (&source[self.span.start..self.span.end], false)
            } else {
                let uncapped = &source[self.span.start..self.span.end];
                let mut total_bytes = self.span.start;
                for (idx, ch) in uncapped.char_indices() {
                    if idx == capped {
                        break;
                    }
                    total_bytes += ch.len_utf8();
                }
                (&source[self.span.start..total_bytes], true)
            }
        };

        eprintln!("error: {}", e_info.name);
        eprintln!("|");
        eprintln!("| {}", substr);
        eprint!("| ");
        for _ in 0..capped {
            eprint!("^");
        }
        if has_been_capped {
            eprint!("...")
        }
        if let Some(desc) = e_info.description {
            eprintln!("\ninfo: {desc}");
        }
        if has_been_capped {
            eprintln!("\nel texto que contenia el error era muy largo; no se ha mostrado completo.")
        }
        let _ = std::io::stderr().flush();
    }
    fn calculate_error_char_count(&self, source: &str) -> usize {
        let substr = &source[self.span.start..self.span.end];
        substr.chars().count() // for utf-8
    }
}

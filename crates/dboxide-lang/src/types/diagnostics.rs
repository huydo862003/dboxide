use std::path::PathBuf;

use crate::syntax::ast::SyntaxKind;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
  UnexpectedEof = 0,
  UnexpectedChar,
  UnterminatedString,

  UnexpectedToken,
  ExpectedToken,
  InvalidTopElement,
  UnclosedDelimiter,
  MissingSyntaxNode,

  FileNotFound,
}

impl DiagnosticCode {
  pub fn as_str(&self) -> &'static str {
    match self {
      DiagnosticCode::UnexpectedEof => "unexpected-eof",
      DiagnosticCode::UnexpectedChar => "unexpected-char",
      DiagnosticCode::UnterminatedString => "unterminated-string",
      DiagnosticCode::UnexpectedToken => "unexpected-token",
      DiagnosticCode::ExpectedToken => "expected-token",
      DiagnosticCode::InvalidTopElement => "invalid-top-element",
      DiagnosticCode::UnclosedDelimiter => "unclosed-delimiter",
      DiagnosticCode::MissingSyntaxNode => "missing-syntax-node",
      DiagnosticCode::FileNotFound => "file-not-found",
    }
  }
}

/// Compilation diagnostics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Diagnostic {
  /* Lexer */
  UnexpectedEof {
    expected: char,
    start_offset: usize,
    end_offset: usize,
  },
  UnexpectedChar {
    ch: char,
    offset: usize,
  },
  UnterminatedString {
    start_offset: usize,
    end_offset: usize,
  },

  /* Parser */
  InvalidTopLevelElement {
    start_offset: usize,
    end_offset: usize,
  },
  UnexpectedToken {
    expected: &'static str,
    start_offset: usize,
    end_offset: usize,
  },
  MissingExpectedToken {
    expected: &'static str,
    start_offset: usize,
    end_offset: usize,
  },
  UnclosedDelimiter {
    delimiter: &'static str,
    open_offset: usize,
  },
  MissingSyntaxNode {
    expected: SyntaxKind,
    start_offset: usize,
    end_offset: usize,
  },

  /* File */
  FileNotFound {
    path: PathBuf,
    start_offset: usize,
    end_offset: usize,
  }
}

impl Diagnostic {
  pub fn offsets(&self) -> Option<(usize, usize)> {
    match self {
      Diagnostic::UnexpectedEof {
        start_offset,
        end_offset,
        ..
      }
      | Diagnostic::UnterminatedString {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnexpectedToken {
        start_offset,
        end_offset,
        ..
      }
      | Diagnostic::MissingExpectedToken {
        start_offset,
        end_offset,
        ..
      }
      | Diagnostic::InvalidTopLevelElement {
        start_offset,
        end_offset,
      }
      | Diagnostic::MissingSyntaxNode {
        start_offset,
        end_offset,
        ..
      }
      | Diagnostic::FileNotFound {
        start_offset,
        end_offset,
        ..
      } => Some((*start_offset, *end_offset)),
      Diagnostic::UnexpectedChar { offset, .. } => Some((*offset, *offset + 1)),
      Diagnostic::UnclosedDelimiter { open_offset, .. } => Some((*open_offset, *open_offset)),
    }
  }

  pub fn message(&self) -> String {
    match self {
      Diagnostic::UnexpectedEof { expected, .. } => {
        format!("unexpected end of input, expected '{expected}'")
      }
      Diagnostic::UnexpectedChar { ch, .. } => format!("unexpected character '{ch}'"),
      Diagnostic::UnterminatedString { .. } => "unterminated string literal".to_string(),
      Diagnostic::UnexpectedToken { expected, .. } => {
        format!("unexpected token, expected {expected}")
      }
      Diagnostic::MissingExpectedToken { expected, .. } => format!("expected {expected}"),
      Diagnostic::InvalidTopLevelElement { .. } => {
        "expected use declaration, block-form element declaration or inline-form element declaration"
          .to_string()
      }
      Diagnostic::UnclosedDelimiter { delimiter, .. } => format!("unclosed '{delimiter}'"),
      Diagnostic::MissingSyntaxNode { expected, .. } => format!("missing {expected:?}"),
      Diagnostic::FileNotFound { path, .. } => format!("File not found: {path:?}"),
    }
  }

  pub fn code(&self) -> DiagnosticCode {
    match self {
      Diagnostic::UnexpectedEof { .. } => DiagnosticCode::UnexpectedEof,
      Diagnostic::UnexpectedChar { .. } => DiagnosticCode::UnexpectedChar,
      Diagnostic::UnterminatedString { .. } => DiagnosticCode::UnterminatedString,
      Diagnostic::UnexpectedToken { .. } => DiagnosticCode::UnexpectedToken,
      Diagnostic::MissingExpectedToken { .. } => DiagnosticCode::ExpectedToken,
      Diagnostic::InvalidTopLevelElement { .. } => DiagnosticCode::InvalidTopElement,
      Diagnostic::UnclosedDelimiter { .. } => DiagnosticCode::UnclosedDelimiter,
      Diagnostic::MissingSyntaxNode { .. } => DiagnosticCode::MissingSyntaxNode,
      Diagnostic::FileNotFound { .. } => DiagnosticCode::FileNotFound,
    }
  }
}

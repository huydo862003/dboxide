#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
  UnexpectedEof = 0,
  UnexpectedChar,
  UnterminatedString,
}

impl DiagnosticCode {
  pub fn as_str(&self) -> &'static str {
    match self {
      DiagnosticCode::UnexpectedEof => "unexpected-eof",
      DiagnosticCode::UnexpectedChar => "unexpected-char",
      DiagnosticCode::UnterminatedString => "unterminated-string",
    }
  }
}

/// Compilation diagnostics
/// When multiple variants match, use the first (most specific) one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diagnostic {
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
}

impl Diagnostic {
  pub fn offsets(&self) -> Option<(usize, usize)> {
    match self {
      Diagnostic::UnexpectedEof { start_offset, end_offset, .. } => Some((*start_offset, *end_offset)),
      Diagnostic::UnexpectedChar { offset, .. } => Some((*offset, *offset + 1)),
      Diagnostic::UnterminatedString { start_offset, end_offset } => Some((*start_offset, *end_offset)),
    }
  }

  pub fn message(&self) -> String {
    match self {
      Diagnostic::UnexpectedEof { expected, .. } => {
        format!("unexpected end of input, expected '{expected}'")
      }
      Diagnostic::UnexpectedChar { ch, .. } => {
        format!("unexpected character '{ch}'")
      }
      Diagnostic::UnterminatedString { .. } => {
        "unterminated string literal".to_string()
      }
    }
  }

  pub fn code(&self) -> DiagnosticCode {
    match self {
      Diagnostic::UnexpectedEof { .. } => DiagnosticCode::UnexpectedEof,
      Diagnostic::UnexpectedChar { .. } => DiagnosticCode::UnexpectedChar,
      Diagnostic::UnterminatedString { .. } => DiagnosticCode::UnterminatedString,
    }
  }
}

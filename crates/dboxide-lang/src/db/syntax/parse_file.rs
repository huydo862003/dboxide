use salsa::{Accumulator, Database, tracked};

use crate::FileHandle;
use crate::ast::{RedNode, cache::CACHE};
use crate::db::types::{CheapRedNode, Diagnostics, File, FileParseResult, TrackedRedNode};
use crate::diagnostics::Diagnostic;
use crate::parse::{ParseCtx, ParseResult};

/// Parse a file into an AST
#[tracked]
pub fn parse_file<'db>(db: &'db dyn Database, file: File) -> FileParseResult<'db> {
  let mut diagnostics = vec![];
  let empty_file = FileHandle::Content {
    path: file.filepath(db).into(),
    content: String::new(),
    ctime: file.ctime(db),
    mtime: file.mtime(db),
  };

  let stream = file.handle(db).open().unwrap_or_else(|| {
    diagnostics.push(Diagnostic::FileNotFound {
      path: file.filepath(db).into(),
      start_offset: 0,
      end_offset: 0,
    });

    empty_file.open().unwrap()
  });

  let parser = ParseCtx::new(stream, CACHE.with(|value| value.clone()));

  let ParseResult {
    ast,
    diagnostics: parse_diagnostics,
  } = parser.parse();

  diagnostics.extend_from_slice(&parse_diagnostics);
  diagnostics
    .iter()
    .for_each(|diagnostic| Diagnostics::accumulate(Diagnostics(diagnostic.clone()), db));

  FileParseResult::new(
    db,
    TrackedRedNode::new(
      db,
      CheapRedNode {
        node: RedNode::new_root(ast),
        file,
      },
    ),
    diagnostics,
  )
}

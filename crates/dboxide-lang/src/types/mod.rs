use std::{fs::File, io::BufReader, iter::Peekable, path::PathBuf, time::SystemTime};
use utf8_chars::BufReadCharsExt;

pub type PeekableStream<'a, T> = Peekable<Box<dyn Iterator<Item = T> + 'a>>;

pub enum FileHandle {
  File {
    path: PathBuf,
    ctime: SystemTime,
    mtime: SystemTime,
  },
  Content {
    path: PathBuf,
    content: String,
    ctime: SystemTime,
    mtime: SystemTime,
  },
}

impl FileHandle {
  pub fn open<'a>(&'a self) -> Option<PeekableStream<'a, char>> {
    match self {
      FileHandle::File { path, .. } => Some(
        (Box::new(FileIterator::new(BufReader::new(File::open(path).ok()?)))
          as Box<dyn Iterator<Item = char>>)
          .peekable(),
      ),
      FileHandle::Content { content, .. } => {
        Some((Box::new(content.chars()) as Box<dyn Iterator<Item = char>>).peekable())
      }
    }
  }
}

struct FileIterator {
  reader_ptr: *mut BufReader<File>,
  iter: Box<dyn Iterator<Item = char>>,
}

impl FileIterator {
  fn new(reader: BufReader<File>) -> Self {
    let reader = Box::leak(Box::new(reader));
    let reader_ptr = reader as *mut BufReader<File>;
    let iter = Box::new(reader.chars().filter_map(|c| c.ok()));

    FileIterator { reader_ptr, iter }
  }
}

impl Iterator for FileIterator {
  type Item = char;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}

impl Drop for FileIterator {
  fn drop(&mut self) {
    unsafe {
      std::mem::drop(Box::from_raw(self.reader_ptr));
    }
  }
}

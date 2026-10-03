/// Cleans and unescapes a raw string literal of any DBML quote style
pub fn interpret_string(raw: &str) -> String {
  let s = raw.trim();
  if s.starts_with("'''") && s.ends_with("'''") && s.len() >= 6 {
    normalize_multiline_indent(&unescape_non_oq_string(&s[3..s.len() - 3]))
  } else if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
    || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
  {
    unescape_non_oq_string(&s[1..s.len() - 1])
  } else if s.starts_with('`') && s.ends_with('`') && s.len() >= 2 {
    unescape_oq_string(&s[1..s.len() - 1])
  } else {
    s.to_string()
  }
}

/// Unescapes standard escape sequences in a string literal
fn unescape_non_oq_string(content: &str) -> String {
  unescaper::unescape(content).unwrap_or_else(|_| content.to_string())
}

/// Unescapes an open-quoted string where only backticks can be escaped with backslash
fn unescape_oq_string(content: &str) -> String {
  let mut result = String::with_capacity(content.len());
  let mut chars = content.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\\' && chars.peek() == Some(&'`') {
      result.push('`');
      chars.next();
    } else {
      result.push(c);
    }
  }
  result
}

/// Strips common leading indentation from multiline string content
pub fn normalize_multiline_indent(content: &str) -> String {
  let lines: Vec<&str> = content.split('\n').collect();
  let first_non_empty = lines.iter().position(|line| !line.trim_start().is_empty());
  let Some(start) = first_non_empty else {
    return content.to_string();
  };
  let trimmed_top = &lines[start..];
  let non_empty: Vec<&str> = trimmed_top
    .iter()
    .copied()
    .filter(|line| !line.trim_start().is_empty())
    .collect();
  if non_empty.is_empty() {
    return trimmed_top.join("\n");
  }
  let min_indent = non_empty
    .iter()
    .map(|line| line.len() - line.trim_start().len())
    .min()
    .unwrap_or(0);
  trimmed_top
    .iter()
    .map(|line| {
      if line.len() >= min_indent {
        &line[min_indent..]
      } else {
        line.trim_start()
      }
    })
    .collect::<Vec<_>>()
    .join("\n")
}

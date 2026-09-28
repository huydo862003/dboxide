use crate::syntax::parse::tests::utils::*;

#[test]
fn setting_simple() {
  let tree = parse_source(
    "Table t {
  id int [pk]
}",
  );
  assert!(tree.contains(
    r#"(SettingListItemName
              "pk")"#
  ));
}

#[test]
fn setting_with_value() {
  let tree = parse_source(
    "Table t {
  id int [default: 0]
}",
  );
  assert!(tree.contains("SettingListItemName"));
  assert!(tree.contains("SettingListItemValue"));
}

#[test]
fn setting_multi_word_name() {
  let tree = parse_source(
    "Table t {
  id int [not null]
}",
  );
  assert!(tree.contains("not"));
  assert!(tree.contains("null"));
  // "not null" should be a single setting item
  let item_count = tree.matches("SettingListItem\n").count();
  assert_eq!(item_count, 1);
}

#[test]
fn multiple_settings() {
  let tree = parse_source(
    "Table t {
  id int [pk, not null, increment]
}",
  );
  let item_count = tree.matches("SettingListItem\n").count();
  assert_eq!(item_count, 3);
}

#[test]
fn setting_backtick_value() {
  let tree = parse_source(
    "Table t {
  created_at timestamp [default: `now()`]
}",
  );
  assert!(tree.contains("`now()`"));
}

#[test]
fn setting_double_quoted_name() {
  let (tree, diags) = parse_source_with_diagnostics(
    r#"Table t {
  id int ["custom name": 'val']
}"#,
  );
  assert!(diags.is_empty());
  assert!(tree.contains("SettingListItemName"));
  assert!(tree.contains(r#""\"custom name\"""#));
}

#[test]
fn setting_rejects_single_quoted_name() {
  let (_tree, diags) = parse_source_with_diagnostics(
    "Table t {
  id int ['custom name': 'val']
}",
  );
  assert!(!diags.is_empty());
}

#[test]
fn setting_with_numeric_value() {
  let tree = parse_source(
    r#"Table t {
  id int [default: 42]
}"#,
  );
  assert!(tree.contains("SettingListItemName"));
  assert!(tree.contains("SettingListItemValue"));
  assert!(tree.contains("42"));
}

#[test]
fn setting_with_string_value() {
  let tree = parse_source(
    r#"Table t {
  name varchar [default: 'guest']
}"#,
  );
  assert!(tree.contains("'guest'"));
}

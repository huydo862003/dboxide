mod dep;
mod diagramview;
mod enums;
mod metadata;
mod multifile;
mod project;
mod ref_element;
mod sticky_note;
mod table;
mod table_group;
mod tablepartial;

use super::*;

#[test]
fn prelude_defines_all_element_types() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(&db, prelude, "test.dbml", "", HashMap::new());
  let members = pretype_members(&db, file);
  assert_contains_symbol(&members, "Table");
  assert_contains_symbol(&members, "Enum");
  assert_contains_symbol(&members, "Ref");
  assert_contains_symbol(&members, "Project");
  assert_contains_symbol(&members, "StickyNote");
  assert_contains_symbol(&members, "TableGroup");
  assert_contains_symbol(&members, "RefAction");
}

#[test]
fn empty_file_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(&db, prelude, "test.dbml", "", HashMap::new());
  assert_no_diagnostics(&db, file);
}

use super::*;

mod inactive_ref {
  use super::*;

  #[test]
  fn inactive_flag_no_diagnostic() {
    let db = DboxideDatabase::default();
    let prelude = setup(&db);
    let file = dbml(
      &db,
      prelude,
      "test.dbml",
      r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [inactive]
"#,
      HashMap::new(),
    );
    assert_no_diagnostics(&db, file);
  }

  #[test]
  fn inactive_with_delete_action_no_diagnostic() {
    let db = DboxideDatabase::default();
    let prelude = setup(&db);
    let file = dbml(
      &db,
      prelude,
      "test.dbml",
      r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [delete: cascade, inactive]
"#,
      HashMap::new(),
    );
    assert_no_diagnostics(&db, file);
  }
}

mod optional_ref {
  use super::*;

  fn ref_with_op_no_diagnostic(op: &str) {
    let db = DboxideDatabase::default();
    let prelude = setup(&db);
    let content = format!(
      r#"
Table users {{ id integer [pk] }}
Table posts {{ user_id integer }}
Ref: posts.user_id {op} users.id
"#,
    );
    let file = dbml(&db, prelude, "test.dbml", &content, HashMap::new());
    assert_no_diagnostics(&db, file);
  }

  #[test]
  fn greater_than() {
    ref_with_op_no_diagnostic(">");
  }

  #[test]
  fn greater_than_nullable_to() {
    ref_with_op_no_diagnostic(">?");
  }

  #[test]
  fn nullable_from_greater_than() {
    ref_with_op_no_diagnostic("?>");
  }

  #[test]
  fn nullable_from_greater_than_nullable_to() {
    ref_with_op_no_diagnostic("?>?");
  }

  #[test]
  fn less_than() {
    ref_with_op_no_diagnostic("<");
  }

  #[test]
  fn less_than_nullable_to() {
    ref_with_op_no_diagnostic("<?");
  }

  #[test]
  fn nullable_from_less_than() {
    ref_with_op_no_diagnostic("?<");
  }

  #[test]
  fn nullable_from_less_than_nullable_to() {
    ref_with_op_no_diagnostic("?<?");
  }

  #[test]
  fn one_to_one() {
    ref_with_op_no_diagnostic("-");
  }

  #[test]
  fn one_to_one_nullable_to() {
    ref_with_op_no_diagnostic("-?");
  }

  #[test]
  fn nullable_from_one_to_one() {
    ref_with_op_no_diagnostic("?-");
  }

  #[test]
  fn nullable_from_one_to_one_nullable_to() {
    ref_with_op_no_diagnostic("?-?");
  }

  #[test]
  fn many_to_many() {
    ref_with_op_no_diagnostic("<>");
  }

  #[test]
  fn many_to_many_nullable_to() {
    ref_with_op_no_diagnostic("<>?");
  }

  #[test]
  fn nullable_from_many_to_many() {
    ref_with_op_no_diagnostic("?<>");
  }

  #[test]
  fn nullable_from_many_to_many_nullable_to() {
    ref_with_op_no_diagnostic("?<>?");
  }
}

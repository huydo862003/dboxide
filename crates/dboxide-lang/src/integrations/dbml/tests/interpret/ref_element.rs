use super::*;

mod inactive_ref {
  use super::*;

  fn find_ref_field_line<'db>(
    db: &'db DboxideDatabase,
    elements: &[ElementObj<'db>],
  ) -> Option<ElementObj<'db>> {
    let ref_element = elements
      .iter()
      .find(|element| match element.element_typ(db) {
        Typ::Element(element) => element.name(db).clone() == "Ref",
        _ => false,
      })?;
    field_list(db, ref_element, "endpoints").into_iter().next()
  }

  #[test]
  fn inactive_setting_is_true() {
    let db = DboxideDatabase::default();
    let prelude = setup(&db);
    let elements = interpret(
      &db,
      prelude,
      r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [inactive]
"#,
    );
    let field_line = find_ref_field_line(&db, &elements).expect("expected ref field line");
    assert_eq!(
      extract_field_bool(&db, &field_line, "inactive"),
      Some(true),
      "expected inactive=true on RefBinary with [inactive] setting",
    );
  }

  #[test]
  fn absent_inactive_setting_has_no_field() {
    let db = DboxideDatabase::default();
    let prelude = setup(&db);
    let elements = interpret(
      &db,
      prelude,
      r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id
"#,
    );
    let field_line = find_ref_field_line(&db, &elements).expect("expected ref field line");
    assert!(
      !has_field(&db, &field_line, "inactive"),
      "expected no inactive field when [inactive] setting is absent",
    );
  }

  #[test]
  fn inactive_with_delete_action_both_captured() {
    let db = DboxideDatabase::default();
    let prelude = setup(&db);
    let elements = interpret(
      &db,
      prelude,
      r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [delete: cascade, inactive]
"#,
    );
    let field_line = find_ref_field_line(&db, &elements).expect("expected ref field line");
    assert_eq!(extract_field_bool(&db, &field_line, "inactive"), Some(true));
    assert_eq!(
      get_field_string(&db, &field_line, "delete").as_deref(),
      Some("cascade"),
    );
  }
}

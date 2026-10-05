use crate::syntax::parse::tests::utils::*;

#[test]
fn attribute_basic() {
  let tree = parse_source(
    "Project p {
  database_type: 'PostgreSQL'
}",
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "Project")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "p"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementAttributeDeclaration
        (ElementAttributeDeclarationName
          (IdentExpr
            "database_type"))
        ":"
        (ElementAttributeDeclarationValue
          (SqStringExpr
            " "
            "'PostgreSQL'")))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn attribute_string_value() {
  let tree = parse_source(
    "Project p {
  Note: 'hello world'
}",
  );
  assert!(tree.contains("ElementAttributeDeclaration"));
  assert!(tree.contains("'hello world'"));
}

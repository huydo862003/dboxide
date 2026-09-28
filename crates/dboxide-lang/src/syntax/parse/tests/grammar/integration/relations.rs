use crate::syntax::parse::tests::utils::*;

/* Dep */

#[test]
fn comprehensive_dep() {
  let input = r#"Table users {
  id int
}
Table orders {
  user_id int
}
Table my_schema.events {
  id int
  ts int
}

Table another_schema.booking {
  id int
  ts int
}


// short-form
// no error
Dep: users -> orders
Dep: orders.user_id <- users.id
Dep: my_schema.events -> users
Dep: my_schema.events.id -> users.id
Dep: my_schema.events.id <- another_schema.booking.id

// full-form
// no error
Dep {
  users -> orders
  my_schema.events -> users
  users.id -> orders.user_id
  my_schema.events.id -> users.id
}

// inline on table header
// no error
Table good_header_bare [dep: <- users] { id int }
Table good_header_schema [dep: <- my_schema.events] { id int }

// inline on column
// no error
Table good_col_bare { id int [dep: -> users.id] }
Table good_col_schema { id int [dep: -> my_schema.events.id] }

// short-form
// no error
Dep: users -> unknown_table
Dep: users.unknown_col -> users.id
Dep: unknown_schema.events -> users
Dep: my_schema.unknown_table -> users
Dep: my_schema.events.unknown_col -> users.id

// full-form
// no error
Dep {
  users -> unknown_a
  my_schema.events -> unknown_b
  users.id -> unknown_c.col
  my_schema.events.id -> users.unknown_d
}

// inline on table header
// no error
Table bad_header_bare [dep: <- unknown_source] { id int }
Table bad_header_schema [dep: <- unknown_schema.events] { id int }

// inline on column
// no error
Table bad_col_bare { id int [dep: -> unknown_table.col] }
Table bad_col_schema { id int [dep: -> my_schema.unknown_table.col] }
"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int"))
      "\n"
      "}"))
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "orders"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "user_id")
        " "
        (IdentExpr
          "int"))
      "\n"
      "}"))
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (InfixExpr
        (IdentExpr
          "my_schema")
        "."
        (IdentExpr
          "events")))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "ts")
        " "
        (IdentExpr
          "int"))
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (InfixExpr
        (IdentExpr
          "another_schema")
        "."
        (IdentExpr
          "booking")))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "ts")
        " "
        (IdentExpr
          "int"))
      "\n"
      "}"))
  "\n"
  "\n"
  "\n"
  "// short-form"
  "\n"
  "// no error"
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (IdentExpr
          "users")
        " "
        "->"
        (IdentExpr
          " "
          "orders"))))
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (InfixExpr
          (IdentExpr
            "orders")
          "."
          (IdentExpr
            "user_id"))
        " "
        "<-"
        (InfixExpr
          (IdentExpr
            " "
            "users")
          "."
          (IdentExpr
            "id")))))
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (InfixExpr
          (IdentExpr
            "my_schema")
          "."
          (IdentExpr
            "events"))
        " "
        "->"
        (IdentExpr
          " "
          "users"))))
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (InfixExpr
          (InfixExpr
            (IdentExpr
              "my_schema")
            "."
            (IdentExpr
              "events"))
          "."
          (IdentExpr
            "id"))
        " "
        "->"
        (InfixExpr
          (IdentExpr
            " "
            "users")
          "."
          (IdentExpr
            "id")))))
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (InfixExpr
          (InfixExpr
            (IdentExpr
              "my_schema")
            "."
            (IdentExpr
              "events"))
          "."
          (IdentExpr
            "id"))
        " "
        "<-"
        (InfixExpr
          (InfixExpr
            (IdentExpr
              " "
              "another_schema")
            "."
            (IdentExpr
              "booking"))
          "."
          (IdentExpr
            "id")))))
  "\n"
  "\n"
  "// full-form"
  "\n"
  "// no error"
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (InfixExpr
          (IdentExpr
            "users")
          " "
          "->"
          (IdentExpr
            " "
            "orders")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (InfixExpr
          (InfixExpr
            (IdentExpr
              "my_schema")
            "."
            (IdentExpr
              "events"))
          " "
          "->"
          (IdentExpr
            " "
            "users")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (InfixExpr
          (InfixExpr
            (IdentExpr
              "users")
            "."
            (IdentExpr
              "id"))
          " "
          "->"
          (InfixExpr
            (IdentExpr
              " "
              "orders")
            "."
            (IdentExpr
              "user_id"))))
      "\n"
      "  "
      (ElementFieldDeclaration
        (InfixExpr
          (InfixExpr
            (InfixExpr
              (IdentExpr
                "my_schema")
              "."
              (IdentExpr
                "events"))
            "."
            (IdentExpr
              "id"))
          " "
          "->"
          (InfixExpr
            (IdentExpr
              " "
              "users")
            "."
            (IdentExpr
              "id"))))
      "\n"
      "}"))
  "\n"
  "\n"
  "// inline on table header"
  "\n"
  "// no error"
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "good_header_bare"))
    " "
    (SettingList
      "["
      (SettingListItem
        (SettingListItemName
          "dep")
        ":"
        (SettingListItemValue
          " "
          "<-"
          " "
          "users"))
      "]")
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int"))
      " "
      "}"))
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "good_header_schema"))
    " "
    (SettingList
      "["
      (SettingListItem
        (SettingListItemName
          "dep")
        ":"
        (SettingListItemValue
          " "
          "<-"
          " "
          "my_schema"
          "."
          "events"))
      "]")
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int"))
      " "
      "}"))
  "\n"
  "\n"
  "// inline on column"
  "\n"
  "// no error"
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "good_col_bare"))
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "dep")
            ":"
            (SettingListItemValue
              " "
              "->"
              " "
              "users"
              "."
              "id"))
          "]"))
      " "
      "}"))
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "good_col_schema"))
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "dep")
            ":"
            (SettingListItemValue
              " "
              "->"
              " "
              "my_schema"
              "."
              "events"
              "."
              "id"))
          "]"))
      " "
      "}"))
  "\n"
  "\n"
  "// short-form"
  "\n"
  "// no error"
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (IdentExpr
          "users")
        " "
        "->"
        (IdentExpr
          " "
          "unknown_table"))))
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (InfixExpr
          (IdentExpr
            "users")
          "."
          (IdentExpr
            "unknown_col"))
        " "
        "->"
        (InfixExpr
          (IdentExpr
            " "
            "users")
          "."
          (IdentExpr
            "id")))))
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (InfixExpr
          (IdentExpr
            "unknown_schema")
          "."
          (IdentExpr
            "events"))
        " "
        "->"
        (IdentExpr
          " "
          "users"))))
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (InfixExpr
          (IdentExpr
            "my_schema")
          "."
          (IdentExpr
            "unknown_table"))
        " "
        "->"
        (IdentExpr
          " "
          "users"))))
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (InfixExpr
          (InfixExpr
            (IdentExpr
              "my_schema")
            "."
            (IdentExpr
              "events"))
          "."
          (IdentExpr
            "unknown_col"))
        " "
        "->"
        (InfixExpr
          (IdentExpr
            " "
            "users")
          "."
          (IdentExpr
            "id")))))
  "\n"
  "\n"
  "// full-form"
  "\n"
  "// no error"
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Dep")
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (InfixExpr
          (IdentExpr
            "users")
          " "
          "->"
          (IdentExpr
            " "
            "unknown_a")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (InfixExpr
          (InfixExpr
            (IdentExpr
              "my_schema")
            "."
            (IdentExpr
              "events"))
          " "
          "->"
          (IdentExpr
            " "
            "unknown_b")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (InfixExpr
          (InfixExpr
            (IdentExpr
              "users")
            "."
            (IdentExpr
              "id"))
          " "
          "->"
          (InfixExpr
            (IdentExpr
              " "
              "unknown_c")
            "."
            (IdentExpr
              "col"))))
      "\n"
      "  "
      (ElementFieldDeclaration
        (InfixExpr
          (InfixExpr
            (InfixExpr
              (IdentExpr
                "my_schema")
              "."
              (IdentExpr
                "events"))
            "."
            (IdentExpr
              "id"))
          " "
          "->"
          (InfixExpr
            (IdentExpr
              " "
              "users")
            "."
            (IdentExpr
              "unknown_d"))))
      "\n"
      "}"))
  "\n"
  "\n"
  "// inline on table header"
  "\n"
  "// no error"
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "bad_header_bare"))
    " "
    (SettingList
      "["
      (SettingListItem
        (SettingListItemName
          "dep")
        ":"
        (SettingListItemValue
          " "
          "<-"
          " "
          "unknown_source"))
      "]")
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int"))
      " "
      "}"))
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "bad_header_schema"))
    " "
    (SettingList
      "["
      (SettingListItem
        (SettingListItemName
          "dep")
        ":"
        (SettingListItemValue
          " "
          "<-"
          " "
          "unknown_schema"
          "."
          "events"))
      "]")
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int"))
      " "
      "}"))
  "\n"
  "\n"
  "// inline on column"
  "\n"
  "// no error"
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "bad_col_bare"))
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "dep")
            ":"
            (SettingListItemValue
              " "
              "->"
              " "
              "unknown_table"
              "."
              "col"))
          "]"))
      " "
      "}"))
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "bad_col_schema"))
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "dep")
            ":"
            (SettingListItemValue
              " "
              "->"
              " "
              "my_schema"
              "."
              "unknown_table"
              "."
              "col"))
          "]"))
      " "
      "}"))
  "\n"
  "")"#;
  assert_eq!(tree, expected);
}

/* partial_injection */

#[test]
fn comprehensive_partial_injection() {
  let input = r#"TablePartial with_timestamp {
  timestamp uuid
}

Table users {
  ~with_timestamp
}
"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "TablePartial")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "with_timestamp"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "timestamp")
        " "
        (IdentExpr
          "uuid"))
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (PrefixExpr
          "~"
          (IdentExpr
            "with_timestamp")))
      "\n"
      "}"))
  "\n"
  "")"#;
  assert_eq!(tree, expected);
}

/* ref_setting */

#[test]
fn comprehensive_ref_setting() {
  let input = r#"Table Users {
	id integer
    status v2.status [default: v2.status.new]
    
    referrer integer [ref: -id]
}

enum v2.status {
	churn
    new [note: 'This is a new employee']
}"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "Users"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "\t"
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "integer"))
      "\n"
      "    "
      (ElementFieldDeclaration
        (IdentExpr
          "status")
        " "
        (InfixExpr
          (IdentExpr
            "v2")
          "."
          (IdentExpr
            "status"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              " "
              "v2"
              "."
              "status"
              "."
              "new"))
          "]"))
      "\n"
      "    "
      "\n"
      "    "
      (ElementFieldDeclaration
        (IdentExpr
          "referrer")
        " "
        (IdentExpr
          "integer")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "ref")
            ":"
            (SettingListItemValue
              " "
              "-"
              "id"))
          "]"))
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "enum")
    " "
    (BlockElementDeclarationTargetFragment
      (InfixExpr
        (IdentExpr
          "v2")
        "."
        (IdentExpr
          "status")))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "\t"
      (ElementFieldDeclaration
        (IdentExpr
          "churn"))
      "\n"
      "    "
      (ElementFieldDeclaration
        (IdentExpr
          "new")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "note")
            ":"
            (SettingListItemValue
              " "
              "'This is a new employee'"))
          "]"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

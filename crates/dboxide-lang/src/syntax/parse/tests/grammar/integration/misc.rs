use crate::syntax::parse::tests::utils::*;

/* erroneous_setting */

#[test]
fn comprehensive_erroneous_setting() {
  let input = r#"Table users as U {
  id int [pk, increment]
}

Table user_role_in_diagram {
  user_id int
  diagram_id int
  role int [note: 'Role = sum(user's available permissions bit value)']
  Indexes {
    (user_id, diagram_id) [pk]
  }
}

Table permissions {
  bit int [pk] 
  name varchar
}

Table diagrams {
  id int [pk, increment] // auto-increment
}

Ref: "users"."id" - "user_role_in_diagram"."user_id"

Ref: "diagrams"."id" - "user_role_in_diagram"."diagram_id"
"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    "as"
    " "
    (ElementDeclarationAlias
      (IdentExpr
        "U"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "pk"))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              "increment"))
          "]"))
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "user_role_in_diagram"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "user_id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "diagram_id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "role"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "note")
            ":"
            (SettingListItemValue
              (SqStringExpr
                " "
                "'Role = sum(user'")))
          (SettingListItem
            (SettingListItemName
              "s"
              " "
              "available"
              " "
              "permissions"
              " "
              "bit"
              " "
              "value"))
          (SettingListItem
            (SettingListItemName
              (Error
                ")"
                "']")))
          "\n"
          "  "
          (SettingListItem
            (SettingListItemName
              "Indexes"))
          " "
          (SettingListItem
            (SettingListItemName
              (Error
                "{")))
          "\n"
          "    "
          (SettingListItem
            (SettingListItemName
              (Error
                "("
                "user_id")))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              "diagram_id"))
          (SettingListItem
            (SettingListItemName
              (Error
                ")"
                " "
                "["
                "pk")))
          "]"))
      "\n"
      "  "
      "}"))
  "\n"
  (Error
    "}")
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "permissions"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "bit"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "pk"))
          "]"))
      " "
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "name"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "varchar")))
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "diagrams"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "pk"))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              "increment"))
          "]"))
      " "
      "// auto-increment"
      "\n"
      "}"))
  "\n"
  "\n"
  (InlineElementDeclaration
    (ElementDeclarationType
      "Ref")
    ":"
    " "
    (ElementFieldDeclaration
      (ElementFieldDeclarationArg
        (InfixExpr
          (InfixExpr
            (DqStringExpr
              "\"users\"")
            "."
            (DqStringExpr
              "\"id\""))
          " "
          "-"
          (InfixExpr
            (DqStringExpr
              " "
              "\"user_role_in_diagram\"")
            "."
            (DqStringExpr
              "\"user_id\""))))))
  "\n"
  "\n"
  (InlineElementDeclaration
    (ElementDeclarationType
      "Ref")
    ":"
    " "
    (ElementFieldDeclaration
      (ElementFieldDeclarationArg
        (InfixExpr
          (InfixExpr
            (DqStringExpr
              "\"diagrams\"")
            "."
            (DqStringExpr
              "\"id\""))
          " "
          "-"
          (InfixExpr
            (DqStringExpr
              " "
              "\"user_role_in_diagram\"")
            "."
            (DqStringExpr
              "\"diagram_id\""))))))
  "\n"
  "")"#;
  assert_eq!(tree, expected);
}

/* last_invalid_number */

#[test]
fn comprehensive_last_invalid_number() {
  let input = r#"Table E {
    id integer
}

Note: 12."#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "E"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "integer")))
      "\n"
      "}"))
  "\n"
  "\n"
  (InlineElementDeclaration
    (ElementDeclarationType
      "Note")
    ":"
    " "
    (ElementFieldDeclaration
      (ElementFieldDeclarationArg
        (InfixExpr
          (NumberExpr
            "12")
          "."
          (Error)))))
  "")"#;
  assert_eq!(tree, expected);
}

/* old_undocumented_syntax */

#[test]
fn comprehensive_old_undocumented_syntax() {
  let input = r#"Table "customer" {
  "customer_id" SMALLINT [pk, not null, increment]
  "store_id" TINYINT [not null]
  "first_name" VARCHAR(45) [not null]
  "last_name" VARCHAR(45) [not null, default: faLse]
  "email" VARCHAR(50) [default: NULL]
  "address_id" SMALLINT [not NULL]
  "active" BOOLEAN [not null, default: TRUE]
  "create_date" DATETIME [not null]
  "last_update" TIMESTAMP [default: `CURRENT_TIMESTAMP`]
}

Table cities {
  id integer [primary key]
  name e [default: "hello"]
  country_id integer
  note: "sasasa"
}

Table country {
	id integer
	cities string[]
}

Table citites {
	id integer
	name string
	indexes {
		id name
	}
}"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (DqStringExpr
        "\"customer\""))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (DqStringExpr
            "\"customer_id\""))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "SMALLINT"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "pk"))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              "not"
              " "
              "null"))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              "increment"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (DqStringExpr
            "\"store_id\""))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "TINYINT"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "not"
              " "
              "null"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (DqStringExpr
            "\"first_name\""))
        " "
        (ElementFieldDeclarationArg
          (CallExpr
            (IdentExpr
              "VARCHAR")
            "("
            (NumberExpr
              "45")
            ")"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "not"
              " "
              "null"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (DqStringExpr
            "\"last_name\""))
        " "
        (ElementFieldDeclarationArg
          (CallExpr
            (IdentExpr
              "VARCHAR")
            "("
            (NumberExpr
              "45")
            ")"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "not"
              " "
              "null"))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              (IdentExpr
                " "
                "faLse")))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (DqStringExpr
            "\"email\""))
        " "
        (ElementFieldDeclarationArg
          (CallExpr
            (IdentExpr
              "VARCHAR")
            "("
            (NumberExpr
              "50")
            ")"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              (IdentExpr
                " "
                "NULL")))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (DqStringExpr
            "\"address_id\""))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "SMALLINT"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "not"
              " "
              "NULL"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (DqStringExpr
            "\"active\""))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "BOOLEAN"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "not"
              " "
              "null"))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              (IdentExpr
                " "
                "TRUE")))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (DqStringExpr
            "\"create_date\""))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "DATETIME"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "not"
              " "
              "null"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (DqStringExpr
            "\"last_update\""))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "TIMESTAMP"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              (OqStringExpr
                " "
                "`CURRENT_TIMESTAMP`")))
          "]"))
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "cities"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "integer"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "primary"
              " "
              "key"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "name"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "e"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              (DqStringExpr
                " "
                "\"hello\"")))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "country_id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "integer")))
      "\n"
      "  "
      (ElementAttributeDeclaration
        (ElementAttributeDeclarationName
          (IdentExpr
            "note"))
        ":"
        (ElementAttributeDeclarationValue
          (DqStringExpr
            " "
            "\"sasasa\"")))
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "country"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "\t"
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "integer")))
      "\n"
      "\t"
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "cities"))
        " "
        (ElementFieldDeclarationArg
          (IndexExpr
            (IdentExpr
              "string")
            "["
            "]")))
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "citites"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "\t"
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "integer")))
      "\n"
      "\t"
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "name"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "string")))
      "\n"
      "\t"
      (BlockElementDeclaration
        (ElementDeclarationType
          "indexes")
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "\t\t"
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (IdentExpr
                "id"))
            " "
            (ElementFieldDeclarationArg
              (IdentExpr
                "name")))
          "\n"
          "\t"
          "}"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

/* trailing_comments */

#[test]
fn comprehensive_trailing_comments() {
  let input = r#"Table bookings {
  id integer
  country varchar
  booking_date date
  created_at timestamp

  indexes {
      (id, country) [pk] // composite primary key
      created_at [name: 'created_at_index', note: 'Date']
      booking_date
      (country, booking_date) [unique]
      booking_date [type: hash]
      (id*2)
      (id*3,`getdate()`)
      (id*3,id)
  }
}
// End of program"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "bookings"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "integer")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "country"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "varchar")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "booking_date"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "date")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "created_at"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "timestamp")))
      "\n"
      "\n"
      "  "
      (BlockElementDeclaration
        (ElementDeclarationType
          "indexes")
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "      "
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (TupleExpr
                "("
                (IdentExpr
                  "id")
                ","
                (IdentExpr
                  " "
                  "country")
                ")"))
            " "
            (SettingList
              "["
              (SettingListItem
                (SettingListItemName
                  "pk"))
              "]"))
          " "
          "// composite primary key"
          "\n"
          "      "
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (IdentExpr
                "created_at"))
            " "
            (SettingList
              "["
              (SettingListItem
                (SettingListItemName
                  "name")
                ":"
                (SettingListItemValue
                  (SqStringExpr
                    " "
                    "'created_at_index'")))
              ","
              " "
              (SettingListItem
                (SettingListItemName
                  "note")
                ":"
                (SettingListItemValue
                  (SqStringExpr
                    " "
                    "'Date'")))
              "]"))
          "\n"
          "      "
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (IdentExpr
                "booking_date")))
          "\n"
          "      "
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (TupleExpr
                "("
                (IdentExpr
                  "country")
                ","
                (IdentExpr
                  " "
                  "booking_date")
                ")"))
            " "
            (SettingList
              "["
              (SettingListItem
                (SettingListItemName
                  "unique"))
              "]"))
          "\n"
          "      "
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (IdentExpr
                "booking_date"))
            " "
            (SettingList
              "["
              (SettingListItem
                (SettingListItemName
                  "type")
                ":"
                (SettingListItemValue
                  (IdentExpr
                    " "
                    "hash")))
              "]"))
          "\n"
          "      "
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (ParenExpr
                "("
                (InfixExpr
                  (IdentExpr
                    "id")
                  "*"
                  (NumberExpr
                    "2"))
                ")")))
          "\n"
          "      "
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (TupleExpr
                "("
                (InfixExpr
                  (IdentExpr
                    "id")
                  "*"
                  (NumberExpr
                    "3"))
                ","
                (OqStringExpr
                  "`getdate()`")
                ")")))
          "\n"
          "      "
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (TupleExpr
                "("
                (InfixExpr
                  (IdentExpr
                    "id")
                  "*"
                  (NumberExpr
                    "3"))
                ","
                (IdentExpr
                  "id")
                ")")))
          "\n"
          "  "
          "}"))
      "\n"
      "}"))
  "\n"
  "// End of program"
  "")"#;
  assert_eq!(tree, expected);
}

/* use_declarations */

#[test]
fn comprehensive_use_declarations() {
  let input = r#"use * from './index.dbml'
use {
  table my_table
  tablepartial my_tablepartial
  enum my_enum as MY_ENUM
  tableGROUP group as G
  note note as note
} from '../index.dbml'

reuse * from './index.dbml'
reuse {
  table my_table
  tablepartial my_tablepartial
  enum my_enum as MY_ENUM
  tableGROUP group as G
  note note as note
} from '../index.dbml'
"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (UseDeclaration
    "use"
    " "
    (Wildcard
      "*")
    " "
    "from"
    " "
    "'./index.dbml'")
  "\n"
  (UseDeclaration
    "use"
    " "
    (UseSpecifierList
      "{"
      "\n"
      "  "
      (UseSpecifier
        (IdentExpr
          (UseSpecifierKind
            "table"))
        (IdentExpr
          (UseSpecifierName
            " "
            "my_table")))
      "\n"
      "  "
      (UseSpecifier
        (IdentExpr
          (UseSpecifierKind
            "tablepartial"))
        (IdentExpr
          (UseSpecifierName
            " "
            "my_tablepartial")))
      "\n"
      "  "
      (UseSpecifier
        (IdentExpr
          (UseSpecifierKind
            "enum"))
        (IdentExpr
          (UseSpecifierName
            " "
            "my_enum"))
        " "
        "as"
        (IdentExpr
          (UseSpecifierAlias
            " "
            "MY_ENUM")))
      "\n"
      "  "
      (UseSpecifier
        (IdentExpr
          (UseSpecifierKind
            "tableGROUP"))
        (IdentExpr
          (UseSpecifierName
            " "
            "group"))
        " "
        "as"
        (IdentExpr
          (UseSpecifierAlias
            " "
            "G")))
      "\n"
      "  "
      (UseSpecifier
        (IdentExpr
          (UseSpecifierKind
            "note"))
        (IdentExpr
          (UseSpecifierName
            " "
            "note"))
        " "
        "as"
        (IdentExpr
          (UseSpecifierAlias
            " "
            "note")))
      "\n"
      "}")
    " "
    "from"
    " "
    "'../index.dbml'")
  "\n"
  "\n"
  (UseDeclaration
    "reuse"
    " "
    (Wildcard
      "*")
    " "
    "from"
    " "
    "'./index.dbml'")
  "\n"
  (UseDeclaration
    "reuse"
    " "
    (UseSpecifierList
      "{"
      "\n"
      "  "
      (UseSpecifier
        (IdentExpr
          (UseSpecifierKind
            "table"))
        (IdentExpr
          (UseSpecifierName
            " "
            "my_table")))
      "\n"
      "  "
      (UseSpecifier
        (IdentExpr
          (UseSpecifierKind
            "tablepartial"))
        (IdentExpr
          (UseSpecifierName
            " "
            "my_tablepartial")))
      "\n"
      "  "
      (UseSpecifier
        (IdentExpr
          (UseSpecifierKind
            "enum"))
        (IdentExpr
          (UseSpecifierName
            " "
            "my_enum"))
        " "
        "as"
        (IdentExpr
          (UseSpecifierAlias
            " "
            "MY_ENUM")))
      "\n"
      "  "
      (UseSpecifier
        (IdentExpr
          (UseSpecifierKind
            "tableGROUP"))
        (IdentExpr
          (UseSpecifierName
            " "
            "group"))
        " "
        "as"
        (IdentExpr
          (UseSpecifierAlias
            " "
            "G")))
      "\n"
      "  "
      (UseSpecifier
        (IdentExpr
          (UseSpecifierKind
            "note"))
        (IdentExpr
          (UseSpecifierName
            " "
            "note"))
        " "
        "as"
        (IdentExpr
          (UseSpecifierAlias
            " "
            "note")))
      "\n"
      "}")
    " "
    "from"
    " "
    "'../index.dbml'")
  "\n"
  "")"#;
  assert_eq!(tree, expected);
}

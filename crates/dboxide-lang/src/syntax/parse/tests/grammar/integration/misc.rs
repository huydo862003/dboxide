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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    "as"
    " "
    (BlockElementDeclarationAlias
      "U")
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
          "int")
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "user_role_in_diagram"))
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
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "diagram_id")
        " "
        (IdentExpr
          "int"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "role")
        " "
        (IdentExpr
          "int")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "note")
            ":"
            (SettingListItemValue
              " "
              "'Role = sum(user'"
              "s"
              " "
              "available"
              " "
              "permissions"
              " "
              "bit"
              " "
              "value"
              ")"
              "']"))
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "permissions"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "bit")
        " "
        (IdentExpr
          "int")
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
        (IdentExpr
          "name")
        " "
        (IdentExpr
          "varchar"))
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
        "diagrams"))
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
          "int")
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
    (BlockElementDeclarationType
      "Ref")
    ":"
    " "
    (ElementFieldDeclaration
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
            "\"user_id\"")))))
  "\n"
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Ref")
    ":"
    " "
    (ElementFieldDeclaration
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
            "\"diagram_id\"")))))
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "E"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "integer"))
      "\n"
      "}"))
  "\n"
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Note")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (NumberExpr
          "12")
        "."
        (Error))))
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (DqStringExpr
        "\"customer\""))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (DqStringExpr
          "\"customer_id\"")
        " "
        (IdentExpr
          "SMALLINT")
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
        (DqStringExpr
          "\"store_id\"")
        " "
        (IdentExpr
          "TINYINT")
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
        (DqStringExpr
          "\"first_name\"")
        " "
        (CallExpr
          (IdentExpr
            "VARCHAR")
          "("
          (NumberExpr
            "45")
          ")")
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
        (DqStringExpr
          "\"last_name\"")
        " "
        (CallExpr
          (IdentExpr
            "VARCHAR")
          "("
          (NumberExpr
            "45")
          ")")
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
              " "
              "faLse"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (DqStringExpr
          "\"email\"")
        " "
        (CallExpr
          (IdentExpr
            "VARCHAR")
          "("
          (NumberExpr
            "50")
          ")")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              " "
              "NULL"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (DqStringExpr
          "\"address_id\"")
        " "
        (IdentExpr
          "SMALLINT")
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
        (DqStringExpr
          "\"active\"")
        " "
        (IdentExpr
          "BOOLEAN")
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
              " "
              "TRUE"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (DqStringExpr
          "\"create_date\"")
        " "
        (IdentExpr
          "DATETIME")
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
        (DqStringExpr
          "\"last_update\"")
        " "
        (IdentExpr
          "TIMESTAMP")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              " "
              "`CURRENT_TIMESTAMP`"))
          "]"))
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
        "cities"))
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
          "integer")
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
        (IdentExpr
          "name")
        " "
        (IdentExpr
          "e")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              " "
              "\"hello\""))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "country_id")
        " "
        (IdentExpr
          "integer"))
      "\n"
      "  "
      (ElementAttributeDeclaration
        (ElementAttributeDeclarationName
          "note")
        ":"
        (ElementAttributeDeclarationValue
          " "
          "\"sasasa\""))
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
        "country"))
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
      "\t"
      (ElementFieldDeclaration
        (IdentExpr
          "cities")
        " "
        (IndexExpr
          (IdentExpr
            "string")
          "["
          "]"))
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
        "citites"))
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
      "\t"
      (ElementFieldDeclaration
        (IdentExpr
          "name")
        " "
        (IdentExpr
          "string"))
      "\n"
      "\t"
      (BlockElementDeclaration
        (BlockElementDeclarationType
          "indexes")
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "\t\t"
          (ElementFieldDeclaration
            (IdentExpr
              "id")
            " "
            (IdentExpr
              "name"))
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "bookings"))
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
          "integer"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "country")
        " "
        (IdentExpr
          "varchar"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "booking_date")
        " "
        (IdentExpr
          "date"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "created_at")
        " "
        (IdentExpr
          "timestamp"))
      "\n"
      "\n"
      "  "
      (BlockElementDeclaration
        (BlockElementDeclarationType
          "indexes")
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "      "
          (ElementFieldDeclaration
            (TupleExpr
              "("
              (IdentExpr
                "id")
              ","
              (IdentExpr
                " "
                "country")
              ")")
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
            (IdentExpr
              "created_at")
            " "
            (SettingList
              "["
              (SettingListItem
                (SettingListItemName
                  "name")
                ":"
                (SettingListItemValue
                  " "
                  "'created_at_index'"))
              ","
              " "
              (SettingListItem
                (SettingListItemName
                  "note")
                ":"
                (SettingListItemValue
                  " "
                  "'Date'"))
              "]"))
          "\n"
          "      "
          (ElementFieldDeclaration
            (IdentExpr
              "booking_date"))
          "\n"
          "      "
          (ElementFieldDeclaration
            (TupleExpr
              "("
              (IdentExpr
                "country")
              ","
              (IdentExpr
                " "
                "booking_date")
              ")")
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
            (IdentExpr
              "booking_date")
            " "
            (SettingList
              "["
              (SettingListItem
                (SettingListItemName
                  "type")
                ":"
                (SettingListItemValue
                  " "
                  "hash"))
              "]"))
          "\n"
          "      "
          (ElementFieldDeclaration
            (ParenExpr
              "("
              (InfixExpr
                (IdentExpr
                  "id")
                "*"
                (NumberExpr
                  "2"))
              ")"))
          "\n"
          "      "
          (ElementFieldDeclaration
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
              ")"))
          "\n"
          "      "
          (ElementFieldDeclaration
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
              ")"))
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
        "table"
        " "
        "my_table")
      "\n"
      "  "
      (UseSpecifier
        "tablepartial"
        " "
        "my_tablepartial")
      "\n"
      "  "
      (UseSpecifier
        "enum"
        " "
        "my_enum"
        " "
        "as"
        " "
        "MY_ENUM")
      "\n"
      "  "
      (UseSpecifier
        "tableGROUP"
        " "
        "group"
        " "
        "as"
        " "
        "G")
      "\n"
      "  "
      (UseSpecifier
        "note"
        " "
        "note"
        " "
        "as"
        " "
        "note")
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
        "table"
        " "
        "my_table")
      "\n"
      "  "
      (UseSpecifier
        "tablepartial"
        " "
        "my_tablepartial")
      "\n"
      "  "
      (UseSpecifier
        "enum"
        " "
        "my_enum"
        " "
        "as"
        " "
        "MY_ENUM")
      "\n"
      "  "
      (UseSpecifier
        "tableGROUP"
        " "
        "group"
        " "
        "as"
        " "
        "G")
      "\n"
      "  "
      (UseSpecifier
        "note"
        " "
        "note"
        " "
        "as"
        " "
        "note")
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

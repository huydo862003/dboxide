#[cfg(test)]
mod tests;

#[cfg(test)]
const DBML_PRELUDE: &str = r#"
use * from 'metatype'

type RefAction = "cascade" | "restrict" | "set null" | "set default" | "no action"

/* Ref [name]: table.col op table.col [settings] */
type Ref [element, use: follow refs] {
  name declaration[Ref]? [label]
  endpoints RefBinary[]

  /* binary: from cardinality to [settings] */
  type RefBinary [field] {
    from qualified_reference[Table.Column][] [arg: 0, attribute]
    cardinality string [arg: 1, attribute]
    to qualified_reference[Table.Column][] [arg: 2, attribute]
    delete RefAction? [setting]
    update RefAction? [setting]
    color string? [setting]
    inactive bool? [setting]
  }
}

/* binary ref operators: from op to -> RefBinary */
fn operator>(from: qualified_reference[Table.Column][], to: qualified_reference[Table.Column][]): obj {
  Ref.RefBinary {
    from: from
    cardinality: '>'
    to: to
  }
}
fn operator<(from: qualified_reference[Table.Column][], to: qualified_reference[Table.Column][]): obj {
  Ref.RefBinary {
    from: from
    cardinality: '<'
    to: to
  }
}
fn operator-(from: qualified_reference[Table.Column][], to: qualified_reference[Table.Column][]): obj {
  Ref.RefBinary {
    from: from
    cardinality: '-'
    to: to
  }
}
fn operator<>(from: qualified_reference[Table.Column][], to: qualified_reference[Table.Column][]): obj {
  Ref.RefBinary {
    from: from
    cardinality: '<>'
    to: to
  }
}

/* unary: [ref: > users.id] - from is implicitly the enclosing column */
type RefUnary [field] {
  cardinality string [arg: 0]
  to obj [arg: 1]
  delete RefAction? [setting]
  update RefAction? [setting]
}

/* binary: Dep: raw_orders -> stg_orders */
type DepBinary [field] {
  from obj [arg: 0]
  direction string [arg: 1]
  to obj [arg: 2]
}

/* Dep [name]: a -> b */
type Dep [element] {
  name declaration[Dep]? [label]
  note string? [setting, attribute]
  color string? [setting, attribute]
  [metadata] Map[string, obj] [attribute]
}

/* unary: [dep: -> stg_orders] */
type DepUnary [field] {
  direction string [arg: 0]
  target obj [arg: 1]
}

type Project [element] {
  name declaration[Project] [label, unique]
  database_type string? [attribute]
  note string? [attribute]
}

type Enum [element] {
  name qualified_declaration[Enum] [label, unique]
  variants Variant[]

  type Variant [field] {
    name declaration[Variant] [arg: 0, label, unique]
    note string? [setting]
  }
}

type TablePartial [element] {
  name declaration[TablePartial] [label, unique]
  headercolor string? [setting, attribute]
  note string? [setting, attribute]
}

type TableGroup [element, use: expand namespaces] {
  name declaration[TableGroup] [label, unique]
  tables TableEntry[]
  note string? [setting]
  color string? [setting]

  type TableEntry [field] {
    table reference[Table] [arg: 0]
  }
}

type Records [element] {
  columns obj[]? [arg: 0.., attribute]
  example bool? [setting]

  type Row [field] {
    values obj[] [arg: 0, csv]
  }
}

type Note [element] {
  content string [arg: 0]
}

type StickyNote [element] {
  name declaration[StickyNote] [label, unique]
  content string [arg: 0]
  color string? [setting]
}

type Metadata [element] {
  target obj [label]
  [metadata] Map[string, obj] [attribute]
}

type DiagramView [element] {
  name declaration[DiagramView] [label, unique]

  type Tables [element] {
    items string[] [arg: 0..]
  }

  type Notes [element] {
    items string[] [arg: 0..]
  }

  type TableGroups [element] {
    items string[] [arg: 0..]
  }

  type Schemas [element] {
    items string[] [arg: 0..]
  }
}

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  alias qualified_declaration[Table] [label alias]
  headercolor string? [setting, attribute]
  note string? [setting, attribute]
  dep DepUnary? [setting: dep]
  columns Column[]
  indexes Index[]
  checks Checks[]
  records Records[]
  [metadata] Map[string, obj] [attribute]

  type Column [field] {
    name declaration[Column] [arg: 0, label, unique]
    "type" string? [arg: 1, text]
    pk bool? [attribute, setting]
    unique bool? [attribute, setting]
    null bool? [setting]
    not_null bool? [setting: "not null"]
    increment bool? [setting]
    default obj? [setting]
    note string? [setting]
    check ostring? [setting]
    "primary key" bool? [attribute, setting]
    dep DepUnary? [setting: dep]

    constraints {
      nullability: () => !(null && not_null)
      increment_not_null: () => !(null && increment)
    }
  }

  type Index [element] {
    pk bool? [setting]
    unique bool? [setting]
    name string? [setting]
    "type" string? [setting]
    note string? [setting]
    columns IndexColumn[]

    type IndexColumn [field] {
      col string [arg: 0, text]
      extra string [arg: 1.., text]
      pk bool? [setting]
      unique bool? [setting]
      name string? [setting]
      "type" string? [setting]
      note string? [setting]
    }
  }

  type Checks [element] {
    name string? [setting]
    constraints CheckConstraint[]

    type CheckConstraint [field] {
      expr obj [arg: 0]
      name string? [setting]
    }
  }

  type Records [element] {
    columns obj[]? [arg: 0.., attribute]
    example bool? [setting]
    rows Row[]

    type Row [field] {
      values obj[] [arg: 0, csv]

      constraints {
        not_null: () => forall i of 0..values.length {
          !((columns[i]@not_null || columns[i]@increment || columns[i]@pk) && values[i] == null)
        }
      }
    }

    constraints {
      /* if any column is pk, every pair of rows must differ in at least one pk column */
      /* skip if no pk columns; otherwise every pair must differ in at least one pk column.
         the inner forall checks "all pk columns match" - if true, that's a violation */
      pk_unique: () => (forall c of 0..columns.length { !(columns[c]@pk) }) || forall r1 of 0..rows.length {
        forall r2 of (r1 + 1)..rows.length {
          !(forall i of 0..columns.length {
            !(columns[i]@pk) || (rows[r1]@values)[i] == (rows[r2]@values)[i]
          })
        }
      }
      /* for every unique column, no two rows share the same non-null value */
      unique_check: () => forall ui of 0..columns.length {
        !(columns[ui]@unique) || forall r1 of 0..rows.length {
          forall r2 of (r1 + 1)..rows.length {
            (rows[r1]@values)[ui] == null || (rows[r2]@values)[ui] == null || (rows[r1]@values)[ui] != (rows[r2]@values)[ui]
          }
        }
      }
    }
  }
}
"#;

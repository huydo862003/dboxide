---
_label: "Original DBML Language Specification"
_icon: icon.document
---

This document reiterates the current markup supported in DBML.

dboxide will be a meta DSL that can cover all DBML use cases.

Original DBML doc: [DBML documentation](https://dbml.dbdiagram.io/home/)

## Tokens

Whitespace is insignificant except as a token separator.

### Comments

```
// line comment
/* block comment */
```

### Identifiers

Identifiers are case-sensitive and can be bare or double-quoted.

```
ident          := [a-zA-Z_][a-zA-Z0-9_]*
               | '"' [^"\n]+ '"'
qualified_name := ident ('.' ident)*
```

```dbml
Table users { ... }                  // bare
Table "user accounts" { ... }        // quoted (spaces allowed)
public.users                         // qualified name
```

::: tip
There are no reserved keywords. Names like `Table`, `Ref`, `true`, and `null` are plain identifiers recognized by position in the grammar, not by reservation.
:::

### String Literals

```
string := "'" ( escape | [^'\n] )* "'"     // standard
        | "'''" ( escape | . )* "'''"      // multi-line
        | '`' [^`]* '`'                    // raw SQL expression
```

Escape sequences: ``\n \t \r \\ \' \" \` \0 \b \v \f \uHHHH``

::: note
Backtick strings are raw SQL expressions passed through verbatim. They are not interpreted as string values and disable static type checking at their call site.
:::

### Numeric Literals

```
integer := '-'? [0-9]+
float   := '-'? [0-9]+ '.' [0-9]+ ( [eE] [+-]? [0-9]+ )?
```

### Boolean & Null Literals

`true`, `false`, and `null` are case-insensitive.

::: info
In `records`, booleans also accept `'Y'`, `'N'`, `'T'`, `'F'`, `1`, and `0`. Null can also be represented as an empty string `''` or an empty field between commas.
:::

### Color Literals

```
color := '#' [0-9a-fA-F]+
```

Color literals are used only in `headercolor` table settings.

### Operators

`?` is a nullable qualifier that can appear on either or both sides of the base cardinality operator.

```
op       := '?'? relation '?'?
relation := '<>' | '>' | '<' | '-'
```

```dbml
Ref: posts.user_id   >  users.id        // many-to-one
Ref: posts.user_id   >? users.id        // nullable on the right
Ref: posts.user_id  ?>  users.id        // nullable on the left
Ref: posts.user_id  ?>? users.id        // both sides nullable
```

### Punctuation

| Token | Role |
| --- | --- |
| `{` `}` | Block delimiters |
| `(` `)` | Group / composite key delimiters |
| `[` `]` | Settings delimiters |
| `,` | Separator |
| `:` | Key-value separator |

## Type System

These are the value types that appear in `default:`, `records`, and settings.

### Value Types

| Type | Examples | Contexts |
| --- | --- | --- |
| String | `'hello'`, `'''multi\nline'''` | `default:`, `note:`, `records` |
| Integer | `42`, `-100` | `default:`, `records` |
| Float | `3.14`, `1.5e10` | `default:`, `records` |
| Boolean | `true`, `false`, `'Y'`, `'N'`, `'T'`, `'F'`, `1`, `0` | `default:`, `records` |
| Null | `null`, `''`, empty field | `default:`, `records` |
| Enum value | `Status.active`, `'inactive'` | `records` |
| Expression | `` `now()` ``, `` `uuid_generate_v4()` `` | `default:`, `records` |
| Color | `#ff0000` | `headercolor:` |

::: note
Timestamps are not a distinct type. They are strings in ISO 8601 or other date/time formats, interpreted according to the target column's SQL type at the database level.
:::

### Column Types

Column types are user-defined and follow SQL type syntax. They are treated as opaque strings unless SQL type awareness is added. A column type may also be an enum name, resolved during semantic analysis.

::: warning
This is one of the weaknesses dboxide aims to fix. The original DBML parser does not validate or interpret column types in any way.
:::

```
column_type := ident ( '(' arg (',' arg)* ')' )?
```

Examples: `integer`, `varchar(255)`, `decimal(10,2)`.

### Default Values

```
default_value := integer | float | string | boolean | null | expression
```

```dbml
Table products {
  price    decimal   [default: 9.99]
  in_stock boolean   [default: true]
  label    varchar   [default: 'general']
  created  timestamp [default: `now()`]
  score    integer   [default: null]
}
```

## Elements

DBML is composed of top-level and nested elements. Every element and field accepts a `[settings]` bracket. Built-in settings are listed below for each; any unrecognized key becomes custom metadata.

### Project

At most one `Project` block is allowed per schema. It has no element-level settings.

```dbml
Project name {
  database_type: 'PostgreSQL'
  Note: 'description'
}
```

**Fields**: free-form `key: 'value'` pairs. `Note` is a reserved field (see [Note](#note)).

**Sub-elements**: `Note`

### Table

Defines a database table.

```dbml
Table [schema.]name [as Alias] [settings] {
  col_name col_type [settings]
  indexes { ... }
  checks { ... }
  records { ... }
  Note: 'string'
  ~partial_name
}
```

**Settings**:

| Setting | Value |
| --- | --- |
| `headercolor` | color |
| `note` | string |

**Fields**: column definitions of the form `col_name col_type [settings]`.

**Column settings**:

| Setting | Value |
| --- | --- |
| `pk` / `primary key` | boolean |
| `not null` / `null` | boolean |
| `unique` | boolean |
| `increment` | boolean |
| `default` | value |
| `check` | expression |
| `note` | string |
| `ref` | `op [schema.]table.col` |

**Sub-elements**: `indexes`, `checks`, `records`, `Note`, `~partial_name`

### indexes

Nested inside `Table` or `TablePartial`. Defines indexes on the enclosing table.

```dbml
indexes {
  col_name [settings]
  (col1, col2) [settings]
  `expression` [settings]
}
```

**Fields**: column name, composite `(col, col)`, or backtick expression.

**Field settings**:

| Setting | Value |
| --- | --- |
| `pk` | boolean |
| `unique` | boolean |
| `type` | `btree` or `hash` |
| `name` | string |
| `note` | string |

### checks

Nested inside `Table`. Defines check constraints.

```dbml
checks {
  `expr` [settings]
}
```

**Fields**: backtick SQL expressions.

**Field settings**:

| Setting | Value |
| --- | --- |
| `name` | string |

### records

Defines sample data. It has no element-level settings. Appears top-level (with table name) or nested inside `Table`.

```dbml
records table_name(col1, col2) {   // top-level
  1, 'Alice'
}

records (col1, col2) { 1, 'Alice' }   // nested, explicit columns
records { 1, 'Alice' }                // nested, positional
```

**Fields**: comma-separated rows of [value types](#value-types). Fields have no settings.

### TablePartial

A reusable column and index template injected into tables via `~name`. Local definitions override partial ones; last injected partial wins on conflict.

::: tip
Use `TablePartial` to share common fields like `id`, `created_at`, and `updated_at` across multiple tables.
:::

```dbml
TablePartial base [settings] {
  col_name col_type [settings]
  indexes { ... }
}

Table users {
  ~base
  name varchar
}
```

**Settings**: same as `Table` settings.

**Fields**: column definitions, same structure as `Table` columns.

**Sub-elements**: `indexes`

### Ref

Declares a foreign key relationship. Three syntactic forms: inline (inside column settings), short, or long.

```dbml
user_id int [ref: op users.id]              // inline column setting

Ref [name]: table.col op table.col [settings]   // short

Ref [name] {                                    // long
  table.col op table.col [settings]
}

Ref: orders.(a, b) > merchants.(a, b)       // composite FK
```

**Settings**:

| Setting | Value |
| --- | --- |
| `delete` | `cascade`, `restrict`, `set null`, `set default`, `no action` |
| `update` | `cascade`, `restrict`, `set null`, `set default`, `no action` |
| `color` | color |
| `inactive` | boolean |

### Dep

Describes data dependencies among tables and columns for data lineage. Arrows always point from upstream to downstream. Three syntactic forms like `Ref`: inline, short, and block.

```dbml
Dep: raw_orders -> stg_orders                         // short, table-level
Dep: raw_orders.amount -> stg_orders.revenue           // short, column-level

Table fct_orders [dep: <- stg_orders] {                // inline, table-level
  revenue decimal [dep: -> report_revenue.total]       // inline, column-level
}

Dep order_staging [color: #79AD51] {                   // block, named
  raw_orders -> stg_orders
  raw_payments -> stg_orders
  raw_orders.amount -> stg_orders.revenue
  note: 'Staging transformation'
  materialized: 'table'
  owner: 'data-team'
}
```

**Settings**:

| Setting | Value |
| --- | --- |
| `color` | color |
| `note` | string |

Block form also accepts free-form `key: value` metadata like `materialized`, `query`, `owner`.

### Enum

Defines a named enumeration of string variants, optionally schema-qualified. It has no element-level settings.

```dbml
Enum [schema.]name {
  variant [settings]
  "Variant with spaces" [settings]
}
```

**Fields**: variant names, either bare identifiers or double-quoted strings.

**Field settings**:

| Setting | Value |
| --- | --- |
| `note` | string |

### TableGroup

Groups tables for visualization only. Has no semantic effect on the schema.

```dbml
TableGroup name [settings] {
  table_name
  schema.table_name
}
```

**Settings**:

| Setting | Value |
| --- | --- |
| `note` | string |
| `color` | color |

**Fields**: table names (bare or schema-qualified). Fields have no settings.

### Note

Attached inside `Project`, `Table`, or `Enum` bodies. Inline or block form.

```dbml
Note: 'single-line'
Note { '''multi-line''' }
```

### Sticky Note

A top-level named `Note` block places a canvas annotation. Distinct from the inline `Note` inside element bodies.

```dbml
Note name [settings] {
  'annotation text'
}
```

**Settings**:

| Setting | Value |
| --- | --- |
| `color` | color or `none` |

### Metadata

Attaches key-value metadata to an element by kind and name. Useful for separating metadata from schema definitions across files.

```dbml
Metadata Table users {
  owner: 'scott'
}

Metadata Column users.email {
  pii: 'true'
}
```

**Targets**: `Table name`, `Column table.col`

**Fields**: `key: value` pairs. Values are strings or colors.

Precedence (lowest to highest): inline settings, metadata blocks in imported files (later imports win), metadata blocks in the current file (later blocks win).

### DiagramView

Defines a named diagram perspective that filters which elements are visible.

```dbml
DiagramView name {
  Tables { * }                  // all tables
  Tables { users, orders }      // specific tables
  Notes { * }
  TableGroups { billing }
  Schemas { public }
}
```

**Sub-elements**: `Tables`, `Notes`, `TableGroups`, `Schemas`. Each takes `{ * }` for all, or a list of specific names.

## Module System

DBML supports splitting a schema across multiple files. All files are merged into one logical schema before analysis.

```dbml
use './path/to/other.dbml'
```

- Paths are relative to the current file.
- `use` is top-level only. Name collisions across files are errors.

::: danger
Name collisions across files are errors. If two files define a table or enum with the same qualified name, the schema is rejected at analysis time.
:::

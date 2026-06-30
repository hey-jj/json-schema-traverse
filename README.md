# json-schema-traverse

Recursively walk a JSON Schema and call a function on every subschema.

The traversal visits each object subschema, including the root, in pre-order.
It can also call a second function in post-order. References (`$ref`) are not
resolved. They are passed to the callback as plain objects. Only object
subschemas are visited. Boolean schemas, arrays, numbers, strings, and null are
never passed to a callback.

## Installation

```toml
[dependencies]
json-schema-traverse = "0.1"
```

## Usage

```rust
use serde_json::json;
use json_schema_traverse::{traverse, Options};

let schema = json!({
    "properties": {
        "foo": {"type": "string"},
        "bar": {"type": "integer"}
    }
});

let mut seen = Vec::new();
traverse(&schema, Options::default(), |ctx| {
    seen.push(ctx.json_ptr.to_owned());
});

assert_eq!(seen, vec!["", "/properties/foo", "/properties/bar"]);
```

Each callback receives a `Context` with seven fields: the subschema, its JSON
Pointer, the root schema, the parent pointer, the parent keyword, the parent
schema, and the index or property name within a multi-schema container.

Use `traverse_pre_post` to run a pre-order and a post-order callback. Set
`Options { all_keys: true }` to also descend into objects under unknown
keywords.

The default keyword tables cover the applicator keywords of draft-07. Keywords
added later, such as `prefixItems`, `dependentSchemas`, `unevaluatedItems`, and
`unevaluatedProperties`, are not in those tables, so their subschemas are
skipped unless `all_keys` is set. `prefixItems` is skipped even with `all_keys`
because it holds an array under an unknown keyword.

Object key order follows the order of the parsed document. The crate enables
the `serde_json` `preserve_order` feature so callbacks fire in source order.

Property maps are one exception. Inside a property-map keyword such as
`properties` or `$defs`, integer-like property names are visited first in
ascending numeric order, then the rest in insertion order. This matches how a
JavaScript `for..in` loop enumerates object keys. Keyword names are never
integer-like, so this only affects property maps keyed by numeric strings.

## License

Licensed under the [MIT license](LICENSE).

//! Behavior the canonical suite asserts only implicitly (A1 through A15).
//!
//! Each expected sequence comes from the traversal rules: which keywords
//! descend, how pointers are built, and the object-only guard.

mod common;

use common::{record, RecordKeyIndex};
use json_schema_traverse::Options;
use pretty_assertions::assert_eq;
use serde_json::json;

/// Visited JSON pointers in order, for compact assertions.
fn ptrs(schema: &serde_json::Value, opts: Options) -> Vec<String> {
    record(schema, opts)
        .into_iter()
        .map(|r| r.json_ptr)
        .collect()
}

/// A1: if, then, and else are single-subschema keywords. Each is descended with
/// no key index.
#[test]
fn if_then_else_descend() {
    let schema = json!({
        "if": {"properties": {"a": {"type": "string"}}},
        "then": {"properties": {"b": {"type": "string"}}},
        "else": {"properties": {"c": {"type": "string"}}}
    });
    let calls = record(&schema, Options::default());
    let got: Vec<_> = calls
        .iter()
        .map(|r| {
            (
                r.json_ptr.as_str(),
                r.parent_keyword.as_deref(),
                r.key_index.clone(),
            )
        })
        .collect();
    assert_eq!(
        got,
        vec![
            ("", None, None),
            ("/if", Some("if"), None),
            (
                "/if/properties/a",
                Some("properties"),
                Some(RecordKeyIndex::Key { key: "a".into() })
            ),
            ("/then", Some("then"), None),
            (
                "/then/properties/b",
                Some("properties"),
                Some(RecordKeyIndex::Key { key: "b".into() })
            ),
            ("/else", Some("else"), None),
            (
                "/else/properties/c",
                Some("properties"),
                Some(RecordKeyIndex::Key { key: "c".into() })
            ),
        ]
    );
}

/// A2: $defs is a props keyword. Its entries are descended with a string key.
#[test]
fn defs_is_props_keyword() {
    let schema = json!({"$defs": {"x": {"type": "string"}}});
    let calls = record(&schema, Options::default());
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1].json_ptr, "/$defs/x");
    assert_eq!(
        calls[1].key_index,
        Some(RecordKeyIndex::Key { key: "x".into() })
    );
    assert_eq!(calls[1].parent_keyword.as_deref(), Some("$defs"));
}

/// A3: property names with `~` and `/` are escaped in the pointer. The key
/// index keeps the raw name. Escape order is `~` first, then `/`.
#[test]
fn property_names_are_escaped_in_pointer() {
    let schema = json!({
        "properties": {
            "a/b": {"type": "string"},
            "c~d": {"type": "string"},
            "m~/n": {"type": "string"}
        }
    });
    let calls = record(&schema, Options::default());
    assert_eq!(
        ptrs(&schema, Options::default()),
        vec![
            "".to_owned(),
            "/properties/a~1b".to_owned(),
            "/properties/c~0d".to_owned(),
            "/properties/m~0~1n".to_owned(),
        ]
    );
    // Raw names survive in key_index.
    assert_eq!(
        calls[1].key_index,
        Some(RecordKeyIndex::Key { key: "a/b".into() })
    );
    assert_eq!(
        calls[2].key_index,
        Some(RecordKeyIndex::Key { key: "c~d".into() })
    );
    assert_eq!(
        calls[3].key_index,
        Some(RecordKeyIndex::Key { key: "m~/n".into() })
    );
    // Each escaped pointer resolves back to the right node.
    for c in &calls {
        assert_eq!(schema.pointer(&c.json_ptr), Some(&c.schema));
    }
}

/// A4: a non-object root produces no callbacks. This includes both booleans, a
/// number, a string, null, and an array.
#[test]
fn non_object_root_yields_no_calls() {
    for root in [
        json!(true),
        json!(false),
        json!(42),
        json!("s"),
        json!(null),
        json!([1, 2]),
    ] {
        assert!(record(&root, Options::default()).is_empty());
        assert!(record(&root, Options { all_keys: true }).is_empty());
    }
}

/// A5: a boolean subschema is skipped. Only the object sibling is visited.
#[test]
fn boolean_subschema_is_skipped() {
    let schema = json!({"properties": {"a": true, "b": {"type": "string"}}});
    assert_eq!(
        ptrs(&schema, Options::default()),
        vec!["".to_owned(), "/properties/b".to_owned()]
    );
}

/// A6: a $ref value is a string leaf. It is not traversed. A sibling
/// `definitions` entry is.
#[test]
fn ref_value_passes_through_unresolved() {
    let schema = json!({
        "$ref": "#/definitions/x",
        "definitions": {"x": {"type": "string"}}
    });
    assert_eq!(
        ptrs(&schema, Options::default()),
        vec!["".to_owned(), "/definitions/x".to_owned()]
    );
    // Even with allKeys the string $ref is recursed into then skipped by the
    // object guard, so no extra call appears.
    assert_eq!(
        ptrs(&schema, Options { all_keys: true }),
        vec!["".to_owned(), "/definitions/x".to_owned()]
    );
}

/// A7: visitation order follows insertion order. Two schemas with the same keys
/// in different order produce different sequences.
#[test]
fn insertion_order_drives_visit_order() {
    let a = json!({"properties": {"x": {"type": "string"}, "y": {"type": "number"}}});
    let b = json!({"properties": {"y": {"type": "number"}, "x": {"type": "string"}}});
    assert_eq!(
        ptrs(&a, Options::default()),
        vec![
            "".to_owned(),
            "/properties/x".to_owned(),
            "/properties/y".to_owned()
        ]
    );
    assert_eq!(
        ptrs(&b, Options::default()),
        vec![
            "".to_owned(),
            "/properties/y".to_owned(),
            "/properties/x".to_owned()
        ]
    );
}

/// A8: all 18 skip keywords set to object values are not descended, even with
/// allKeys. Only the root is visited.
#[test]
fn all_skip_keywords_not_descended() {
    let schema = json!({
        "default": {"x": 1},
        "enum": {"x": 1},
        "const": {"x": 1},
        "required": {"x": 1},
        "maximum": {"x": 1},
        "minimum": {"x": 1},
        "exclusiveMaximum": {"x": 1},
        "exclusiveMinimum": {"x": 1},
        "multipleOf": {"x": 1},
        "maxLength": {"x": 1},
        "minLength": {"x": 1},
        "pattern": {"x": 1},
        "format": {"x": 1},
        "maxItems": {"x": 1},
        "minItems": {"x": 1},
        "uniqueItems": {"x": 1},
        "maxProperties": {"x": 1},
        "minProperties": {"x": 1}
    });
    assert_eq!(
        ptrs(&schema, Options { all_keys: true }),
        vec!["".to_owned()]
    );
}

/// A9: an array keyword whose value is not an array is not descended. Without
/// allKeys `allOf` as an object is ignored.
#[test]
fn non_array_array_keyword_is_ignored() {
    let schema = json!({"allOf": {"not": {"type": "string"}}});
    assert_eq!(ptrs(&schema, Options::default()), vec!["".to_owned()]);
    // With allKeys, allOf is not a skip keyword, so the object is descended once
    // as a single value, and its `not` child is reached.
    assert_eq!(
        ptrs(&schema, Options { all_keys: true }),
        vec!["".to_owned(), "/allOf".to_owned(), "/allOf/not".to_owned()]
    );
}

/// A10: empty containers yield no children and do not panic.
#[test]
fn empty_containers_have_no_children() {
    let schema = json!({"properties": {}, "allOf": [], "items": []});
    assert_eq!(ptrs(&schema, Options::default()), vec!["".to_owned()]);
}

/// A11: integer-like property names visit numeric-first, then string names in
/// insertion order. This matches a JavaScript `for..in` walk. The leading zero
/// in `"01"` keeps it out of the integer group.
#[test]
fn integer_like_property_names_visit_numeric_first() {
    let schema = json!({"properties": {
        "foo": {"type": "string"},
        "2":   {"type": "a"},
        "1":   {"type": "b"},
        "bar": {"type": "c"},
        "10":  {"type": "d"},
        "01":  {"type": "e"}
    }});
    assert_eq!(
        ptrs(&schema, Options::default()),
        vec![
            "".to_owned(),
            "/properties/1".to_owned(),
            "/properties/2".to_owned(),
            "/properties/10".to_owned(),
            "/properties/foo".to_owned(),
            "/properties/bar".to_owned(),
            "/properties/01".to_owned(),
        ]
    );
}

/// A12: array indices stay tied to the raw array position. Non-object elements
/// are skipped but do not renumber the survivors.
#[test]
fn array_index_survives_skipped_elements() {
    let schema = json!({"allOf": [true, {"type": "x"}, 42, false, {"type": "y"}]});
    let calls = record(&schema, Options::default());
    let got: Vec<_> = calls
        .iter()
        .map(|r| (r.json_ptr.as_str(), r.key_index.clone()))
        .collect();
    assert_eq!(
        got,
        vec![
            ("", None),
            ("/allOf/1", Some(RecordKeyIndex::Index { index: 1 })),
            ("/allOf/4", Some(RecordKeyIndex::Index { index: 4 })),
        ]
    );
}

/// A13: an empty property name yields a trailing-slash pointer that still
/// resolves, and the key index holds the empty string.
#[test]
fn empty_property_name_yields_trailing_slash() {
    let schema = json!({"properties": {"": {"type": "string"}}});
    let calls = record(&schema, Options::default());
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1].json_ptr, "/properties/");
    assert_eq!(
        calls[1].key_index,
        Some(RecordKeyIndex::Key { key: String::new() })
    );
    assert_eq!(
        schema.pointer("/properties/"),
        Some(&json!({"type": "string"}))
    );
}

/// A14: unicode property names pass through unescaped. RFC 6901 escaping only
/// touches `~` and `/`, so multibyte names appear verbatim and still resolve.
#[test]
fn unicode_property_names_pass_through() {
    let schema = json!({"properties": {"héllo→wörld": {"type": "string"}}});
    let calls = record(&schema, Options::default());
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1].json_ptr, "/properties/héllo→wörld");
    assert_eq!(
        calls[1].key_index,
        Some(RecordKeyIndex::Key {
            key: "héllo→wörld".to_owned()
        })
    );
    assert_eq!(
        schema.pointer("/properties/héllo→wörld"),
        Some(&json!({"type": "string"}))
    );
}

/// A15: a props keyword whose value is null is not descended. The sibling
/// object keyword still is.
#[test]
fn props_keyword_with_null_value_is_skipped() {
    let schema = json!({"definitions": null, "not": {"type": "x"}});
    assert_eq!(
        ptrs(&schema, Options::default()),
        vec!["".to_owned(), "/not".to_owned()]
    );
}

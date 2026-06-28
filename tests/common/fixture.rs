//! The master fixture schema and its golden call sequence.
//!
//! `subschema`, `expected_calls`, and `expected_calls_child` mirror the
//! generators that build the canonical test schema. Building the schema and the
//! expected records from the same helpers proves the expected sequence is not
//! hand fudged. The big schema exercises every keyword class: single-subschema
//! keywords, array keywords (including a tuple `items` inside `allOf[2]`), and
//! object-map keywords, plus a skipped `required` array.
//!
//! `foo` and `bar` are the actual property names in the fixture, so the
//! placeholder-name lint is allowed here.
#![allow(clippy::disallowed_names)]

use super::{Record, RecordKeyIndex};
use serde_json::{json, Value};

/// Build the uniform leaf subschema used under each keyword.
///
/// It has empty `properties`, `additionalProperties: false`,
/// `additionalItems: false`, an `anyOf` of two format leaves, and two named
/// property schemas keyed by the keyword.
pub fn subschema(keyword: &str) -> Value {
    json!({
        "properties": {
            format!("foo_{keyword}"): {"title": "foo"},
            format!("bar_{keyword}"): {"title": "bar"}
        },
        "additionalProperties": false,
        "additionalItems": false,
        "anyOf": [
            {"format": "email"},
            {"format": "hostname"}
        ]
    })
}

/// Build the master schema.
pub fn schema() -> Value {
    json!({
        "additionalItems": subschema("additionalItems"),
        "items": subschema("items"),
        "contains": subschema("contains"),
        "additionalProperties": subschema("additionalProperties"),
        "propertyNames": subschema("propertyNames"),
        "not": subschema("not"),
        "allOf": [
            subschema("allOf_0"),
            subschema("allOf_1"),
            {
                "items": [
                    subschema("items_0"),
                    subschema("items_1")
                ]
            }
        ],
        "anyOf": [
            subschema("anyOf_0"),
            subschema("anyOf_1")
        ],
        "oneOf": [
            subschema("oneOf_0"),
            subschema("oneOf_1")
        ],
        "definitions": {
            "foo": subschema("definitions_foo"),
            "bar": subschema("definitions_bar")
        },
        "properties": {
            "foo": subschema("properties_foo"),
            "bar": subschema("properties_bar")
        },
        "patternProperties": {
            "foo": subschema("patternProperties_foo"),
            "bar": subschema("patternProperties_bar")
        },
        "dependencies": {
            "foo": subschema("dependencies_foo"),
            "bar": subschema("dependencies_bar")
        },
        "required": ["foo", "bar"]
    })
}

fn rec(
    schema: Value,
    json_ptr: &str,
    parent_json_ptr: Option<&str>,
    parent_keyword: Option<&str>,
    key_index: Option<RecordKeyIndex>,
) -> Record {
    Record {
        tag: None,
        schema,
        json_ptr: json_ptr.to_owned(),
        parent_json_ptr: parent_json_ptr.map(str::to_owned),
        parent_keyword: parent_keyword.map(str::to_owned),
        key_index,
    }
}

fn at<'a>(root: &'a Value, ptr: &str) -> &'a Value {
    root.pointer(ptr).expect("pointer resolves in fixture")
}

fn key(name: &str) -> Option<RecordKeyIndex> {
    Some(RecordKeyIndex::Key {
        key: name.to_owned(),
    })
}

fn index(i: usize) -> Option<RecordKeyIndex> {
    Some(RecordKeyIndex::Index { index: i })
}

/// Records for a single-subschema keyword and its descendants.
fn expected_calls(root: &Value, keyword: &str) -> Vec<Record> {
    let base = format!("/{keyword}");
    let foo = format!("foo_{keyword}");
    let bar = format!("bar_{keyword}");
    let foo_ptr = format!("{base}/properties/{foo}");
    let bar_ptr = format!("{base}/properties/{bar}");
    let any0 = format!("{base}/anyOf/0");
    let any1 = format!("{base}/anyOf/1");
    vec![
        rec(
            at(root, &base).clone(),
            &base,
            Some(""),
            Some(keyword),
            None,
        ),
        rec(
            at(root, &foo_ptr).clone(),
            &foo_ptr,
            Some(&base),
            Some("properties"),
            key(&foo),
        ),
        rec(
            at(root, &bar_ptr).clone(),
            &bar_ptr,
            Some(&base),
            Some("properties"),
            key(&bar),
        ),
        rec(
            at(root, &any0).clone(),
            &any0,
            Some(&base),
            Some("anyOf"),
            index(0),
        ),
        rec(
            at(root, &any1).clone(),
            &any1,
            Some(&base),
            Some("anyOf"),
            index(1),
        ),
    ]
}

/// Records for one array-keyword child and its descendants.
fn expected_calls_child(root: &Value, keyword: &str, i: usize) -> Vec<Record> {
    let base = format!("/{keyword}/{i}");
    let foo = format!("foo_{keyword}_{i}");
    let bar = format!("bar_{keyword}_{i}");
    let foo_ptr = format!("{base}/properties/{foo}");
    let bar_ptr = format!("{base}/properties/{bar}");
    let any0 = format!("{base}/anyOf/0");
    let any1 = format!("{base}/anyOf/1");
    vec![
        rec(
            at(root, &base).clone(),
            &base,
            Some(""),
            Some(keyword),
            index(i),
        ),
        rec(
            at(root, &foo_ptr).clone(),
            &foo_ptr,
            Some(&base),
            Some("properties"),
            key(&foo),
        ),
        rec(
            at(root, &bar_ptr).clone(),
            &bar_ptr,
            Some(&base),
            Some("properties"),
            key(&bar),
        ),
        rec(
            at(root, &any0).clone(),
            &any0,
            Some(&base),
            Some("anyOf"),
            index(0),
        ),
        rec(
            at(root, &any1).clone(),
            &any1,
            Some(&base),
            Some("anyOf"),
            index(1),
        ),
    ]
}

/// Build the golden ordered call sequence for [`schema`].
pub fn expected_calls_all() -> Vec<Record> {
    let root = schema();
    let r = &root;
    let mut out = Vec::new();

    out.push(rec(root.clone(), "", None, None, None));
    out.extend(expected_calls(r, "additionalItems"));
    out.extend(expected_calls(r, "items"));
    out.extend(expected_calls(r, "contains"));
    out.extend(expected_calls(r, "additionalProperties"));
    out.extend(expected_calls(r, "propertyNames"));
    out.extend(expected_calls(r, "not"));
    out.extend(expected_calls_child(r, "allOf", 0));
    out.extend(expected_calls_child(r, "allOf", 1));

    // allOf[2] holds a tuple `items` array. Each tuple element is a full
    // subschema. The pointers nest under /allOf/2/items/<i>.
    let tuple = tuple_records(r);
    out.extend(tuple);

    out.extend(expected_calls_child(r, "anyOf", 0));
    out.extend(expected_calls_child(r, "anyOf", 1));
    out.extend(expected_calls_child(r, "oneOf", 0));
    out.extend(expected_calls_child(r, "oneOf", 1));
    out.extend(props_records(r, "definitions"));
    out.extend(props_records(r, "properties"));
    out.extend(props_records(r, "patternProperties"));
    out.extend(props_records(r, "dependencies"));
    out
}

/// The explicit allOf[2] tuple branch.
fn tuple_records(root: &Value) -> Vec<Record> {
    let mut out = Vec::new();
    out.push(rec(
        at(root, "/allOf/2").clone(),
        "/allOf/2",
        Some(""),
        Some("allOf"),
        index(2),
    ));
    for i in 0..2 {
        let base = format!("/allOf/2/items/{i}");
        let foo = format!("foo_items_{i}");
        let bar = format!("bar_items_{i}");
        let foo_ptr = format!("{base}/properties/{foo}");
        let bar_ptr = format!("{base}/properties/{bar}");
        let any0 = format!("{base}/anyOf/0");
        let any1 = format!("{base}/anyOf/1");
        out.push(rec(
            at(root, &base).clone(),
            &base,
            Some("/allOf/2"),
            Some("items"),
            index(i),
        ));
        out.push(rec(
            at(root, &foo_ptr).clone(),
            &foo_ptr,
            Some(&base),
            Some("properties"),
            key(&foo),
        ));
        out.push(rec(
            at(root, &bar_ptr).clone(),
            &bar_ptr,
            Some(&base),
            Some("properties"),
            key(&bar),
        ));
        out.push(rec(
            at(root, &any0).clone(),
            &any0,
            Some(&base),
            Some("anyOf"),
            index(0),
        ));
        out.push(rec(
            at(root, &any1).clone(),
            &any1,
            Some(&base),
            Some("anyOf"),
            index(1),
        ));
    }
    out
}

/// Records for a props keyword with `foo` and `bar` entries.
fn props_records(root: &Value, keyword: &str) -> Vec<Record> {
    let mut out = Vec::new();
    for name in ["foo", "bar"] {
        let base = format!("/{keyword}/{name}");
        let label = format!("{keyword}_{name}");
        let foo = format!("foo_{label}");
        let bar = format!("bar_{label}");
        let foo_ptr = format!("{base}/properties/{foo}");
        let bar_ptr = format!("{base}/properties/{bar}");
        let any0 = format!("{base}/anyOf/0");
        let any1 = format!("{base}/anyOf/1");
        out.push(rec(
            at(root, &base).clone(),
            &base,
            Some(""),
            Some(keyword),
            key(name),
        ));
        out.push(rec(
            at(root, &foo_ptr).clone(),
            &foo_ptr,
            Some(&base),
            Some("properties"),
            key(&foo),
        ));
        out.push(rec(
            at(root, &bar_ptr).clone(),
            &bar_ptr,
            Some(&base),
            Some("properties"),
            key(&bar),
        ));
        out.push(rec(
            at(root, &any0).clone(),
            &any0,
            Some(&base),
            Some("anyOf"),
            index(0),
        ));
        out.push(rec(
            at(root, &any1).clone(),
            &any1,
            Some(&base),
            Some("anyOf"),
            index(1),
        ));
    }
    out
}

/// Load the verbatim fixture schema committed alongside the tests.
pub fn schema_from_file() -> Value {
    let raw = include_str!("../fixtures/schema.json");
    serde_json::from_str(raw).expect("schema.json parses")
}

/// Load the verbatim golden records committed alongside the tests.
pub fn expected_from_file() -> Vec<Record> {
    let raw = include_str!("../fixtures/schema.expected.json");
    serde_json::from_str(raw).expect("schema.expected.json parses")
}

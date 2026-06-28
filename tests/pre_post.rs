//! pre and post ordering (T8 through T10).

mod common;

use common::{record_post, record_pre, record_pre_post, Record, RecordKeyIndex};
use json_schema_traverse::Options;
use pretty_assertions::assert_eq;
use serde_json::{json, Value};

fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "number"}
        }
    })
}

fn tagged(tag: &str, schema: Value, ptr: &str, key_index: Option<RecordKeyIndex>) -> Record {
    let is_root = ptr.is_empty();
    Record {
        tag: Some(tag.to_owned()),
        schema,
        json_ptr: ptr.to_owned(),
        parent_json_ptr: if is_root { None } else { Some(String::new()) },
        parent_keyword: if is_root {
            None
        } else {
            Some("properties".to_owned())
        },
        key_index,
    }
}

fn name_key() -> Option<RecordKeyIndex> {
    Some(RecordKeyIndex::Key {
        key: "name".to_owned(),
    })
}

fn age_key() -> Option<RecordKeyIndex> {
    Some(RecordKeyIndex::Key {
        key: "age".to_owned(),
    })
}

/// T8: pre-order. Parent before children.
#[test]
fn pre_order() {
    let s = schema();
    let calls = record_pre(&s, &Options::default());
    let expected = vec![
        tagged("pre", s.clone(), "", None),
        tagged(
            "pre",
            s["properties"]["name"].clone(),
            "/properties/name",
            name_key(),
        ),
        tagged(
            "pre",
            s["properties"]["age"].clone(),
            "/properties/age",
            age_key(),
        ),
    ];
    assert_eq!(calls, expected);
}

/// T9: post-order. Children before parent, root last.
#[test]
fn post_order() {
    let s = schema();
    let calls = record_post(&s, &Options::default());
    let expected = vec![
        tagged(
            "post",
            s["properties"]["name"].clone(),
            "/properties/name",
            name_key(),
        ),
        tagged(
            "post",
            s["properties"]["age"].clone(),
            "/properties/age",
            age_key(),
        ),
        tagged("post", s.clone(), "", None),
    ];
    assert_eq!(calls, expected);
}

/// T10: pre and post interleaved.
#[test]
fn pre_and_post_interleaved() {
    let s = schema();
    let calls = record_pre_post(&s, &Options::default());
    let expected = vec![
        tagged("pre", s.clone(), "", None),
        tagged(
            "pre",
            s["properties"]["name"].clone(),
            "/properties/name",
            name_key(),
        ),
        tagged(
            "post",
            s["properties"]["name"].clone(),
            "/properties/name",
            name_key(),
        ),
        tagged(
            "pre",
            s["properties"]["age"].clone(),
            "/properties/age",
            age_key(),
        ),
        tagged(
            "post",
            s["properties"]["age"].clone(),
            "/properties/age",
            age_key(),
        ),
        tagged("post", s.clone(), "", None),
    ];
    assert_eq!(calls, expected);
}

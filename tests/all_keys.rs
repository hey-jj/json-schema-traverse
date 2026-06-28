//! allKeys option behavior (T4 through T7).

mod common;

use common::{record, Record, RecordKeyIndex};
use json_schema_traverse::Options;
use pretty_assertions::assert_eq;
use serde_json::{json, Value};

fn root_record(schema: &Value) -> Record {
    Record {
        tag: None,
        schema: schema.clone(),
        json_ptr: String::new(),
        parent_json_ptr: None,
        parent_keyword: None,
        key_index: None,
    }
}

/// T4: allKeys true descends into the unknown keyword `someObject`. The nested
/// `minimum` and `maximum` are scalar leaves, so traversal stops there.
#[test]
fn all_keys_true_descends_unknown_keyword() {
    let schema = json!({"someObject": {"minimum": 1, "maximum": 2}});
    let calls = record(&schema, Options { all_keys: true });
    let expected = vec![
        root_record(&schema),
        Record {
            tag: None,
            schema: schema["someObject"].clone(),
            json_ptr: "/someObject".to_owned(),
            parent_json_ptr: Some(String::new()),
            parent_keyword: Some("someObject".to_owned()),
            key_index: None,
        },
    ];
    assert_eq!(calls, expected);
}

/// T5: allKeys false disables unknown-keyword descent.
#[test]
fn all_keys_false_skips_unknown_keyword() {
    let schema = json!({"someObject": {"minimum": 1, "maximum": 2}});
    let calls = record(&schema, Options { all_keys: false });
    assert_eq!(calls, vec![root_record(&schema)]);
}

/// T6: the default (no allKeys) matches allKeys false.
#[test]
fn default_skips_unknown_keyword() {
    let schema = json!({"someObject": {"minimum": 1, "maximum": 2}});
    let calls = record(&schema, Options::default());
    assert_eq!(calls, vec![root_record(&schema)]);
}

/// T7: mixed edges under allKeys true. skip keywords (`const`, `enum`) are not
/// descended even though their values are object or array. `required` is an
/// array under a non-array keyword, so it is ignored. Empty `patternProperties`
/// yields no children. `dependencies: true` is not an object, so nothing
/// descends. The unknown `another` object is visited. Both `properties`
/// children are visited. The `{$data}` value of `minimum` is a leaf because
/// `minimum` is a skip keyword.
#[test]
fn skip_keywords_not_descended_under_all_keys() {
    let schema = json!({
        "const": {"foo": "bar"},
        "enum": ["a", "b"],
        "required": ["foo"],
        "another": {},
        "patternProperties": {},
        "dependencies": true,
        "properties": {
            "smaller": {"type": "number"},
            "larger": {"type": "number", "minimum": {"$data": "1/smaller"}}
        }
    });
    let calls = record(&schema, Options { all_keys: true });
    let expected = vec![
        root_record(&schema),
        Record {
            tag: None,
            schema: schema["another"].clone(),
            json_ptr: "/another".to_owned(),
            parent_json_ptr: Some(String::new()),
            parent_keyword: Some("another".to_owned()),
            key_index: None,
        },
        Record {
            tag: None,
            schema: schema["properties"]["smaller"].clone(),
            json_ptr: "/properties/smaller".to_owned(),
            parent_json_ptr: Some(String::new()),
            parent_keyword: Some("properties".to_owned()),
            key_index: Some(RecordKeyIndex::Key {
                key: "smaller".to_owned(),
            }),
        },
        Record {
            tag: None,
            schema: schema["properties"]["larger"].clone(),
            json_ptr: "/properties/larger".to_owned(),
            parent_json_ptr: Some(String::new()),
            parent_keyword: Some("properties".to_owned()),
            key_index: Some(RecordKeyIndex::Key {
                key: "larger".to_owned(),
            }),
        },
    ];
    assert_eq!(calls, expected);
}

//! Structural invariants over arbitrary schemas (P1 through P5).

mod common;

use common::{record, record_post, record_pre};
use json_schema_traverse::Options;
use proptest::prelude::*;
use serde_json::{Map, Value};

/// Generate small arbitrary JSON values, biased toward objects and known
/// keywords so traversal has something to descend into.
fn arb_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i32>().prop_map(Value::from),
        "[a-z/~]{0,4}".prop_map(Value::String),
    ];
    leaf.prop_recursive(4, 48, 6, |inner| {
        // Keyword names are not escaped when built into a pointer, matching
        // the rule that real keyword names never contain `/` or `~`. Keep
        // generated keys free of those characters so pointers resolve. Property
        // names with `/` and `~` are exercised by the escape round-trip test.
        let keys = prop_oneof![
            Just("properties".to_owned()),
            Just("items".to_owned()),
            Just("allOf".to_owned()),
            Just("not".to_owned()),
            Just("definitions".to_owned()),
            Just("required".to_owned()),
            "[a-z]{1,4}",
        ];
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::vec((keys, inner), 0..5).prop_map(|pairs| {
                let mut m = Map::new();
                for (k, v) in pairs {
                    m.insert(k, v);
                }
                Value::Object(m)
            }),
        ]
    })
}

proptest! {
    /// P1: every emitted pointer resolves to the emitted schema.
    #[test]
    fn pointer_soundness(schema in arb_value()) {
        for opts in [Options::default(), Options { all_keys: true }] {
            for r in record(&schema, &opts) {
                let at = schema.pointer(&r.json_ptr);
                prop_assert_eq!(at, Some(&r.schema), "ptr {:?}", r.json_ptr);
            }
        }
    }

    /// P2: pre and post visit the same nodes. The pre order is a valid
    /// pre-order DFS and post is the matching post-order, so the post sequence
    /// is the pre sequence reversed by node.
    #[test]
    fn pre_post_symmetry(schema in arb_value()) {
        for opts in [Options::default(), Options { all_keys: true }] {
            let pre: Vec<String> = record_pre(&schema, &opts)
                .into_iter().map(|r| r.json_ptr).collect();
            let post: Vec<String> = record_post(&schema, &opts)
                .into_iter().map(|r| r.json_ptr).collect();
            prop_assert_eq!(pre.len(), post.len());
            let mut pre_sorted = pre.clone();
            pre_sorted.sort();
            let mut post_sorted = post.clone();
            post_sorted.sort();
            prop_assert_eq!(pre_sorted, post_sorted, "same node set");
        }
    }

    /// P3: the nodes visited with all_keys false are a subset of those visited
    /// with all_keys true.
    #[test]
    fn all_keys_monotonicity(schema in arb_value()) {
        let off: std::collections::BTreeSet<String> =
            record(&schema, &Options::default()).into_iter().map(|r| r.json_ptr).collect();
        let on: std::collections::BTreeSet<String> =
            record(&schema, &Options { all_keys: true }).into_iter().map(|r| r.json_ptr).collect();
        prop_assert!(off.is_subset(&on));
    }

    /// P5: traversal never mutates the input.
    #[test]
    fn input_is_not_mutated(schema in arb_value()) {
        let before = schema.clone();
        let _ = record(&schema, &Options { all_keys: true });
        prop_assert_eq!(schema, before);
    }
}

// P4: escaping a name and unescaping it round-trips, and an escaped property
// pointer resolves. The crate keeps `escape_json_ptr` private, so this drives
// it through traversal.
proptest! {
    #[test]
    fn escape_round_trip(name in "[a-zA-Z0-9/~]{1,8}") {
        let mut props = Map::new();
        props.insert(name.clone(), Value::Object(Map::new()));
        let mut root = Map::new();
        root.insert("properties".to_owned(), Value::Object(props));
        let schema = Value::Object(root);

        let calls = record(&schema, &Options::default());
        // root plus the one property.
        prop_assert_eq!(calls.len(), 2);
        let child = &calls[1];
        // The escaped pointer resolves to the property value.
        prop_assert_eq!(schema.pointer(&child.json_ptr), Some(&child.schema));
        // Unescaping the final token recovers the raw name.
        let token = child.json_ptr.strip_prefix("/properties/").unwrap();
        let unescaped = token.replace("~1", "/").replace("~0", "~");
        prop_assert_eq!(unescaped, name);
    }
}

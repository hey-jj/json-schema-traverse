//! Master golden test over the big fixture schema (T1, T2, T3).

mod common;

use common::fixture::{expected_calls_all, expected_from_file, schema, schema_from_file};
use common::record;
use json_schema_traverse::Options;
use pretty_assertions::assert_eq;

/// The programmatic generators and the committed fixture files must agree.
/// This catches drift in either direction.
#[test]
fn generated_schema_matches_committed_file() {
    assert_eq!(schema(), schema_from_file());
}

#[test]
fn generated_expected_matches_committed_file() {
    assert_eq!(expected_calls_all(), expected_from_file());
}

/// T1: the `{cb}` options form yields the full golden sequence.
#[test]
fn traverses_all_keywords_recursively() {
    let s = schema();
    let calls = record(&s, &Options::default());
    assert_eq!(calls, expected_calls_all());
}

/// T2 and T3: the legacy call shapes are behaviorally identical. The Rust API
/// has one traversal function, so both shapes collapse to the same call. The
/// cases stay named so the inventory maps one to one.
#[test]
fn legacy_two_arg_form_matches() {
    let s = schema();
    let calls = record(&s, &Options::default());
    assert_eq!(calls, expected_calls_all());
}

#[test]
fn legacy_empty_opts_form_matches() {
    let s = schema();
    let calls = record(&s, &Options { all_keys: false });
    assert_eq!(calls, expected_calls_all());
}

/// The big run produces 112 records.
#[test]
fn record_count_is_stable() {
    let s = schema();
    assert_eq!(record(&s, &Options::default()).len(), 112);
}

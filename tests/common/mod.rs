//! Shared test harness.
//!
//! Records every callback as a serializable [`Record`] so a run can be golden
//! compared against an expected sequence. Each record carries the cloned
//! subschema value, its JSON pointer, the parent pointer, the parent keyword,
//! and the index or property name. This is a serializable form of the seven
//! callback fields, matching the call sequence the traversal must produce.
//!
//! Each integration test compiles as its own crate and uses a subset of these
//! helpers, so unused items are expected here.
#![allow(dead_code)]

pub mod fixture;

use json_schema_traverse::{traverse, traverse_pre_post, Context, KeyIndex, Options};
use serde::Deserialize;
use serde_json::Value;

/// Index or property name within a multi-schema container, in a form that
/// serializes the same way the fixtures store it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum RecordKeyIndex {
    /// Numeric array index.
    Index { index: usize },
    /// String property name.
    Key { key: String },
}

impl From<&KeyIndex> for RecordKeyIndex {
    fn from(ki: &KeyIndex) -> Self {
        match ki {
            KeyIndex::Index(i) => RecordKeyIndex::Index { index: *i },
            KeyIndex::Key(k) => RecordKeyIndex::Key { key: k.clone() },
        }
    }
}

/// One recorded callback. The optional `tag` marks pre or post for the
/// pre/post tests. Single-callback tests leave it `None`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Record {
    /// Pre/post marker. `None` for the single-callback form.
    #[serde(default)]
    pub tag: Option<String>,
    /// The visited subschema, cloned.
    pub schema: Value,
    /// JSON Pointer from the root to the subschema.
    pub json_ptr: String,
    /// JSON Pointer to the parent schema, or `None` for the root.
    pub parent_json_ptr: Option<String>,
    /// Keyword that contains the subschema, or `None` for the root.
    pub parent_keyword: Option<String>,
    /// Index or property name, or `None` when neither applies.
    pub key_index: Option<RecordKeyIndex>,
}

impl Record {
    fn from_ctx(tag: Option<&str>, ctx: &Context) -> Self {
        Record {
            tag: tag.map(str::to_owned),
            schema: ctx.schema.clone(),
            json_ptr: ctx.json_ptr.to_owned(),
            parent_json_ptr: ctx.parent_json_ptr.map(str::to_owned),
            parent_keyword: ctx.parent_keyword.map(str::to_owned),
            key_index: ctx.key_index.as_ref().map(RecordKeyIndex::from),
        }
    }
}

/// Assert the identity contract the canonical suite checks by reference: the
/// value handed to the callback is exactly the node its pointer addresses, and
/// the same for the parent.
fn check_identity(ctx: &Context) {
    let at = ctx
        .root_schema
        .pointer(ctx.json_ptr)
        .expect("json_ptr must resolve in root");
    assert_eq!(
        at, ctx.schema,
        "schema at {:?} does not match the addressed node",
        ctx.json_ptr
    );
    if let Some(pptr) = ctx.parent_json_ptr {
        let parent_at = ctx
            .root_schema
            .pointer(pptr)
            .expect("parent_json_ptr must resolve in root");
        let parent = ctx
            .parent_schema
            .expect("parent schema present with pointer");
        assert_eq!(
            parent_at, parent,
            "parent at {pptr:?} does not match the addressed node",
        );
    }
}

/// Record a single-callback (pre-only) run.
pub fn record(schema: &Value, opts: Options) -> Vec<Record> {
    let mut calls = Vec::new();
    traverse(schema, opts, |ctx| {
        check_identity(ctx);
        calls.push(Record::from_ctx(None, ctx));
    });
    calls
}

/// Record a pre-only run, tagging each record `"pre"`.
pub fn record_pre(schema: &Value, opts: Options) -> Vec<Record> {
    let mut calls = Vec::new();
    traverse_pre_post(
        schema,
        opts,
        |ctx| {
            check_identity(ctx);
            calls.push(Record::from_ctx(Some("pre"), ctx));
        },
        |_| {},
    );
    calls
}

/// Record a post-only run, tagging each record `"post"`.
pub fn record_post(schema: &Value, opts: Options) -> Vec<Record> {
    let mut calls = Vec::new();
    traverse_pre_post(
        schema,
        opts,
        |_| {},
        |ctx| {
            check_identity(ctx);
            calls.push(Record::from_ctx(Some("post"), ctx));
        },
    );
    calls
}

/// Record a combined pre and post run into a single tagged sequence.
pub fn record_pre_post(schema: &Value, opts: Options) -> Vec<Record> {
    use std::cell::RefCell;
    let calls = RefCell::new(Vec::new());
    traverse_pre_post(
        schema,
        opts,
        |ctx| {
            check_identity(ctx);
            calls.borrow_mut().push(Record::from_ctx(Some("pre"), ctx));
        },
        |ctx| {
            check_identity(ctx);
            calls.borrow_mut().push(Record::from_ctx(Some("post"), ctx));
        },
    );
    calls.into_inner()
}

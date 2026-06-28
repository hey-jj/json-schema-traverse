//! Recursively walk a JSON Schema, invoking callbacks on every subschema.
//!
//! The traversal visits each subschema object in a JSON Schema document,
//! including the root, in pre-order. It can also call a second callback in
//! post-order. References (`$ref`) are not resolved. They are passed to the
//! callback as plain objects.
//!
//! Only object subschemas are visited. Boolean schemas (`true` / `false`),
//! arrays, numbers, strings, and null are never passed to a callback. This
//! matches the draft-06+ rule that boolean schemas carry no nested schemas.
//!
//! # Example
//!
//! ```
//! use serde_json::json;
//! use json_schema_traverse::{traverse, Options};
//!
//! let schema = json!({
//!     "properties": {
//!         "foo": {"type": "string"},
//!         "bar": {"type": "integer"}
//!     }
//! });
//!
//! let mut seen = Vec::new();
//! traverse(&schema, Options::default(), |ctx| {
//!     seen.push(ctx.json_ptr.to_owned());
//! });
//!
//! // Root, then each property value, in insertion order.
//! assert_eq!(seen, vec!["", "/properties/foo", "/properties/bar"]);
//! ```
//!
//! Object key order follows the order of the parsed document, so build
//! `serde_json::Value` with the `preserve_order` feature when call order
//! matters. This crate enables that feature.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod keywords;

use serde_json::Value;

/// Location of a subschema inside a multi-schema container.
///
/// Array keywords yield a numeric index. Object-map keywords yield a property
/// name. Single-subschema keywords and the root yield neither, represented by
/// the absence of a value (`None`) in [`Context::key_index`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyIndex {
    /// Array index for an array keyword such as `allOf` or tuple `items`.
    Index(usize),
    /// Property name for an object-map keyword such as `properties`.
    Key(String),
}

/// The seven values handed to each callback for a visited subschema.
///
/// Every field borrows. The `'a` lifetime ties the borrowed values to the data
/// that outlives a single callback: the root schema and the pointer strings
/// built during the walk. A callback that needs to keep a pointer or a keyword
/// past the call clones it with `.to_owned()`. The fields mirror the callback
/// arguments described in the crate docs.
#[derive(Debug, Clone, PartialEq)]
pub struct Context<'a> {
    /// The current subschema object.
    pub schema: &'a Value,
    /// JSON Pointer (RFC 6901) from the root to this subschema. The root is the
    /// empty string.
    pub json_ptr: &'a str,
    /// The root schema passed to [`traverse`].
    pub root_schema: &'a Value,
    /// JSON Pointer to the parent schema object. `None` for the root.
    pub parent_json_ptr: Option<&'a str>,
    /// Keyword name that contains this subschema, such as `properties` or
    /// `anyOf`. `None` for the root.
    pub parent_keyword: Option<&'a str>,
    /// The parent schema object. For a property value this is the schema that
    /// holds the `properties` keyword, not the `properties` map itself. `None`
    /// for the root.
    pub parent_schema: Option<&'a Value>,
    /// Index or property name within a multi-schema container. `None` for
    /// single-subschema keywords and the root.
    pub key_index: Option<KeyIndex>,
}

/// Traversal options.
#[derive(Debug, Default, Clone, Copy)]
pub struct Options {
    /// Descend into objects nested under unknown keywords.
    ///
    /// With this off (the default) the traversal only descends into the known
    /// keyword sets in [`keywords`]. With it on the traversal also descends
    /// into any object whose keyword is not in [`keywords::SKIP_KEYWORDS`].
    pub all_keys: bool,
}

/// Walk `schema` and call `cb` on every subschema in pre-order.
///
/// This is the single-callback form. The callback fires for the root and for
/// each nested subschema object, before any of its own children.
///
/// # Example
///
/// ```
/// use serde_json::json;
/// use json_schema_traverse::{traverse, Options};
///
/// let schema = json!({"not": {"type": "string"}});
/// let mut count = 0;
/// traverse(&schema, Options::default(), |_ctx| count += 1);
/// assert_eq!(count, 2); // root and the `not` subschema
/// ```
pub fn traverse<F>(schema: &Value, opts: Options, mut cb: F)
where
    F: FnMut(&Context),
{
    let mut noop = |_: &Context| {};
    walk(
        opts, &mut cb, &mut noop, schema, "", schema, None, None, None, None,
    );
}

/// Walk `schema` calling `pre` in pre-order and `post` in post-order.
///
/// `pre` fires for a node before its children. `post` fires for a node after
/// all of its children. For a node `n` with children `c1` and `c2` the order
/// is `pre(n)`, `pre(c1)`, `post(c1)`, `pre(c2)`, `post(c2)`, `post(n)`.
///
/// # Example
///
/// ```
/// use std::cell::RefCell;
/// use serde_json::json;
/// use json_schema_traverse::{traverse_pre_post, Options};
///
/// let schema = json!({"properties": {"a": {"type": "string"}}});
/// let order = RefCell::new(Vec::new());
/// traverse_pre_post(
///     &schema,
///     Options::default(),
///     |ctx| order.borrow_mut().push(format!("pre {}", ctx.json_ptr)),
///     |ctx| order.borrow_mut().push(format!("post {}", ctx.json_ptr)),
/// );
/// assert_eq!(
///     order.into_inner(),
///     vec!["pre ", "pre /properties/a", "post /properties/a", "post "]
/// );
/// ```
pub fn traverse_pre_post<P, Q>(schema: &Value, opts: Options, mut pre: P, mut post: Q)
where
    P: FnMut(&Context),
    Q: FnMut(&Context),
{
    walk(
        opts, &mut pre, &mut post, schema, "", schema, None, None, None, None,
    );
}

/// Recursive worker. Visits `schema` only when it is a plain object.
#[allow(clippy::too_many_arguments)]
fn walk<'a>(
    opts: Options,
    pre: &mut dyn FnMut(&Context),
    post: &mut dyn FnMut(&Context),
    schema: &'a Value,
    json_ptr: &str,
    root_schema: &'a Value,
    parent_json_ptr: Option<&str>,
    parent_keyword: Option<&str>,
    parent_schema: Option<&'a Value>,
    key_index: Option<KeyIndex>,
) {
    let Value::Object(map) = schema else {
        return;
    };

    let ctx = Context {
        schema,
        json_ptr,
        root_schema,
        parent_json_ptr,
        parent_keyword,
        parent_schema,
        key_index,
    };
    pre(&ctx);

    for (key, sch) in map {
        match sch {
            Value::Array(items) if keywords::is_array_keyword(key) => {
                for (i, item) in items.iter().enumerate() {
                    let child_ptr = format!("{json_ptr}/{key}/{i}");
                    walk(
                        opts,
                        pre,
                        post,
                        item,
                        &child_ptr,
                        root_schema,
                        Some(json_ptr),
                        Some(key),
                        Some(schema),
                        Some(KeyIndex::Index(i)),
                    );
                }
            }
            // An array under a non-array keyword is ignored entirely. The guard
            // matches arrays before the props and single-value branches, so an
            // array never reaches them even if its keyword is in another table.
            Value::Array(_) => {}
            _ if keywords::is_props_keyword(key) => {
                if let Value::Object(props) = sch {
                    for (prop, prop_sch) in props {
                        let child_ptr = format!("{json_ptr}/{key}/{}", escape_json_ptr(prop));
                        walk(
                            opts,
                            pre,
                            post,
                            prop_sch,
                            &child_ptr,
                            root_schema,
                            Some(json_ptr),
                            Some(key),
                            Some(schema),
                            Some(KeyIndex::Key(prop.clone())),
                        );
                    }
                }
            }
            _ if keywords::is_keyword(key)
                || (opts.all_keys && !keywords::is_skip_keyword(key)) =>
            {
                let child_ptr = format!("{json_ptr}/{key}");
                walk(
                    opts,
                    pre,
                    post,
                    sch,
                    &child_ptr,
                    root_schema,
                    Some(json_ptr),
                    Some(key),
                    Some(schema),
                    None,
                );
            }
            _ => {}
        }
    }

    post(&ctx);
}

/// Escape a property name for use as a JSON Pointer reference token (RFC 6901).
///
/// Replaces `~` with `~0`, then `/` with `~1`. Order matters so the `~1` from
/// the second pass is not re-escaped by the first.
fn escape_json_ptr(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn context_clones_equal_to_source() {
        let root = json!({"not": {"type": "string"}});
        let child = &root["not"];
        let ctx = Context {
            schema: child,
            json_ptr: "/not",
            root_schema: &root,
            parent_json_ptr: Some(""),
            parent_keyword: Some("not"),
            parent_schema: Some(&root),
            key_index: None,
        };
        let copy = ctx.clone();
        assert_eq!(ctx, copy);
    }
}

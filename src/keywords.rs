//! Keyword tables that drive traversal.
//!
//! Each table is a set of keyword names. The traversal consults them to decide
//! how to descend into a value. The tables are public so callers can check
//! membership or compare against their own keyword logic.

/// Keywords whose value is a single subschema. The traversal descends directly
/// into the value.
///
/// `items` also appears in [`ARRAY_KEYWORDS`]. When `items` holds an array the
/// array rule wins, otherwise the value is treated as a single subschema.
pub const KEYWORDS: &[&str] = &[
    "additionalItems",
    "items",
    "contains",
    "additionalProperties",
    "propertyNames",
    "not",
    "if",
    "then",
    "else",
];

/// Keywords whose value, when it is an array, is an array of subschemas. The
/// traversal descends into each element by numeric index.
pub const ARRAY_KEYWORDS: &[&str] = &["items", "allOf", "anyOf", "oneOf"];

/// Keywords whose value is an object map from name to subschema. The traversal
/// descends into each value by property name, escaping the name in the pointer.
pub const PROPS_KEYWORDS: &[&str] = &[
    "$defs",
    "definitions",
    "properties",
    "patternProperties",
    "dependencies",
];

/// Value-only keywords. The traversal never descends into these, even with
/// [`Options::all_keys`](crate::Options::all_keys) set.
pub const SKIP_KEYWORDS: &[&str] = &[
    "default",
    "enum",
    "const",
    "required",
    "maximum",
    "minimum",
    "exclusiveMaximum",
    "exclusiveMinimum",
    "multipleOf",
    "maxLength",
    "minLength",
    "pattern",
    "format",
    "maxItems",
    "minItems",
    "uniqueItems",
    "maxProperties",
    "minProperties",
];

/// Returns true when `key` is a single-subschema keyword. See [`KEYWORDS`].
pub fn is_keyword(key: &str) -> bool {
    KEYWORDS.contains(&key)
}

/// Returns true when `key` is an array-of-subschemas keyword. See
/// [`ARRAY_KEYWORDS`].
pub fn is_array_keyword(key: &str) -> bool {
    ARRAY_KEYWORDS.contains(&key)
}

/// Returns true when `key` is an object-map-of-subschemas keyword. See
/// [`PROPS_KEYWORDS`].
pub fn is_props_keyword(key: &str) -> bool {
    PROPS_KEYWORDS.contains(&key)
}

/// Returns true when `key` is a value-only keyword that is never descended. See
/// [`SKIP_KEYWORDS`].
pub fn is_skip_keyword(key: &str) -> bool {
    SKIP_KEYWORDS.contains(&key)
}

//! The public keyword tables hold exactly the expected sets.

use json_schema_traverse::keywords::{
    is_array_keyword, is_keyword, is_props_keyword, is_skip_keyword, ARRAY_KEYWORDS, KEYWORDS,
    PROPS_KEYWORDS, SKIP_KEYWORDS,
};

#[test]
fn keywords_set_is_exact() {
    let mut got = KEYWORDS.to_vec();
    got.sort_unstable();
    let mut want = vec![
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
    want.sort_unstable();
    assert_eq!(got, want);
}

#[test]
fn array_keywords_set_is_exact() {
    let mut got = ARRAY_KEYWORDS.to_vec();
    got.sort_unstable();
    let mut want = vec!["items", "allOf", "anyOf", "oneOf"];
    want.sort_unstable();
    assert_eq!(got, want);
}

#[test]
fn props_keywords_set_is_exact() {
    let mut got = PROPS_KEYWORDS.to_vec();
    got.sort_unstable();
    let mut want = vec![
        "$defs",
        "definitions",
        "properties",
        "patternProperties",
        "dependencies",
    ];
    want.sort_unstable();
    assert_eq!(got, want);
}

#[test]
fn skip_keywords_set_is_exact() {
    let mut got = SKIP_KEYWORDS.to_vec();
    got.sort_unstable();
    let mut want = vec![
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
    want.sort_unstable();
    assert_eq!(got, want);
}

#[test]
fn items_is_both_keyword_and_array_keyword() {
    assert!(is_keyword("items"));
    assert!(is_array_keyword("items"));
}

#[test]
fn membership_helpers_agree_with_tables() {
    for k in KEYWORDS {
        assert!(is_keyword(k));
    }
    for k in ARRAY_KEYWORDS {
        assert!(is_array_keyword(k));
    }
    for k in PROPS_KEYWORDS {
        assert!(is_props_keyword(k));
    }
    for k in SKIP_KEYWORDS {
        assert!(is_skip_keyword(k));
    }
    assert!(!is_keyword("unknown"));
    assert!(!is_array_keyword("unknown"));
    assert!(!is_props_keyword("unknown"));
    assert!(!is_skip_keyword("unknown"));
}

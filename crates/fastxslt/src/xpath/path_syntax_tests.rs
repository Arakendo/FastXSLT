use std::ops::Range;

use super::{PathFailure, PathOrigin, parse_location_path};
use crate::xdm::owned_tree_experiment::SourceLocation;

fn location() -> SourceLocation {
    SourceLocation {
        resource: "memory:stylesheet.xsl".to_owned(),
        span: Range { start: 12, end: 25 },
    }
}

#[test]
fn parses_the_golden_relative_child_path() {
    let path = parse_location_path("greeting/name", location()).expect("path should parse");

    assert_eq!(path.steps, ["greeting", "name"]);
    assert_eq!(path.origin, PathOrigin::Relative);
    assert_eq!(path.final_predicate, None);
    assert_eq!(path.location, location());
}

#[test]
fn distinguishes_invalid_from_unsupported_path_syntax() {
    assert!(matches!(
        parse_location_path("", location()),
        Err(PathFailure::Invalid { .. })
    ));
    let invalid = parse_location_path("1greeting/name", location())
        .expect_err("an invalid child-name path must fail");
    let unsupported = parse_location_path("greeting///name", location())
        .expect_err("repeated descendant separators are outside the private slice");

    assert!(matches!(invalid, PathFailure::Invalid { .. }));
    assert!(matches!(unsupported, PathFailure::Unsupported { .. }));
    assert_eq!(failure_location(&invalid), &location());
    assert_eq!(failure_location(&unsupported), &location());
    assert!(matches!(
        parse_location_path("sum(for $i in item return $i)", location()),
        Err(PathFailure::Unsupported { .. })
    ));
}

#[test]
fn accepts_supported_ncname_punctuation_without_claiming_unicode_names() {
    let path = parse_location_path("catalog/item.name/item-2/_value", location())
        .expect("ASCII NCName punctuation belongs to the private grammar");
    assert_eq!(path.steps, ["catalog", "item.name", "item-2", "_value"]);

    assert!(matches!(
        parse_location_path("café/name", location()),
        Err(PathFailure::Unsupported { .. })
    ));
    assert!(matches!(
        parse_location_path("catalog/ancestor::node()", location()),
        Err(PathFailure::Unsupported { .. })
    ));
    let self_node = parse_location_path("catalog/self::node()", location())
        .expect("the admitted self node test should parse");
    assert_eq!(self_node.steps[1], "node()");
}

#[test]
fn explicit_and_abbreviated_axes_share_typed_step_semantics() {
    let implicit = parse_location_path("//center/south-east", location())
        .expect("implicit child steps should parse");
    let explicit = parse_location_path("//center/child::south-east", location())
        .expect("explicit named child-axis step should parse");

    assert_eq!(explicit.steps, implicit.steps);
    assert_eq!(explicit.origin, PathOrigin::Descendant);
    assert_eq!(
        parse_location_path("//center/*", location())
            .expect("abbreviated wildcard should parse")
            .steps,
        parse_location_path("//center/child::*", location())
            .expect("explicit wildcard should parse")
            .steps
    );
    assert_eq!(
        parse_location_path("//center/node()", location())
            .expect("abbreviated node test should parse")
            .steps,
        parse_location_path("//center/child::node()", location())
            .expect("explicit node test should parse")
            .steps
    );
    assert_eq!(
        parse_location_path("//west/@*", location())
            .expect("abbreviated attribute wildcard should parse")
            .steps,
        parse_location_path("//west/attribute::*", location())
            .expect("explicit attribute wildcard should parse")
            .steps
    );
    assert_eq!(
        parse_location_path("//west/@west-attr-2", location())
            .expect("abbreviated named attribute should parse")
            .steps,
        parse_location_path("//west/attribute::west-attr-2", location())
            .expect("explicit named attribute should parse")
            .steps
    );
}

fn failure_location(failure: &PathFailure) -> &SourceLocation {
    match failure {
        PathFailure::Invalid { location, .. } | PathFailure::Unsupported { location, .. } => {
            location
        }
    }
}

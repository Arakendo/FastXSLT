//! Conserved admission for the complete XSLT30 `decl/output` denominator.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
};

use super::{
    ExecutionFailure, ExecutionPolicy, FailureCategory, InvocationEntry, TransformRequest,
    TransformSetBuilder, compile_resource, execute_program, execute_transform_set,
    serialize_xml_bytes,
};
use crate::execution_control_experiment::{CancellationToken, InvocationControl, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder, resolve_reference};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
use crate::xslt::golden_semantics_experiment::STANDARD_INITIAL_TEMPLATE_NAME;
use crate::xslt30_overlay_test_support::assert_output_case_passed;

const SET_FILE: &str = "tests/decl/output/_output-test-set.xml";
const CASE_COUNT: usize = 232;
const OVERLAY: &str = include_str!("../../../../corpus/overlays/xslt30/output-denominator-v0.toml");

#[derive(Default)]
struct InventoryObservation {
    names: BTreeSet<String>,
    assertions: BTreeMap<String, usize>,
    specs: BTreeMap<String, usize>,
    features: BTreeMap<String, usize>,
    environment_shapes: BTreeMap<String, usize>,
    direct_stylesheets: usize,
    resolved_environment_stylesheets: usize,
    source_files: usize,
    inline_sources: usize,
    expected_file_references: usize,
}

#[test]
fn executes_output_0104_with_predefined_xml_attribute_prefix_and_literal_apostrophe() {
    let execution = execute_output_case("output-0104", None);
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert!(
        execution
            .actual
            .contains("<html xml:lang=\"en\" lang=\"en\">")
    );
    assert!(
        execution
            .actual
            .contains("<div style=\"don't try this\">example.org</div>")
    );
    assert!(!execution.actual.contains("&apos;"));
}

#[test]
fn executes_output_0128_without_injecting_html_content_type_metadata() {
    let execution = execute_assert_serialization_case("output-0128", "xml");
    assert_eq!(execution.method.as_deref(), Some("xml"));
    assert_eq!(execution.include_content_type, Some(true));
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_output_0110_with_explicit_xhtml_declaration_omission() {
    let execution = execute_assert_serialization_case("output-0110", "xhtml");
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert!(execution.omit_xml_declaration);
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_output_0105_with_explicit_xhtml_method_for_a_null_namespace_root() {
    let execution = execute_output_case("output-0105", None);
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert_eq!(
        execution.actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html><body>This is the body</body></html>"
    );
}

#[test]
fn executes_xhtml_indentation_boolean_variants_without_reflowing_text() {
    let expected = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\">\n  <body>This is the body</body>\n</html>";
    for case_name in ["output-0106", "output-0106a", "output-0106b"] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("xhtml"), "{case_name}");
        assert_eq!(execution.actual, expected, "{case_name}");
    }
}

#[test]
fn executes_xhtml_script_and_style_text_with_xml_compatible_escaping() {
    for case_name in ["output-0107", "output-0108"] {
        let execution = execute_assert_serialization_case(case_name, "xhtml");
        assert_eq!(execution.method.as_deref(), Some("xhtml"), "{case_name}");
        assert_eq!(
            execution.expected.as_deref(),
            Some(execution.actual.as_str()),
            "{case_name}"
        );
    }
}

#[test]
fn executes_output_0118_with_a_static_processing_instruction() {
    let execution = execute_output_case("output-0118", None);
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert!(
        execution
            .actual
            .contains("<?my-pi href=\"book.css\" type=\"text/css\"?>")
    );
    assert!(!execution.actual.contains("<!DOCTYPE"));
}

#[test]
fn executes_output_0109_with_an_empty_xhtml_namespace_root() {
    let execution = execute_output_case("output-0109", None);
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert_eq!(
        execution.actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"></html>"
    );
}

#[test]
fn executes_xslt30_true_lexicals_for_xhtml_declaration_omission() {
    for case_name in ["output-0110a", "output-0110b"] {
        let execution = execute_assert_serialization_case(case_name, "xhtml");
        assert_eq!(execution.method.as_deref(), Some("xhtml"));
        assert!(execution.omit_xml_declaration, "{case_name}");
        assert_eq!(
            execution.expected.as_deref(),
            Some(execution.actual.as_str()),
            "{case_name}"
        );
    }
}

#[test]
fn executes_output_0121_with_the_default_xhtml_declaration() {
    let execution = execute_assert_serialization_case("output-0121", "xhtml");
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert!(!execution.omit_xml_declaration);
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_output_0127_through_bounded_all_of_serialization_matches() {
    const CASE_NAME: &str = "output-0127";
    let execution = execute_output_case(CASE_NAME, None);
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert_eq!(execution.include_content_type, Some(false));
    assert!(execution.expected.is_none());

    let (test_set, _) = load_test_set();
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| {
            local_name(&test_set, *node) == "test-case"
                && attribute(&test_set, *node, "name") == Some(CASE_NAME)
        })
        .expect("pinned output-0127 case");
    let result = child_named(&test_set, case, "result").expect("output-0127 result");
    let all_of = first_element_child(&test_set, result).expect("output-0127 assertion");
    assert_eq!(local_name(&test_set, all_of), "all-of");
    let patterns = element_children(&test_set, all_of)
        .into_iter()
        .map(|assertion| {
            assert_eq!(local_name(&test_set, assertion), "serialization-matches");
            test_set.string_value(assertion)
        })
        .collect::<Vec<_>>();
    assert_eq!(patterns.len(), 2);
    for pattern in patterns {
        assert!(
            matches_literal_whitespace_pattern(&execution.actual, &pattern)
                .expect("output-0127 pattern belongs to the admitted comparator subset"),
            "pattern did not match: {pattern}"
        );
    }
}

#[test]
fn bounded_serialization_matcher_requires_whitespace_and_rejects_other_operators() {
    assert_eq!(
        matches_literal_whitespace_pattern(
            "<html xmlns=\"urn.test\">",
            "<html\\s+xmlns=\"urn.test\">"
        ),
        Some(true)
    );
    assert_eq!(
        matches_literal_whitespace_pattern(
            "<htmlxmlns=\"urn.test\">",
            "<html\\s+xmlns=\"urn.test\">"
        ),
        Some(false)
    );
    assert_eq!(
        matches_literal_whitespace_pattern("alpha", "alpha|beta"),
        None
    );
    assert_eq!(matches_literal_whitespace_pattern("42", "\\d+"), None);
}

#[test]
fn executes_xhtml_cdata_terminator_boundary_cases() {
    let paired_brackets = execute_output_case("output-0114", None);
    assert_eq!(paired_brackets.method.as_deref(), Some("xhtml"));
    assert_eq!(
        paired_brackets.actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><out><example><![CDATA[]]]]></example></out></html>"
    );

    let terminator = execute_output_case("output-0115", None);
    assert_eq!(terminator.method.as_deref(), Some("xhtml"));
    assert_eq!(
        terminator.actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><out><example><![CDATA[]]]]><![CDATA[>]]></example></out></html>"
    );
}

#[test]
fn executes_bounded_xhtml_doctype_variants() {
    let system = execute_output_case("output-0111", None);
    assert_eq!(system.doctype_system.as_deref(), Some("out.dtd"));
    assert_eq!(system.doctype_public, None);
    assert_eq!(
        system.actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><!DOCTYPE html SYSTEM \"out.dtd\"><html xmlns=\"http://www.w3.org/1999/xhtml\"></html>"
    );

    let public_only = execute_output_case("output-0112", None);
    assert_eq!(
        public_only.doctype_public.as_deref(),
        Some("-//BOAG//DTD Websites V1.3//EN")
    );
    assert_eq!(public_only.doctype_system, None);
    assert_eq!(
        public_only.actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"></html>"
    );

    let paired = execute_output_case("output-0113", None);
    assert_eq!(paired.doctype_system.as_deref(), Some("out.dtd"));
    assert_eq!(
        paired.doctype_public.as_deref(),
        Some("-//BOAG//DTD Websites V1.3//EN")
    );
    assert_eq!(
        paired.actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><!DOCTYPE html PUBLIC \"-//BOAG//DTD Websites V1.3//EN\" \"out.dtd\"><html xmlns=\"http://www.w3.org/1999/xhtml\"></html>"
    );
}

#[test]
fn executes_xhtml_void_elements_for_false_indentation_lexicals() {
    for case_name in ["output-0116", "output-0116a", "output-0116b"] {
        let execution = execute_assert_serialization_case(case_name, "xhtml");
        assert_eq!(execution.method.as_deref(), Some("xhtml"), "{case_name}");
        assert_eq!(execution.indent, Some(false), "{case_name}");
        assert_eq!(
            execution.expected.as_deref(),
            Some(execution.actual.as_str()),
            "{case_name}"
        );
    }
}

#[test]
fn executes_xhtml_public_only_and_cdata_controls() {
    let boolean_attribute = execute_output_case("output-0117", None);
    assert_eq!(
        boolean_attribute.doctype_public.as_deref(),
        Some("-//W3C//DTD HTML 4.0 Transitional")
    );
    assert_eq!(boolean_attribute.doctype_system, None);
    assert_eq!(
        boolean_attribute.actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><Option selected=\"selected\"></Option></html>"
    );

    let paired_body = execute_output_case("output-0119", None);
    assert_eq!(
        paired_body.actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><body></body></html>"
    );

    let cdata = execute_assert_serialization_case("output-0120", "xhtml");
    assert_eq!(cdata.expected.as_deref(), Some(cdata.actual.as_str()));
}

#[test]
fn executes_explicit_and_inferred_xhtml_content_type_metadata() {
    let explicit = execute_output_case("output-0126", None);
    assert_eq!(explicit.method.as_deref(), Some("xhtml"));
    assert!(explicit.actual.contains(
        "<head><meta http-equiv=\"Content-Type\" content=\"application/xhtml-xml; charset=UTF-8\" />"
    ));

    let inferred = execute_output_case("output-0130", None);
    assert_eq!(inferred.method, None);
    assert!(
        inferred
            .actual
            .starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>")
    );
    assert!(inferred.actual.contains(
        "<head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=UTF-8\" />"
    ));
}

#[test]
fn executes_explicit_false_lexicals_for_retained_xhtml_declaration() {
    for case_name in ["output-0148", "output-0148a", "output-0148b"] {
        let execution = execute_assert_serialization_case(case_name, "xhtml");
        assert_eq!(execution.method.as_deref(), Some("xhtml"));
        assert!(!execution.omit_xml_declaration, "{case_name}");
        assert_eq!(
            execution.expected.as_deref(),
            Some(execution.actual.as_str()),
            "{case_name}"
        );
    }
}

#[test]
fn executes_output_0129_as_descendant_text_without_injected_markup() {
    let execution = execute_assert_serialization_case("output-0129", "html");
    assert_eq!(execution.method.as_deref(), Some("text"));
    assert_eq!(execution.include_content_type, Some(true));
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_output_0166_as_utf8_without_a_byte_order_mark() {
    let execution = execute_assert_serialization_case("output-0166", "text");
    assert_eq!(execution.method.as_deref(), Some("xml"));
    assert_eq!(execution.encoding.as_deref(), Some("UTF-8"));
    assert_eq!(execution.byte_order_mark, Some(false));
    assert!(!execution.actual.starts_with('\u{feff}'));
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_output_0165_as_utf8_bytes_with_a_byte_order_mark() {
    const CASE_NAME: &str = "output-0165";
    let bytes = execute_output_bytes_case(CASE_NAME);
    assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]));
    assert_eq!(
        &bytes[3..],
        b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><html><body>Hello</body></html>"
    );
}

#[test]
fn executes_text_output_with_and_without_a_utf8_byte_order_mark() {
    let with_mark = execute_output_bytes_case("output-0171");
    assert_eq!(with_mark, b"\xef\xbb\xbfHello");

    let without_mark = execute_output_bytes_case("output-0172");
    assert_eq!(without_mark, b"Hello");
}

#[test]
fn executes_html_output_with_and_without_a_utf8_byte_order_mark() {
    let body = b"<html><body>Hello</body></html>";
    let with_mark = execute_output_bytes_case("output-0162");
    assert_eq!(&with_mark[..3], &[0xef, 0xbb, 0xbf]);
    assert_eq!(&with_mark[3..], body);

    let without_mark = execute_output_bytes_case("output-0163");
    assert_eq!(without_mark, body);
}

#[test]
fn executes_xhtml_byte_order_mark_boolean_variants() {
    let body = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><body>Hello</body></html>";
    for case_name in ["output-0136", "output-0136a", "output-0136b"] {
        let bytes = execute_output_bytes_case(case_name);
        assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]), "{case_name}");
        assert_eq!(&bytes[3..], body, "{case_name}");
    }
    for case_name in ["output-0137", "output-0137a", "output-0137b"] {
        assert_eq!(execute_output_bytes_case(case_name), body, "{case_name}");
    }
}

#[test]
fn executes_xhtml_utf8_multibyte_text_bytes() {
    let bytes = execute_output_bytes_case("output-0139");
    assert_eq!(
        bytes,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><body>HelloÁ</body></html>"
            .as_bytes()
    );
    assert!(bytes.ends_with(b"Hello\xc3\x81</body></html>"));
}

#[test]
fn executes_output_0131_as_a_multi_root_xhtml_result() {
    let execution = execute_assert_serialization_case("output-0131", "xhtml");
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_output_0122_with_merged_cdata_element_names() {
    let execution = execute_assert_serialization_case("output-0122", "xml");
    assert_eq!(execution.method.as_deref(), Some("xml"));
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_output_0138_with_expanded_name_cdata_selection() {
    let actual = String::from_utf8(execute_output_bytes_case("output-0138"))
        .expect("output-0138 declares UTF-8 output");

    for selected in [
        "<h1><![CDATA[a & b]]></h1>",
        "<one:h3 xmlns:one=\"http://ns.example.com\"><![CDATA[a & b]]></one:h3>",
        "<my:h3 xmlns:my=\"http://ns.example.com\"><![CDATA[a & b]]></my:h3>",
        "<h5><![CDATA[a & b]]></h5>",
    ] {
        assert!(
            actual.contains(selected),
            "missing selected fragment: {selected}\nactual: {actual}"
        );
    }
    for unselected in [
        "<h2>a &amp; b</h2>",
        "<h3>a &amp; b</h3>",
        "<h3 xmlns=\"http://www.mytest.example.org\">a &amp; b</h3>",
        "<h4>a &amp; b</h4>",
    ] {
        assert!(
            actual.contains(unselected),
            "missing unselected fragment: {unselected}"
        );
    }
}

#[test]
fn executes_output_0153_with_explicit_xml_10_serialization_version() {
    let execution = execute_output_case("output-0153", None);
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert_eq!(execution.version.as_deref(), Some("1.0"));
    assert_eq!(
        execution.actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"></html>"
    );
}

#[test]
fn executes_xhtml_content_type_insertion_and_replacement_cases() {
    for (case_name, media_type) in [
        ("output-0142", "application/xhtml-xml"),
        ("output-0143", "text/html"),
        ("output-0144", "text/html"),
        ("output-0145", "application/postscript"),
    ] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("xhtml"), "{case_name}");
        assert!(
            execution
                .actual
                .contains("<html xmlns=\"http://www.w3.org/1999/xhtml\">"),
            "{case_name}"
        );
        let meta =
            format!("<meta http-equiv=\"Content-Type\" content=\"{media_type}; charset=UTF-8\" />");
        assert!(execution.actual.contains(&meta), "{case_name}");
        assert_eq!(
            execution
                .actual
                .matches("http-equiv=\"Content-Type\"")
                .count(),
            1,
            "{case_name}"
        );
        assert!(!execution.actual.contains("media-type="), "{case_name}");
        let head_start = execution.actual.find("<head>").expect("XHTML head");
        let meta_start = execution.actual.find(&meta).expect("content-type meta");
        let head_end = execution.actual.find("</head>").expect("XHTML head end");
        assert!(
            head_start < meta_start && meta_start < head_end,
            "{case_name}"
        );
    }
}

#[test]
fn executes_output_0151_with_xhtml_empty_element_conventions() {
    let execution = execute_output_case("output-0151", None);
    assert!(execution.actual.contains(
        "<head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=UTF-8\" /><title></title></head>"
    ));
    assert!(execution.actual.contains("<body><p></p></body>"));
}

#[test]
fn executes_output_0156_with_inert_xml_content_type_setting() {
    let execution = execute_output_case("output-0156", None);
    assert_eq!(execution.method.as_deref(), Some("xml"));
    assert_eq!(execution.include_content_type, Some(false));
    assert_eq!(
        execution.actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out xmlns=\"http://www.w3.org/1999/xhtml\">Hello</out>"
    );
}

#[test]
fn executes_xml_output_with_inert_escape_uri_attributes_in_an_initial_mode() {
    for case_name in ["output-0155a", "output-0155b"] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("xml"), "{case_name}");
        assert!(execution.actual.contains("href=\"¡\""), "{case_name}");
        assert!(!execution.actual.contains("%C2%A1"), "{case_name}");
    }
}

#[test]
fn reports_native_invalid_output_properties_with_the_standard_static_error() {
    for case_name in [
        "output-0197",
        "output-0197a",
        "output-0198",
        "output-0198a",
        "output-0199",
        "output-0199a",
        "output-0280",
        "output-0280a",
        "output-0281",
        "output-0281a",
        "output-0282",
        "output-0282a",
        "output-0283",
        "output-0283a",
        "output-0284",
        "output-0230",
    ] {
        let failure = compile_output_case_failure(case_name, "XTSE0020");
        assert_eq!(failure.code, "XTSE0020", "{case_name}");
        assert_eq!(failure.category, FailureCategory::Invalid, "{case_name}");
        assert_eq!(
            failure
                .location
                .as_ref()
                .map(|location| location.resource.as_str()),
            Some(format!("urn:w3c:xslt30:{case_name}:stylesheet").as_str()),
            "{case_name}"
        );
    }
}

#[test]
fn ignores_xml_space_on_output_without_relaxing_serialization_attributes() {
    let execution = execute_output_case("output-0285", None);
    assert_eq!(execution.method, None);
    assert!(execution.actual.ends_with("<ok/>") || execution.actual.ends_with("<ok></ok>"));
}

#[test]
fn reports_missing_character_map_name_without_admitting_character_maps() {
    let case_name = "output-0501";
    let failure = compile_output_case_failure(case_name, "XTSE0010");
    assert_eq!(failure.code, "XTSE0010");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(
        failure
            .location
            .as_ref()
            .map(|location| location.resource.as_str()),
        Some("urn:w3c:xslt30:output-0501:stylesheet")
    );
}

#[test]
fn executes_output_0173_with_merged_standalone_and_cdata_settings() {
    let execution = execute_assert_serialization_case("output-0173", "xhtml");
    assert_eq!(execution.method.as_deref(), Some("xml"));
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_one_named_xml_character_map() {
    let execution = execute_assert_serialization_case("output-0201", "xml");
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
    assert!(execution.actual.contains("yy€yy"));
    assert!(!execution.actual.contains("yy$yy"));
}

#[test]
fn executes_direct_character_map_composition_and_local_override() {
    let inherited = execute_assert_serialization_case("output-0202", "xml");
    assert_eq!(
        inherited.expected.as_deref(),
        Some(inherited.actual.as_str())
    );
    assert!(inherited.actual.contains("yy€yy"));

    let overridden = execute_assert_serialization_case("output-0203", "xml");
    assert_eq!(
        overridden.expected.as_deref(),
        Some(overridden.actual.as_str())
    );
    assert!(overridden.actual.contains("xxAxx"));
    assert!(overridden.actual.contains("yy*yy"));
    assert!(!overridden.actual.contains("yy€yy"));
}

#[test]
fn executes_multiple_character_maps_with_text_output() {
    let execution = execute_assert_serialization_case("output-0303", "text");
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
    assert_eq!(execution.actual, "xx&xx\nyy+Ayy\nzz%&zz\n");
}

#[test]
fn resolves_character_map_qnames_by_expanded_name() {
    let execution = execute_assert_serialization_case("output-0205", "xml");
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
    assert!(execution.actual.contains("yy€yy"));
    assert!(!execution.actual.contains("yy$yy"));
}

#[test]
fn repeated_character_map_references_are_idempotent() {
    let execution = execute_assert_serialization_case("output-0206", "xml");
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
    assert!(execution.actual.contains("yy€yy"));
    assert!(!execution.actual.contains("yy$yy"));
}

#[test]
fn applies_multiple_character_maps_to_xhtml_output() {
    let execution = execute_output_case("output-0301", None);
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert!(
        execution
            .actual
            .contains("<html xmlns=\"http://www.w3.org/1999/xhtml\">")
    );
    let body = execution
        .actual
        .split_once("<body>")
        .expect("XHTML body start")
        .1;
    let (after_first, _) = body
        .strip_prefix("<p>xx&xx</p>")
        .expect("first mapped paragraph")
        .split_once("<p>yy+Ayy</p>")
        .expect("second mapped paragraph after first");
    assert!(after_first.chars().all(char::is_whitespace));
    let after_second = body
        .split_once("<p>yy+Ayy</p>")
        .expect("second mapped paragraph")
        .1;
    let (between, after_third) = after_second
        .split_once("<p>zz%&zz</p>")
        .expect("third mapped paragraph after second");
    assert!(between.chars().all(char::is_whitespace));
    let before_close = after_third
        .strip_suffix("</body></html>")
        .expect("XHTML body and document close");
    assert!(before_close.chars().all(char::is_whitespace));
    assert!(!execution.actual.contains("yy*$yy"));
}

#[test]
fn applies_multiple_character_maps_to_bounded_html_output() {
    let execution = execute_output_case("output-0302", None);
    assert_eq!(execution.method.as_deref(), Some("html"));
    assert_eq!(
        execution.actual,
        "<html><body><p>xx&xx</p><p>yy+Ayy</p><p>zz%&zz</p></body></html>"
    );
}

#[test]
fn applies_an_html5_character_map_to_text_and_attribute_values() {
    let execution = execute_output_case("output-0604", None);
    assert_eq!(execution.method.as_deref(), Some("html"));
    assert!(execution.actual.contains("value=\"ab[C]de\""));
    assert!(execution.actual.contains("vw[X]yz"));
}

#[test]
fn serializes_an_html5_c1_control_as_a_numeric_reference() {
    let execution = execute_output_case("output-0195b", None);
    assert_eq!(execution.method.as_deref(), Some("html"));
    assert!(execution.actual.contains("<doc>&#x9F;</doc>"));
}

#[test]
fn executes_source_free_xhtml_non_uri_attribute_controls() {
    for case_name in [
        "output-0102d",
        "output-0102f",
        "output-0103d",
        "output-0103f",
    ] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("xhtml"));
        assert!(execution.actual.contains('¡'));
        assert!(!execution.actual.contains("%C2%A1"));
    }
}

#[test]
fn escapes_source_free_xhtml_c1_non_uri_attributes() {
    for case_name in ["output-0102e", "output-0103e"] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("xhtml"));
        assert!(execution.actual.contains("accesskey=\"&#x96;\""));
    }
}

#[test]
fn applies_source_free_xhtml_uri_attribute_escaping() {
    let enabled_a = execute_output_case("output-0102a", None);
    assert!(enabled_a.actual.contains("href=\"%C2%A1\""));
    let enabled_b = execute_output_case("output-0102b", None);
    assert!(enabled_b.actual.contains("href=\"%C2%96\""));
    let enabled_c = execute_output_case("output-0102c", None);
    assert!(
        enabled_c
            .actual
            .contains("href=\"% %C2%96 %C2%96 a &quot;  %C2%A1 &lt; &gt; &amp; end\""),
        "{}",
        enabled_c.actual
    );

    let disabled_a = execute_output_case("output-0103a", None);
    assert!(disabled_a.actual.contains("href=\"¡\""));
    let disabled_b = execute_output_case("output-0103b", None);
    assert!(disabled_b.actual.contains("href=\"&#x96;\""));
    let disabled_c = execute_output_case("output-0103c", None);
    assert!(
        disabled_c
            .actual
            .contains("href=\"% %C2%96 &#x96; a &quot;  ¡ &lt; &gt; &amp; end\"")
    );
}

#[test]
fn applies_bounded_html_content_type_meta_controls() {
    for case_name in ["output-0123", "output-0124", "output-0124a", "output-0124b"] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("html"));
        assert!(execution.actual.contains(
            "<meta http-equiv=\"Content-Type\" content=\"application/xhtml-xml; charset=UTF-8\">"
        ));
        assert!(!execution.actual.contains("charset=UTF-8\" />"));
    }

    for case_name in ["output-0125", "output-0125a", "output-0125b"] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("html"));
        assert!(!execution.actual.to_ascii_lowercase().contains("http-equiv"));
        assert!(!execution.actual.contains("charset=UTF-8"));
    }
}

#[test]
fn replaces_bounded_existing_html_content_type_meta() {
    for case_name in ["output-0157", "output-0158"] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("html"));
        assert_eq!(
            execution
                .actual
                .matches("http-equiv=\"Content-Type\"")
                .count(),
            1
        );
        assert!(
            execution
                .actual
                .contains("content=\"text/html; charset=UTF-8\"")
        );
        assert!(!execution.actual.contains("UTF-16"));
        assert!(!execution.actual.contains("version='3.0'"));
    }
}

#[test]
fn applies_multiple_character_maps_to_xml_output() {
    let execution = execute_assert_serialization_case("output-0309", "xml");
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
    assert!(execution.actual.contains("<a>xx&xx</a>"));
    assert!(execution.actual.contains("<a>yy+Ayy</a>"));
}

#[test]
fn cdata_text_bypasses_xml_character_maps() {
    let execution = execute_assert_serialization_case("output-0310", "xml");
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
    assert!(execution.actual.contains("<a>AAA</a>"));
    assert!(execution.actual.contains("<b><![CDATA[aaa]]></b>"));
    assert!(execution.actual.contains("<b><![CDATA[xx#xx]]></b>"));
}

#[test]
fn merges_output_character_map_lists_in_declaration_order() {
    let execution = execute_assert_serialization_case("output-0305", "xml");
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
    assert!(execution.actual.contains("<p>xx&xx</p>"));
    assert!(execution.actual.contains("<p>yy*Byy</p>"));
    assert!(execution.actual.contains("<p>zz*&zz</p>"));
}

#[test]
fn merges_output_map_lists_after_nested_import_precedence() {
    let execution = execute_assert_serialization_case("output-0306", "xml");
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
    assert!(execution.actual.contains("<p>xx&xx</p>"));
    assert!(execution.actual.contains("<p>yy*Byy</p>"));
    assert!(execution.actual.contains("<p>zz*&zz</p>"));
}

#[test]
fn serializes_quote_characters_inside_doctype_identifiers() {
    const CASE_NAME: &str = "output-0311";
    let execution = execute_output_case(CASE_NAME, None);
    assert_eq!(execution.doctype_public.as_deref(), Some("ABC'DEF"));
    assert_eq!(execution.doctype_system.as_deref(), Some("ABC\"DEF"));
    assert!(execution.actual.starts_with("<?xml"));
    assert!(
        execution
            .actual
            .contains("<!DOCTYPE a PUBLIC \"ABC'DEF\" 'ABC\"DEF'>")
    );
    assert!(execution.actual.ends_with("<a/>"));

    let (test_set, _) = load_test_set();
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| {
            local_name(&test_set, *node) == "test-case"
                && attribute(&test_set, *node, "name") == Some(CASE_NAME)
        })
        .expect("pinned output-0311 case");
    let result = child_named(&test_set, case, "result").expect("output-0311 result");
    let all_of = first_element_child(&test_set, result).expect("output-0311 assertion");
    assert_eq!(local_name(&test_set, all_of), "all-of");
    let patterns = element_children(&test_set, all_of)
        .into_iter()
        .map(|assertion| {
            assert_eq!(local_name(&test_set, assertion), "serialization-matches");
            test_set.string_value(assertion)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        patterns,
        [
            "<\\?xml",
            "<!DOCTYPE\\s+a\\s+PUBLIC\\s+\"ABC'DEF\"\\s+'ABC\"DEF'>",
            "<a/>"
        ]
    );
}

#[test]
fn executes_bounded_xhtml5_automatic_doctype_selection() {
    for (case_name, root_name) in [
        ("output-0208", "html"),
        ("output-0209", "HTML"),
        ("output-0210", "HtMl"),
        ("output-0212", "html"),
    ] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("xhtml"), "{case_name}");
        assert!(
            execution
                .actual
                .contains(&format!("<!DOCTYPE {root_name}>")),
            "{case_name}"
        );
        assert!(
            execution.actual.contains(&format!("<{root_name}")),
            "{case_name}"
        );
    }
    let metadata = execute_output_case("output-0208", None);
    assert!(
        metadata
            .actual
            .contains("<meta http-equiv=\"Content-Type\"")
    );

    for case_name in ["output-0213", "output-0214", "output-0215"] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("xhtml"), "{case_name}");
        assert!(!execution.actual.contains("<!DOCTYPE"), "{case_name}");
        assert!(
            !execution.actual.contains("http-equiv=\"Content-Type\""),
            "{case_name}"
        );
    }
    assert!(
        execute_output_case("output-0215", None)
            .actual
            .contains("<z:html xmlns:z=\"http://www.example.com/not-xhtml\">")
    );
}

#[test]
fn normalizes_xhtml_element_prefixes_for_bounded_version_five() {
    let execution = execute_output_case("output-0211", None);
    assert!(execution.actual.contains("<!DOCTYPE html>"));
    assert!(
        execution
            .actual
            .contains("<html xmlns=\"http://www.w3.org/1999/xhtml\">")
    );
    assert!(!execution.actual.contains("xmlns:h"));
    assert!(!execution.actual.contains("h:"));
    assert!(execution.actual.contains("<head>"));
    assert!(execution.actual.contains("<body>"));
}

#[test]
fn executes_bounded_xhtml5_empty_element_rules() {
    const VOID_NAMES: [&str; 14] = [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ];
    for case_name in ["output-0216", "output-0217", "output-0222", "output-0223"] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("xhtml"), "{case_name}");
        for name in VOID_NAMES {
            assert!(
                execution.actual.contains(&format!("<{name}"))
                    && !execution.actual.contains(&format!("</{name}>")),
                "{case_name}: {name}"
            );
        }
    }

    for case_name in ["output-0218", "output-0219", "output-0220", "output-0221"] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("xhtml"), "{case_name}");
        for name in ["title", "p", "i", "u", "div", "code", "strong"] {
            assert!(
                execution.actual.contains(&format!("</{name}>")),
                "{case_name}: {name}"
            );
        }
    }

    let prefixed = execute_output_case("output-0221", None);
    assert!(!prefixed.actual.contains("xmlns:h"));
    assert!(!prefixed.actual.contains("h:"));
}

#[test]
fn executes_bounded_xhtml5_explicit_doctype_variants() {
    const PUBLIC: &str = "-//W3C//DTD XHTML 1.0 Strict//EN";
    const SYSTEM: &str = "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd";

    for case_name in ["output-0227", "output-0231"] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(
            execution.doctype_public.as_deref(),
            Some(PUBLIC),
            "{case_name}"
        );
        assert_eq!(
            execution.doctype_system.as_deref(),
            Some(SYSTEM),
            "{case_name}"
        );
        assert!(
            execution
                .actual
                .contains(&format!("<!DOCTYPE html PUBLIC \"{PUBLIC}\" \"{SYSTEM}\">")),
            "{case_name}"
        );
    }

    let system_only = execute_output_case("output-0228", None);
    assert_eq!(system_only.doctype_public, None);
    assert_eq!(system_only.doctype_system.as_deref(), Some(SYSTEM));
    assert!(
        system_only
            .actual
            .contains(&format!("<!DOCTYPE html SYSTEM \"{SYSTEM}\">"))
    );

    let public_only = execute_output_case("output-0229", None);
    assert_eq!(public_only.doctype_public.as_deref(), Some(PUBLIC));
    assert_eq!(public_only.doctype_system, None);
    assert!(public_only.actual.contains("<!DOCTYPE html>"));
    assert!(!public_only.actual.contains(" PUBLIC "));
}

#[test]
fn normalizes_bounded_xhtml5_svg_and_mathml_element_namespaces() {
    for case_name in ["output-0224", "output-0225", "output-0226"] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some("xhtml"), "{case_name}");
        assert!(
            execution
                .actual
                .contains("<html xmlns=\"http://www.w3.org/1999/xhtml\">"),
            "{case_name}"
        );
        assert!(
            execution
                .actual
                .contains("<svg xmlns=\"http://www.w3.org/2000/svg\""),
            "{case_name}: {}",
            execution.actual
        );
        assert!(!execution.actual.contains("h:"), "{case_name}");
        assert!(!execution.actual.contains("s:"), "{case_name}");
    }

    for case_name in ["output-0224", "output-0225"] {
        let execution = execute_output_case(case_name, None);
        assert!(
            execution
                .actual
                .contains("<math xmlns=\"http://www.w3.org/1998/Math/MathML\""),
            "{case_name}"
        );
        assert!(!execution.actual.contains("m:"), "{case_name}");
    }
}

#[test]
fn places_xml_doctype_after_root_misc_and_before_document_element() {
    let execution = execute_output_case("output-0234", None);
    let comment = execution
        .actual
        .find("<!--This should precede the DOCTYPE declaration-->")
        .expect("serialized root comment");
    let processing_instruction = execution
        .actual
        .find("<?pi R squared?>")
        .expect("serialized root processing instruction");
    let doctype = execution
        .actual
        .find("<!DOCTYPE out PUBLIC \"//PUBLIC//\" \"system.dtd\">")
        .expect("serialized doctype");
    let element = execution.actual.find("<out>").expect("document element");
    assert!(comment < processing_instruction);
    assert!(processing_instruction < doctype);
    assert!(doctype < element);
}

#[test]
fn reports_html_serialization_parameter_failures_before_shape_selection() {
    for (case_name, code) in [
        ("output-0184", "SESU0007"),
        ("output-0191", "SESU0011"),
        ("output-0194", "SESU0013"),
    ] {
        let failure = try_execute_output_case(case_name, None)
            .expect_err("unsupported HTML serialization parameter must fail");
        assert_eq!(failure.code, code, "{case_name}");
        assert_eq!(
            failure.category,
            FailureCategory::Unsupported,
            "{case_name}"
        );
        assert_eq!(
            failure.request_id.as_deref(),
            Some(case_name),
            "{case_name}"
        );
    }
}

#[test]
fn reports_html_processing_instruction_delimiter_as_sere0015() {
    let case_name = "output-0196";
    let failure = try_execute_output_case(case_name, None)
        .expect_err("HTML processing-instruction data containing > must fail");
    assert_eq!(failure.code, "SERE0015");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.request_id.as_deref(), Some(case_name));
}

#[test]
fn places_automatic_html5_doctype_after_root_comment() {
    let execution = execute_output_case("output-0233", None);
    let comment = execution
        .actual
        .find("<!--This should precede the DOCTYPE declaration-->")
        .expect("serialized root comment");
    let doctype = execution
        .actual
        .find("<!DOCTYPE html>")
        .expect("automatic HTML 5 doctype");
    let element = execution.actual.find("<html>").expect("document element");
    assert!(comment < doctype);
    assert!(doctype < element);
}

#[test]
fn serializes_html5_void_elements_without_end_tags() {
    let execution = execute_output_case("output-0601", None);
    for name in [
        "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen", "link",
        "meta", "param", "source", "track", "wbr",
    ] {
        assert!(execution.actual.contains(&format!("<{name}>")), "{name}");
        assert!(!execution.actual.contains(&format!("</{name}>")), "{name}");
        assert!(!execution.actual.contains(&format!("<{name}/>")), "{name}");
    }
}

#[test]
fn normalizes_html5_svg_mathml_and_preserves_foreign_element_namespaces() {
    let svg = execute_output_case("output-0602a", None);
    assert!(
        svg.actual
            .contains("<svg xmlns=\"http://www.w3.org/2000/svg\"")
    );
    assert!(!svg.actual.contains("xmlns:svg="));

    let mathml = execute_output_case("output-0602b", None);
    assert!(
        mathml
            .actual
            .contains("<math xmlns=\"http://www.w3.org/1998/Math/MathML\"")
    );
    assert!(!mathml.actual.contains("xmlns:mathML="));

    let foreign = execute_output_case("output-0602c", None);
    assert!(foreign.actual.contains("xmlns:n=\"NamespaceN\""));
    assert!(foreign.actual.contains("<n:zzz>"));
}

#[test]
fn preserves_html5_namespaced_attribute_prefixes() {
    let svg = execute_output_case("output-0603a", None);
    assert!(
        svg.actual
            .contains("xmlns:svg=\"http://www.w3.org/2000/svg\"")
    );
    assert!(svg.actual.contains("svg:att=\"34\""));

    let mathml = execute_output_case("output-0603b", None);
    assert!(
        mathml
            .actual
            .contains("xmlns:mathML=\"http://www.w3.org/1998/Math/MathML\"")
    );
    assert!(mathml.actual.contains("mathML:att=\"123\""));

    let foreign = execute_output_case("output-0603c", None);
    assert!(foreign.actual.contains("xmlns:m=\"NamespaceM\""));
    assert!(foreign.actual.contains("<p m:zzz=\"value\">"));
}

#[test]
fn resolves_imported_character_maps_and_higher_precedence_override() {
    let imported_only = execute_assert_serialization_case("output-0204", "xml");
    assert_eq!(
        imported_only.expected.as_deref(),
        Some(imported_only.actual.as_str())
    );
    assert!(imported_only.actual.contains("yy€yy"));

    let overridden = execute_assert_serialization_case("output-0207", "xml");
    assert_eq!(
        overridden.expected.as_deref(),
        Some(overridden.actual.as_str())
    );
    assert!(overridden.actual.contains("yy*yy"));
    assert!(!overridden.actual.contains("yy€yy"));
}

#[test]
fn higher_precedence_empty_doctype_identifiers_suppress_imported_values() {
    let execution = execute_assert_serialization_case("output-0312", "xml");
    assert_eq!(execution.expected.as_deref(), Some("<a><b/></a>"));
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
    assert_eq!(execution.doctype_public.as_deref(), Some(""));
    assert_eq!(execution.doctype_system.as_deref(), Some(""));
}

#[test]
fn executes_xhtml_standalone_yes_no_and_omit_variants() {
    for case_name in [
        "output-0149",
        "output-0149a",
        "output-0149b",
        "output-0150",
        "output-0150a",
        "output-0150b",
        "output-0152",
    ] {
        let execution = execute_assert_serialization_case(case_name, "xhtml");
        assert_eq!(execution.method.as_deref(), Some("xhtml"), "{case_name}");
        assert_eq!(
            execution.expected.as_deref(),
            Some(execution.actual.as_str()),
            "{case_name}"
        );
    }
}

#[test]
fn executes_xml_and_text_with_normalization_form_none() {
    let decomposed = "A\u{301}";
    let xml = execute_output_bytes_case("output-0168");
    assert_eq!(
        xml,
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><html><body>{decomposed}</body></html>")
            .as_bytes()
    );
    assert!(xml.windows(3).any(|bytes| bytes == [0x41, 0xcc, 0x81]));

    let text = execute_output_bytes_case("output-0170");
    assert_eq!(text, decomposed.as_bytes());

    let html = execute_output_bytes_case("output-0161");
    assert_eq!(
        html,
        format!("<html><body>{decomposed}</body></html>").as_bytes()
    );
    assert!(html.windows(3).any(|bytes| bytes == [0x41, 0xcc, 0x81]));
}

#[test]
fn preserves_html_ins_and_del_content_whitespace_without_indent() {
    let execution = execute_output_case("output-0160", None);
    assert_eq!(execution.method.as_deref(), Some("html"));
    assert!(
        execution
            .actual
            .contains("is <del>\n    \t\t\t20\t</del><ins>     1 2 </ins> pieces")
    );
}

#[test]
fn emits_manually_escaped_html_script_as_raw_text() {
    let execution = execute_output_case("output-0154", None);
    assert_eq!(execution.method.as_deref(), Some("html"));
    assert!(execution.actual.contains(
        "<script type=\"text/javascript\">document.write (\"<EM>This will work<\\/EM>\")</script>"
    ));
}

#[test]
fn preserves_bounded_html_raw_and_preformatted_text() {
    let execution = execute_output_case("output-0159", None);
    assert_eq!(execution.method.as_deref(), Some("html"));
    assert!(execution.actual.contains(
        "<script type=\"text/javascript\">document.write (\"<EM>This will work<\\/EM>\")"
    ));
    assert!(
        execution
            .actual
            .contains("<style>\n\n        < > &\n\n      </style>")
    );
    assert!(execution.actual.contains("<b> Save    all\tspace here</b>"));
    assert!(execution.actual.contains(
        "<textarea rows=\"2\" cols=\"20\">\nThe cat was playing in the garden.\n\nSuddenly a dog showed up....."
    ));
}

#[test]
fn serializes_bounded_html5_input_value_without_uri_escaping() {
    let execution = execute_output_case("output-0724", None);
    assert_eq!(execution.method.as_deref(), Some("html"));
    assert!(
        execution
            .actual
            .ends_with("<input type=\"text\" value=\"✈\">")
    );
    assert!(!execution.actual.contains("%E2%9C%88"));
    assert!(!execution.actual.ends_with("/>"));
}

#[test]
fn suppresses_indentation_inside_bounded_html_and_xhtml_paragraphs() {
    let paragraph = "<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</p>";
    for (case_name, method) in [("output-0725", "html"), ("output-0726", "xhtml")] {
        let execution = execute_output_case(case_name, None);
        assert_eq!(execution.method.as_deref(), Some(method), "{case_name}");
        assert_eq!(
            execution.suppress_indentation_elements,
            [crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: "p".to_owned(),
            }],
            "{case_name}"
        );
        assert!(execution.actual.contains(paragraph), "{case_name}");
    }
}

#[test]
fn reports_inconsistent_xml_serialization_parameters_as_sepm0009() {
    for case_name in ["output-0186", "output-0187"] {
        let (test_set, _) = load_test_set();
        let root = document_element(&test_set);
        let case = element_children(&test_set, root)
            .into_iter()
            .find(|node| {
                local_name(&test_set, *node) == "test-case"
                    && attribute(&test_set, *node, "name") == Some(case_name)
            })
            .expect("pinned serialization-error case");
        let result = child_named(&test_set, case, "result").expect("output result");
        let assertion = first_element_child(&test_set, result).expect("serialization error");
        assert_eq!(
            local_name(&test_set, assertion),
            "assert-serialization-error"
        );
        assert_eq!(attribute(&test_set, assertion, "code"), Some("SEPM0009"));

        let failure = try_execute_output_case(case_name, None)
            .expect_err("inconsistent output parameters must fail serialization");
        assert_eq!(failure.code, "SEPM0009", "{case_name}");
        assert_eq!(failure.category, FailureCategory::Invalid, "{case_name}");
        assert_eq!(
            failure.request_id.as_deref(),
            Some(case_name),
            "{case_name}"
        );
    }
}

#[test]
fn reports_xml_10_prefix_undeclaration_as_sepm0010() {
    let case_name = "output-0188";
    let (test_set, _) = load_test_set();
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| {
            local_name(&test_set, *node) == "test-case"
                && attribute(&test_set, *node, "name") == Some(case_name)
        })
        .expect("pinned serialization-error case");
    let result = child_named(&test_set, case, "result").expect("output result");
    let assertion = first_element_child(&test_set, result).expect("serialization error");
    assert_eq!(
        local_name(&test_set, assertion),
        "assert-serialization-error"
    );
    assert_eq!(attribute(&test_set, assertion, "code"), Some("SEPM0010"));

    let failure = try_execute_output_case(case_name, None)
        .expect_err("XML 1.0 cannot serialize namespace undeclarations");
    assert_eq!(failure.code, "SEPM0010");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.request_id.as_deref(), Some(case_name));
}

#[test]
fn reports_document_shaped_serialization_requirements_as_sepm0004() {
    for case_name in ["output-0182", "output-0183"] {
        let (test_set, _) = load_test_set();
        let root = document_element(&test_set);
        let case = element_children(&test_set, root)
            .into_iter()
            .find(|node| {
                local_name(&test_set, *node) == "test-case"
                    && attribute(&test_set, *node, "name") == Some(case_name)
            })
            .expect("pinned serialization-error case");
        let result = child_named(&test_set, case, "result").expect("output result");
        let assertion = first_element_child(&test_set, result).expect("serialization error");
        assert_eq!(
            local_name(&test_set, assertion),
            "assert-serialization-error"
        );
        assert_eq!(attribute(&test_set, assertion, "code"), Some("SEPM0004"));

        let failure = try_execute_output_case(case_name, None)
            .expect_err("document-shaped serialization must reject multiple elements");
        assert_eq!(failure.code, "SEPM0004", "{case_name}");
        assert_eq!(failure.category, FailureCategory::Invalid, "{case_name}");
        assert_eq!(
            failure.request_id.as_deref(),
            Some(case_name),
            "{case_name}"
        );
    }
}

#[test]
fn reports_unsupported_serialization_encoding_as_sesu0007() {
    let case_name = "output-0185";
    let (test_set, _) = load_test_set();
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| {
            local_name(&test_set, *node) == "test-case"
                && attribute(&test_set, *node, "name") == Some(case_name)
        })
        .expect("pinned serialization-error case");
    let result = child_named(&test_set, case, "result").expect("output result");
    let assertion = first_element_child(&test_set, result).expect("serialization error");
    assert_eq!(
        local_name(&test_set, assertion),
        "assert-serialization-error"
    );
    assert_eq!(attribute(&test_set, assertion, "code"), Some("SESU0007"));

    let failure = try_execute_output_case(case_name, None)
        .expect_err("an unavailable requested encoding must fail serialization");
    assert_eq!(failure.code, "SESU0007");
    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(failure.request_id.as_deref(), Some(case_name));
}

#[test]
fn reports_unavailable_xml_compatible_encodings_through_native_alternatives() {
    for case_name in ["output-0178", "output-0180"] {
        let (test_set, _) = load_test_set();
        let root = document_element(&test_set);
        let case = element_children(&test_set, root)
            .into_iter()
            .find(|node| {
                local_name(&test_set, *node) == "test-case"
                    && attribute(&test_set, *node, "name") == Some(case_name)
            })
            .expect("pinned encoding alternative case");
        let result = child_named(&test_set, case, "result").expect("output result");
        let any_of = first_element_child(&test_set, result).expect("any-of assertion");
        assert_eq!(local_name(&test_set, any_of), "any-of");
        let native_codes: Vec<_> = element_children(&test_set, any_of)
            .into_iter()
            .filter(|node| local_name(&test_set, *node) == "assert-serialization-error")
            .filter_map(|node| attribute(&test_set, node, "code"))
            .collect();
        assert!(native_codes.contains(&"SESU0007"), "{case_name}");

        let failure = try_execute_output_case(case_name, None)
            .expect_err("an unavailable requested encoding must fail serialization");
        assert_eq!(failure.code, "SESU0007", "{case_name}");
        assert_eq!(
            failure.category,
            FailureCategory::Unsupported,
            "{case_name}"
        );
        assert_eq!(
            failure.request_id.as_deref(),
            Some(case_name),
            "{case_name}"
        );
    }
}

#[test]
fn reports_unsupported_normalization_form_as_sesu0011() {
    for case_name in ["output-0189", "output-0190", "output-0192"] {
        let (test_set, _) = load_test_set();
        let root = document_element(&test_set);
        let case = element_children(&test_set, root)
            .into_iter()
            .find(|node| {
                local_name(&test_set, *node) == "test-case"
                    && attribute(&test_set, *node, "name") == Some(case_name)
            })
            .expect("pinned serialization-error case");
        let result = child_named(&test_set, case, "result").expect("output result");
        let assertion = first_element_child(&test_set, result).expect("serialization error");
        assert_eq!(
            local_name(&test_set, assertion),
            "assert-serialization-error"
        );
        assert_eq!(attribute(&test_set, assertion, "code"), Some("SESU0011"));

        let failure = try_execute_output_case(case_name, None)
            .expect_err("an unavailable normalization form must fail serialization");
        assert_eq!(failure.code, "SESU0011", "{case_name}");
        assert_eq!(
            failure.category,
            FailureCategory::Unsupported,
            "{case_name}"
        );
        assert_eq!(
            failure.request_id.as_deref(),
            Some(case_name),
            "{case_name}"
        );
    }
}

#[test]
fn reports_unavailable_fully_normalized_as_an_admitted_native_alternative() {
    let case_name = "output-0193";
    let (test_set, _) = load_test_set();
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| {
            local_name(&test_set, *node) == "test-case"
                && attribute(&test_set, *node, "name") == Some(case_name)
        })
        .expect("pinned normalization-error case");
    let result = child_named(&test_set, case, "result").expect("output result");
    let any_of = first_element_child(&test_set, result).expect("any-of assertion");
    assert_eq!(local_name(&test_set, any_of), "any-of");
    let native_codes: Vec<_> = element_children(&test_set, any_of)
        .into_iter()
        .filter(|node| local_name(&test_set, *node) == "assert-serialization-error")
        .filter_map(|node| attribute(&test_set, node, "code"))
        .collect();
    assert!(native_codes.contains(&"SESU0011"));

    let failure = try_execute_output_case(case_name, None)
        .expect_err("unavailable fully-normalized output must fail serialization");
    assert_eq!(failure.code, "SESU0011");
    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(failure.request_id.as_deref(), Some(case_name));
}

#[test]
fn executes_xhtml_with_normalization_form_none() {
    let decomposed = "A\u{301}";
    let bytes = execute_output_bytes_case("output-0147");
    assert_eq!(
        bytes,
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><body>{decomposed}</body></html>"
        )
        .as_bytes()
    );
    assert!(bytes.windows(3).any(|part| part == [0x41, 0xcc, 0x81]));
}

#[derive(Debug)]
struct SerializationExecution {
    method: Option<String>,
    version: Option<String>,
    encoding: Option<String>,
    doctype_system: Option<String>,
    doctype_public: Option<String>,
    include_content_type: Option<bool>,
    byte_order_mark: Option<bool>,
    suppress_indentation_elements: Vec<crate::xml::quick_xml_experiment::ExpandedName>,
    omit_xml_declaration: bool,
    indent: Option<bool>,
    actual: String,
    expected: Option<String>,
}

fn execute_assert_serialization_case(
    case_name: &str,
    assertion_method: &str,
) -> SerializationExecution {
    execute_output_case(case_name, Some(assertion_method))
}

fn execute_output_case(case_name: &str, assertion_method: Option<&str>) -> SerializationExecution {
    try_execute_output_case(case_name, assertion_method).expect("execute output case")
}

fn output_stylesheet_nodes(
    document: &Document,
    test: NodeId,
    environment: Option<NodeId>,
) -> Vec<NodeId> {
    let local = element_children(document, test)
        .into_iter()
        .filter(|node| local_name(document, *node) == "stylesheet")
        .collect::<Vec<_>>();
    if local.is_empty() {
        environment.map_or_else(Vec::new, |environment| {
            element_children(document, environment)
                .into_iter()
                .filter(|node| local_name(document, *node) == "stylesheet")
                .collect()
        })
    } else {
        local
    }
}

fn try_execute_output_case(
    case_name: &str,
    assertion_method: Option<&str>,
) -> Result<SerializationExecution, ExecutionFailure> {
    assert_output_case_passed(case_name);

    let (test_set, set_path) = load_test_set();
    let directory = set_path.parent().expect("output test-set directory");
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| {
            local_name(&test_set, *node) == "test-case"
                && attribute(&test_set, *node, "name") == Some(case_name)
        })
        .expect("pinned output case");
    let test = child_named(&test_set, case, "test").expect("output test");
    let environment = resolve_environment(&test_set, root, case);
    let stylesheet_nodes = output_stylesheet_nodes(&test_set, test, environment);
    let stylesheet_file = stylesheet_nodes
        .first()
        .copied()
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("output stylesheet file");
    let source = environment.and_then(|environment| child_named(&test_set, environment, "source"));
    let source_bytes = source.map(|source| {
        if let Some(file) = attribute(&test_set, source, "file") {
            fs::read(directory.join(file)).expect("read source and close handle")
        } else {
            let content = child_named(&test_set, source, "content").expect("inline source content");
            test_set.string_value(content).into_bytes()
        }
    });
    let expected_file =
        assertion_method.map(|method| expected_serialization_file(&test_set, case, method));

    let source_id = format!("urn:w3c:xslt30:{case_name}:source");
    let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
    let resource_count = stylesheet_nodes.len() + usize::from(source_bytes.is_some());
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(
        resource_count,
        65_536,
        resource_count * 65_536,
    ));
    if let Some(source_bytes) = source_bytes {
        resources
            .admit(source_id.clone(), source_bytes)
            .expect("admit output source");
    }
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file)).expect("read stylesheet and close handle"),
        )
        .expect("admit output stylesheet");
    admit_secondary_stylesheets(
        &mut resources,
        &test_set,
        &stylesheet_nodes,
        directory,
        &stylesheet_id,
    );
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile output case");
    let output_settings = program.output.clone();

    let mut set = TransformSetBuilder::new(
        snapshot,
        program,
        1,
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit: 8_192,
            work_limits: WorkLimits::unbounded(),
        },
    );
    set.add(TransformRequest {
        identity: case_name.to_owned(),
        result_identity: format!("result:{case_name}"),
        entry: output_invocation_entry(&test_set, test, source.map(|_| source_id)),
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit output request");
    let results = execute_transform_set(set.seal())?;
    let actual = results.by_request[case_name].serialized.clone();
    let expected = expected_file.map(|expected_file| {
        fs::read_to_string(directory.join(expected_file))
            .expect("read expected serialization and close handle")
            .replace("\r\n", "\n")
    });
    Ok(SerializationExecution {
        method: output_settings.method,
        version: output_settings.version,
        encoding: output_settings.encoding,
        doctype_system: output_settings.doctype_system,
        doctype_public: output_settings.doctype_public,
        include_content_type: output_settings.include_content_type,
        byte_order_mark: output_settings.byte_order_mark,
        suppress_indentation_elements: output_settings.suppress_indentation_elements,
        omit_xml_declaration: output_settings.omit_xml_declaration,
        indent: output_settings.indent,
        actual,
        expected,
    })
}

fn expected_serialization_file(test_set: &Document, case: NodeId, method: &str) -> String {
    let result = child_named(test_set, case, "result").expect("output result");
    let top_level = first_element_child(test_set, result).expect("output assertion");
    let assertion = if local_name(test_set, top_level) == "any-of" {
        child_named(test_set, top_level, "assert-serialization")
            .expect("admitted any-of has a file-backed serialization alternative")
    } else {
        top_level
    };
    assert_eq!(local_name(test_set, assertion), "assert-serialization");
    assert_eq!(attribute(test_set, assertion, "method"), Some(method));
    attribute(test_set, assertion, "file")
        .expect("expected file")
        .to_owned()
}

fn admit_secondary_stylesheets(
    resources: &mut ResourceSetBuilder,
    test_set: &Document,
    stylesheet_nodes: &[NodeId],
    directory: &Path,
    principal_identity: &str,
) {
    for stylesheet in stylesheet_nodes.iter().skip(1) {
        let file = attribute(test_set, *stylesheet, "file").expect("secondary stylesheet file");
        let (identity, fragment) = resolve_reference(principal_identity, file)
            .expect("resolve secondary stylesheet identity from principal base");
        assert!(fragment.is_none(), "stylesheet files do not use fragments");
        resources
            .admit(
                identity,
                fs::read(directory.join(file)).expect("read secondary stylesheet and close handle"),
            )
            .expect("admit secondary output stylesheet");
    }
}

fn output_invocation_entry(
    test_set: &Document,
    test: NodeId,
    source_id: Option<String>,
) -> InvocationEntry {
    if let Some(name) = child_named(test_set, test, "initial-template")
        .and_then(|node| attribute(test_set, node, "name"))
    {
        return InvocationEntry::InitialTemplate {
            name: name.to_owned(),
        };
    }
    let initial_mode = child_named(test_set, test, "initial-mode")
        .and_then(|node| attribute(test_set, node, "name"));
    let Some(source_id) = source_id else {
        return InvocationEntry::InitialTemplate {
            name: STANDARD_INITIAL_TEMPLATE_NAME.to_owned(),
        };
    };
    match initial_mode {
        Some(name) => InvocationEntry::InitialMode {
            resource: source_id,
            name: name.to_owned(),
        },
        None => InvocationEntry::PrincipalSource {
            resource: source_id,
        },
    }
}

fn compile_output_case_failure(case_name: &str, expected_code: &str) -> super::ExecutionFailure {
    assert!(OVERLAY.contains(&format!("case_name = \"{case_name}\"")));
    let (test_set, set_path) = load_test_set();
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| attribute(&test_set, *node, "name") == Some(case_name))
        .expect("pinned invalid output case");
    let result = child_named(&test_set, case, "result").expect("output result");
    assert!(descendants(result, &test_set).into_iter().any(|node| {
        local_name(&test_set, node) == "error"
            && attribute(&test_set, node, "code") == Some(expected_code)
    }));
    let test = child_named(&test_set, case, "test").expect("output test");
    let file = child_named(&test_set, test, "stylesheet")
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("output stylesheet file");
    let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 65_536, 65_536));
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(set_path.parent().expect("test-set directory").join(file))
                .expect("read stylesheet and close handle"),
        )
        .expect("admit invalid output stylesheet");
    compile_resource(&resources.seal(), &stylesheet_id)
        .expect_err("invalid output property must fail compilation")
}

fn execute_output_bytes_case(case_name: &str) -> Vec<u8> {
    assert!(OVERLAY.contains(&format!("case_name = \"{case_name}\"")));
    let (test_set, set_path) = load_test_set();
    let directory = set_path.parent().expect("output test-set directory");
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| {
            local_name(&test_set, *node) == "test-case"
                && attribute(&test_set, *node, "name") == Some(case_name)
        })
        .expect("pinned byte-output case");
    let test = child_named(&test_set, case, "test").expect("byte-output test");
    let stylesheet_file = child_named(&test_set, test, "stylesheet")
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("byte-output stylesheet file");
    let environment = resolve_environment(&test_set, root, case).expect("byte-output environment");
    let source = child_named(&test_set, environment, "source").expect("byte-output source");
    let source_content = child_named(&test_set, source, "content").expect("inline source content");
    let source_id = format!("urn:w3c:xslt30:{case_name}:source");
    let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
    let source_bytes = test_set.string_value(source_content).into_bytes();

    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(source_id.clone(), source_bytes.clone())
        .expect("admit byte-output source");
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file))
                .expect("read and close byte-output stylesheet"),
        )
        .expect("admit byte-output stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile byte-output case");
    assert_eq!(program.output.encoding.as_deref(), Some("UTF-8"));

    let parsed = parse_document(
        &source_id,
        &source_bytes,
        ParseLimits {
            max_events: 1_024,
            max_depth: 64,
        },
    )
    .expect("parse byte-output source");
    let source = Document::from_parsed(parsed).expect("build byte-output source XDM");
    let mut control = InvocationControl::unbounded();
    let result = execute_program(&program, &source, case_name, &mut control)
        .expect("execute byte-output case");
    serialize_xml_bytes(&result, &program.output, case_name, 8_192, &mut control)
        .expect("serialize UTF-8 output with a byte-order mark")
}

fn matches_literal_whitespace_pattern(actual: &str, pattern: &str) -> Option<bool> {
    if pattern.contains(['[', ']', '(', ')', '|', '*', '?', '^', '$', '{', '}'])
        || pattern.replace("\\s+", "").contains('\\')
    {
        return None;
    }
    let mut remainder = actual;
    for (index, literal) in pattern.split("\\s+").enumerate() {
        let Some(position) = remainder.find(literal) else {
            return Some(false);
        };
        let skipped = &remainder[..position];
        if index > 0 && (skipped.is_empty() || !skipped.chars().all(char::is_whitespace)) {
            return Some(false);
        }
        remainder = &remainder[position + literal.len()..];
    }
    Some(true)
}

#[test]
fn inventories_complete_output_denominator_and_seals_each_environment() {
    assert!(OVERLAY.contains(&format!("set_file = \"{SET_FILE}\"")));
    assert!(OVERLAY.contains(&format!("case_count = {CASE_COUNT}")));
    assert!(OVERLAY.contains("selection = \"harness-unsupported\""));
    assert!(OVERLAY.contains("execution = \"not-run\""));

    let (test_set, set_path) = load_test_set();
    let directory = set_path.parent().expect("output test-set directory");
    let root = document_element(&test_set);
    assert_eq!(
        dependency_records(&test_set, root, "feature"),
        vec![("serialization", Some("true"))]
    );
    assert_unique_environments(&test_set, root);

    let observation = observe_cases(&test_set, root, directory);
    assert_complete_inventory(&observation);
}

fn assert_unique_environments(document: &Document, root: NodeId) {
    let environments = element_children(document, root)
        .into_iter()
        .filter(|node| local_name(document, *node) == "environment")
        .collect::<Vec<_>>();
    let environment_names = environments
        .iter()
        .map(|node| {
            attribute(document, *node, "name")
                .expect("top-level environment identity")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(environment_names.len(), environments.len());
}

fn observe_cases(document: &Document, root: NodeId, directory: &Path) -> InventoryObservation {
    let cases = element_children(document, root)
        .into_iter()
        .filter(|node| local_name(document, *node) == "test-case")
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), CASE_COUNT);

    let mut observation = InventoryObservation::default();
    for case in cases {
        observe_case(document, root, case, directory, &mut observation);
    }
    observation
}

fn observe_case(
    document: &Document,
    root: NodeId,
    case: NodeId,
    directory: &Path,
    observation: &mut InventoryObservation,
) {
    let case_name = attribute(document, case, "name").expect("output case identity");
    assert!(
        observation.names.insert(case_name.to_owned()),
        "duplicate case {case_name}"
    );
    count_dependencies(document, case, "spec", &mut observation.specs);
    count_dependencies(document, case, "feature", &mut observation.features);

    let result = child_named(document, case, "result").expect("output case result");
    let assertion = first_element_child(document, result).expect("output assertion");
    *observation
        .assertions
        .entry(local_name(document, assertion).to_owned())
        .or_insert(0) += 1;
    observation.expected_file_references += verify_expected_files(document, result, directory);

    let environment = resolve_environment(document, root, case);
    let environment_shape = match child_named(document, case, "environment") {
        Some(reference) if attribute(document, reference, "ref").is_some() => "ref",
        Some(_) => "inline",
        None => "missing",
    };
    *observation
        .environment_shapes
        .entry(environment_shape.to_owned())
        .or_insert(0) += 1;

    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(16, 65_536, 524_288));
    let mut admitted = Vec::new();
    if let Some(test) = child_named(document, case, "test") {
        observation.direct_stylesheets += admit_stylesheets(
            document,
            test,
            directory,
            case_name,
            "test",
            &mut resources,
            &mut admitted,
        );
    }
    if let Some(environment) = environment {
        observation.resolved_environment_stylesheets += admit_stylesheets(
            document,
            environment,
            directory,
            case_name,
            "environment",
            &mut resources,
            &mut admitted,
        );
        let (files, inline) = admit_sources(
            document,
            environment,
            directory,
            case_name,
            &mut resources,
            &mut admitted,
        );
        observation.source_files += files;
        observation.inline_sources += inline;
    }
    assert!(
        !admitted.is_empty(),
        "{case_name} has no admitted engine input"
    );
    let snapshot = resources.seal();
    for identity in admitted {
        assert!(snapshot.get(&identity).is_some(), "missing {identity}");
    }
}

fn assert_complete_inventory(observation: &InventoryObservation) {
    assert_eq!(observation.names.len(), CASE_COUNT);
    assert_eq!(
        observation.assertions,
        BTreeMap::from([
            ("all-of".to_owned(), 89),
            ("any-of".to_owned(), 29),
            ("assert-serialization".to_owned(), 43),
            ("assert-serialization-error".to_owned(), 14),
            ("error".to_owned(), 6),
            ("not".to_owned(), 4),
            ("serialization-matches".to_owned(), 47),
        ])
    );
    assert_eq!(
        observation.specs,
        BTreeMap::from([
            ("XSLT10+".to_owned(), 1),
            ("XSLT20".to_owned(), 7),
            ("XSLT20+".to_owned(), 133),
            ("XSLT30+".to_owned(), 91)
        ])
    );
    assert_eq!(
        observation.features,
        BTreeMap::from([
            ("HTML4".to_owned(), 1),
            ("HTML5".to_owned(), 2),
            ("XPath_3.1".to_owned(), 14),
            ("higher_order_functions".to_owned(), 1),
        ])
    );
    assert_eq!(
        observation.environment_shapes,
        BTreeMap::from([
            ("inline".to_owned(), 3),
            ("missing".to_owned(), 27),
            ("ref".to_owned(), 202)
        ])
    );
    assert_eq!(observation.direct_stylesheets, 223);
    assert_eq!(observation.resolved_environment_stylesheets, 18);
    assert_eq!(observation.source_files, 7);
    assert_eq!(observation.inline_sources, 186);
    assert_eq!(observation.expected_file_references, 50);

    for (family, count) in &observation.assertions {
        assert!(OVERLAY.contains(&format!("name = \"{family}\"\ncount = {count}")));
    }
}

fn admit_stylesheets(
    document: &Document,
    owner: NodeId,
    directory: &Path,
    case_name: &str,
    owner_kind: &str,
    resources: &mut ResourceSetBuilder,
    admitted: &mut Vec<String>,
) -> usize {
    let stylesheets = element_children(document, owner)
        .into_iter()
        .filter(|node| local_name(document, *node) == "stylesheet")
        .collect::<Vec<_>>();
    for (ordinal, stylesheet) in stylesheets.iter().enumerate() {
        let file = attribute(document, *stylesheet, "file").expect("stylesheet file");
        let identity = format!(
            "urn:w3c:xslt30:decl:output:{case_name}:{owner_kind}:stylesheet:{ordinal}:{file}"
        );
        resources
            .admit(
                identity.clone(),
                fs::read(directory.join(file)).expect("read stylesheet and close handle"),
            )
            .expect("admit bounded stylesheet");
        admitted.push(identity);
    }
    stylesheets.len()
}

fn admit_sources(
    document: &Document,
    environment: NodeId,
    directory: &Path,
    case_name: &str,
    resources: &mut ResourceSetBuilder,
    admitted: &mut Vec<String>,
) -> (usize, usize) {
    let sources = element_children(document, environment)
        .into_iter()
        .filter(|node| local_name(document, *node) == "source")
        .collect::<Vec<_>>();
    let mut file_count = 0;
    let mut inline_count = 0;
    for (ordinal, source) in sources.iter().enumerate() {
        let (suffix, bytes) = if let Some(file) = attribute(document, *source, "file") {
            file_count += 1;
            (
                file.to_owned(),
                fs::read(directory.join(file)).expect("read source and close handle"),
            )
        } else {
            inline_count += 1;
            let content = child_named(document, *source, "content").expect("inline source content");
            (
                "inline".to_owned(),
                document.string_value(content).into_bytes(),
            )
        };
        let identity = format!("urn:w3c:xslt30:decl:output:{case_name}:source:{ordinal}:{suffix}");
        resources
            .admit(identity.clone(), bytes)
            .expect("admit bounded source");
        admitted.push(identity);
    }
    (file_count, inline_count)
}

fn verify_expected_files(document: &Document, result: NodeId, directory: &Path) -> usize {
    descendants(result, document)
        .into_iter()
        .filter_map(|node| attribute(document, node, "file"))
        .inspect(|file| {
            assert!(
                directory.join(file).is_file(),
                "missing expected file {file}"
            );
        })
        .count()
}

fn resolve_environment(document: &Document, root: NodeId, case: NodeId) -> Option<NodeId> {
    let reference = child_named(document, case, "environment")?;
    let Some(name) = attribute(document, reference, "ref") else {
        return Some(reference);
    };
    element_children(document, root).into_iter().find(|node| {
        local_name(document, *node) == "environment"
            && attribute(document, *node, "name") == Some(name)
    })
}

fn count_dependencies(
    document: &Document,
    case: NodeId,
    kind: &str,
    counts: &mut BTreeMap<String, usize>,
) {
    for (value, _) in dependency_records(document, case, kind) {
        *counts.entry(value.to_owned()).or_insert(0) += 1;
    }
}

fn dependency_records<'a>(
    document: &'a Document,
    owner: NodeId,
    kind: &str,
) -> Vec<(&'a str, Option<&'a str>)> {
    let Some(dependencies) = child_named(document, owner, "dependencies") else {
        return Vec::new();
    };
    element_children(document, dependencies)
        .into_iter()
        .filter(|node| local_name(document, *node) == kind)
        .map(|node| {
            (
                attribute(document, node, "value").expect("dependency value"),
                attribute(document, node, "satisfied"),
            )
        })
        .collect()
}

fn load_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/decl/output/_output-test-set.xml");
    let bytes = fs::read(&path).expect("read pinned output test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:decl:output:test-set",
        &bytes,
        ParseLimits {
            max_events: 50_000,
            max_depth: 128,
        },
    )
    .expect("parse pinned output test set");
    (
        Document::from_parsed(parsed).expect("build output test-set document"),
        path,
    )
}

fn document_element(document: &Document) -> NodeId {
    first_element_child(document, document.document_node()).expect("test-set document element")
}

fn descendants(parent: NodeId, document: &Document) -> Vec<NodeId> {
    let mut found = Vec::new();
    for child in element_children(document, parent) {
        found.push(child);
        found.extend(descendants(child, document));
    }
    found
}

fn element_children(document: &Document, parent: NodeId) -> Vec<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .filter(|node| document.kind(*node) == NodeKind::Element)
        .collect()
}

fn child_named(document: &Document, parent: NodeId, local: &str) -> Option<NodeId> {
    element_children(document, parent)
        .into_iter()
        .find(|node| local_name(document, *node) == local)
}

fn first_element_child(document: &Document, parent: NodeId) -> Option<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .find(|node| document.kind(*node) == NodeKind::Element)
}

fn local_name(document: &Document, node: NodeId) -> &str {
    &document.name(node).expect("element name").local
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
}

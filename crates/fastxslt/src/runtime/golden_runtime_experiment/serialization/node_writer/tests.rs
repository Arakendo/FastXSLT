//! Differential output, charge-point and namespace-unwind controls.

use super::super::{NamespaceMode, NormalizationForm};
use super::*;
use crate::execution_control_experiment::{InvocationControl, WorkDomain, WorkLimits};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};

fn name(namespace: Option<&str>, local: &str) -> ExpandedName {
    ExpandedName {
        namespace: namespace.map(str::to_owned),
        local: local.to_owned(),
    }
}

fn element(namespace: Option<&str>, local: &str, children: Vec<ResultNode>) -> ResultNode {
    ResultNode::Element {
        name: name(namespace, local),
        namespaces: Vec::new().into(),
        attributes: Vec::new(),
        children,
    }
}

fn fixture(namespace: Option<&str>) -> ResultNode {
    let mut root = element(
        namespace,
        "html",
        vec![
            element(namespace, "head", vec![element(namespace, "meta", vec![])]),
            element(
                namespace,
                "body",
                vec![
                    element(namespace, "script", vec![ResultNode::Text("a<&é".into())]),
                    element(namespace, "data", vec![ResultNode::Text("x]]>y".into())]),
                    element(namespace, "br", vec![]),
                    ResultNode::Comment("comment".into()),
                    ResultNode::ProcessingInstruction {
                        target: "pi".into(),
                        value: "value".into(),
                    },
                    ResultNode::Xslt10DisableOutputEscapingText("<raw/>".into()),
                ],
            ),
        ],
    );
    if let ResultNode::Element {
        namespaces,
        attributes,
        children,
        ..
    } = &mut root
    {
        *namespaces = vec![
            NamespaceBinding {
                prefix: None,
                namespace: namespace.unwrap_or("").into(),
            },
            NamespaceBinding {
                prefix: Some("p".into()),
                namespace: "urn:attribute".into(),
            },
        ]
        .into();
        attributes.push(ResultAttribute {
            name: name(Some("urn:attribute"), "key"),
            value: "a&\"é".into(),
        });
        children.push(element(None, "plain", vec![]));
    }
    root
}

fn options<'a>(cdata: &'a [ExpandedName], maps: &'a [(char, String)]) -> SerializationOptions<'a> {
    SerializationOptions {
        cdata_section_elements: cdata,
        suppress_indentation_elements: &[],
        character_map: maps,
        xhtml_mode: XhtmlMode::None,
        content_type_media_type: Some("text/html"),
        content_type_encoding: "UTF-8",
        html_mode: HtmlMode::None,
        html_raw_text_context: HtmlRawTextContext::Inactive,
        escape_uri_attributes: true,
        normalization_form: NormalizationForm::Nfc,
        xml_empty_element_tag: false,
        indent: true,
        indentation_state: IndentationState::Enabled,
    }
}

fn run(
    node: &ResultNode,
    options: SerializationOptions<'_>,
    recursive: bool,
    byte_limit: usize,
    work_limit: usize,
    cancel_at: Option<usize>,
) -> (Result<(), ExecutionFailure>, String, usize) {
    let mut control = InvocationControl::new(
        crate::execution_control_experiment::CancellationToken::new(),
        WorkLimits {
            serialized_bytes: work_limit,
            ..WorkLimits::unbounded()
        },
    );
    if let Some(index) = cancel_at {
        control = control.cancelling_on_charge(WorkDomain::SerializedByte, index);
    }
    let mode = if options.xhtml_mode == XhtmlMode::DefaultNamespace {
        NamespaceMode::CompleteClone
    } else {
        NamespaceMode::ScopedStack
    };
    let mut scope = NamespaceScope::new(mode);
    let mut output = BudgetedString::new(byte_limit, "writer-parity", &mut control);
    let result = if recursive {
        recursive_reference::serialize_node(node, &mut scope, options, 0, &mut output)
    } else {
        serialize_node(node, &mut scope, options, 0, &mut output)
    };
    let bytes = output.finish();
    // The root's p binding must be gone even after a nested output failure.
    let probe = element(Some("urn:attribute"), "probe", vec![]);
    let mut fresh_control = InvocationControl::unbounded();
    let mut fresh_output = BudgetedString::new(8192, "unwind", &mut fresh_control);
    assert!(serialize_node(&probe, &mut scope, options, 0, &mut fresh_output).is_err());
    (result, bytes, control.consumed(WorkDomain::SerializedByte))
}

#[test]
fn iterative_writer_matches_recursive_output_modes_and_work_charges() {
    let maps = [('é', "mapped".into())];
    for namespace in [
        None,
        Some("urn:result"),
        Some("http://www.w3.org/1999/xhtml"),
    ] {
        let node = fixture(namespace);
        let cdata = [name(namespace, "data")];
        for html in [HtmlMode::None, HtmlMode::Legacy, HtmlMode::Five] {
            for xhtml in [
                XhtmlMode::None,
                XhtmlMode::PreservePrefixes,
                XhtmlMode::DefaultNamespace,
            ] {
                for indent in [false, true] {
                    for xml_empty_element_tag in [false, true] {
                        let options = SerializationOptions {
                            html_mode: html,
                            xhtml_mode: xhtml,
                            indent,
                            xml_empty_element_tag,
                            ..options(&cdata, &maps)
                        };
                        let expected = run(&node, options, true, 8192, usize::MAX, None);
                        assert!(expected.0.is_ok());
                        assert_eq!(run(&node, options, false, 8192, usize::MAX, None), expected);
                    }
                }
            }
        }
    }
}

#[test]
fn iterative_writer_preserves_failure_prefix_charges_and_namespace_unwind() {
    let node = fixture(Some("urn:result"));
    let options = options(&[], &[]);
    let full = run(&node, options, true, 8192, usize::MAX, None);
    for limit in 0..=full.1.len() {
        assert_eq!(
            run(&node, options, false, limit, usize::MAX, None),
            run(&node, options, true, limit, usize::MAX, None)
        );
        assert_eq!(
            run(&node, options, false, 8192, limit, None),
            run(&node, options, true, 8192, limit, None)
        );
    }
    for charge in 0..=full.1.len() {
        assert_eq!(
            run(&node, options, false, 8192, usize::MAX, Some(charge)),
            run(&node, options, true, 8192, usize::MAX, Some(charge))
        );
    }
}

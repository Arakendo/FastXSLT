//! AR-0016 runtime parity and overlapping-generation controls.

use std::collections::BTreeMap;
use std::sync::{Arc, Barrier};

use crate::execution_control_experiment::InvocationControl;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::{
    MultipleMatchPolicy, WhitespaceRepresentation, execute_program_with_parameters_using,
    serialize_xml,
};

const LIMITS: ParseLimits = ParseLimits {
    max_events: 256,
    max_depth: 16,
};

fn document(identity: &str, xml: &[u8]) -> Document {
    Document::from_parsed(
        parse_document(identity, xml, LIMITS).expect("AR-0016 fixture should parse"),
    )
    .expect("AR-0016 fixture XDM should build")
}

fn compile(identity: &str, xml: &[u8]) -> StylesheetProgram {
    crate::compile::golden_stylesheet_experiment::compile_stylesheet(&document(identity, xml))
        .expect("AR-0016 stylesheet should compile")
}

fn execute_with(
    program: &StylesheetProgram,
    source: &Document,
    representation: WhitespaceRepresentation,
    request_id: &str,
) -> String {
    let mut control = InvocationControl::unbounded();
    let semantic = execute_program_with_parameters_using(
        program,
        source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        request_id,
        representation,
        &mut control,
    )
    .expect("AR-0016 differential transform should execute");
    serialize_xml(&semantic, &program.output, request_id, 8_192, &mut control)
        .expect("AR-0016 differential result should serialize")
}

#[test]
fn effective_positions_and_source_copy_match_the_complete_reference() {
    let source = document(
        "memory:whitespace-position-source.xml",
        b"<root>\n  <member>one</member>\n  <member>two</member>\n  <member>three</member>\n</root>",
    );
    let program = compile(
        "memory:whitespace-position-style.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:output method="xml" omit-xml-declaration="yes"/>
            <xsl:strip-space elements="*"/>
            <xsl:template match="/"><xsl:apply-templates/></xsl:template>
            <xsl:template match="root"><out><xsl:apply-templates/></out></xsl:template>
            <xsl:template match="member[position()&lt;last()]"><position value="{position()}" last="{last()}"/><xsl:copy><xsl:apply-templates/></xsl:copy></xsl:template>
            <xsl:template match="member[position()=last()]"><position final="yes" value="{position()}" last="{last()}"/><xsl:copy><xsl:apply-templates/></xsl:copy></xsl:template>
        </xsl:stylesheet>"#,
    );

    let reference = execute_with(
        &program,
        &source,
        WhitespaceRepresentation::CompleteReference,
        "position-copy-reference",
    );
    let view = execute_with(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        "position-copy-view",
    );

    assert_eq!(view, reference);
    assert_eq!(
        view,
        "<out><position value=\"1\" last=\"3\"></position><member>one</member><position value=\"2\" last=\"3\"></position><member>two</member><position final=\"yes\" value=\"3\" last=\"3\"></position><member>three</member></out>"
    );
}

#[test]
fn overlapping_stylesheet_generations_keep_independent_whitespace_policies() {
    let source = Arc::new(document(
        "memory:whitespace-generation-source.xml",
        b"<root>  <a>A</a>\n  <b>B</b>  </root>",
    ));
    let old_generation = Arc::new(compile(
        "memory:whitespace-old-generation.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:strip-space elements="*"/><xsl:template match="/"><old><xsl:value-of select="."/></old></xsl:template></xsl:stylesheet>"#,
    ));
    let new_generation = Arc::new(compile(
        "memory:whitespace-new-generation.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><new><xsl:value-of select="."/></new></xsl:template></xsl:stylesheet>"#,
    ));
    let start = Arc::new(Barrier::new(3));

    let old_worker = {
        let source = source.clone();
        let program = old_generation.clone();
        let start = start.clone();
        std::thread::spawn(move || {
            start.wait();
            execute_with(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                "old-generation",
            )
        })
    };
    let new_worker = {
        let source = source.clone();
        let program = new_generation.clone();
        let start = start.clone();
        std::thread::spawn(move || {
            start.wait();
            execute_with(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                "new-generation",
            )
        })
    };
    start.wait();

    let old = old_worker.join().expect("old generation should join");
    let new = new_worker.join().expect("new generation should join");
    assert_eq!(old, "<old>AB</old>");
    assert_eq!(new, "<new>  A\n  B  </new>");
    assert_eq!(source.string_value(source.document_node()), "  A\n  B  ");
}

//! Path-union identity/order, focus, scalar conversion, and control conservation.
//! Test-only composition through the same compiled and sealed runtime boundary.

use super::{
    ResourceLimits, ResourceSetBuilder, SOURCE_ID, STYLESHEET_ID, TransformSetBuilder,
    compile_resource, execute_transform_set, policy, request,
};

#[test]
fn xslt10_path_unions_support_count_and_last_position() {
    const SOURCE: &str = "urn:fastxslt:path-union-position:source";
    const STYLESHEET: &str = "urn:fastxslt:path-union-position:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="articleinfo/section/simplesect/title"><count><xsl:value-of select="count(ancestor::section | ancestor::simplesect | ancestor::articleinfo)"/></count><last><xsl:for-each select="(ancestor::section | ancestor::simplesect | ancestor::articleinfo)[last()]"><xsl:value-of select="name(.)"/></xsl:for-each></last></xsl:for-each></out></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<articleinfo><section><simplesect><title>T</title></simplesect></section></articleinfo>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(16_384));
    builder
        .add(request("path-union-position", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute stylesheet");

    assert_eq!(
        results.by_request["path-union-position"].serialized,
        "<out><count>3</count><last>simplesect</last></out>"
    );
}

#[test]
fn xslt10_template_arguments_count_normalized_path_unions() {
    const SOURCE: &str = "urn:fastxslt:argument-count-union:source";
    const STYLESHEET: &str = "urn:fastxslt:argument-count-union:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates select="doc/a"/></xsl:template><xsl:template match="a"><xsl:call-template name="emit"><xsl:with-param name="count" select="count(following::node() | following::*/@*)"/></xsl:call-template></xsl:template><xsl:template name="emit"><xsl:param name="count"/><out><xsl:value-of select="$count"/></out></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, b"<doc><a/><b x=\"1\">t</b></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request("argument-count-union", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute stylesheet");

    assert_eq!(
        results.by_request["argument-count-union"].serialized,
        "<out>3</out>"
    );
}

#[test]
fn xslt10_sort_uses_the_first_document_order_node_from_a_path_union() {
    const SOURCE: &str = "urn:fastxslt:xslt10-sort-path-union:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-sort-path-union:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/entry"><xsl:sort select="name/last | primary/name/last"/></xsl:apply-templates></out></xsl:template><xsl:template match="entry"><xsl:value-of select="name/last"/><xsl:value-of select="primary/name/last"/><xsl:text>|</xsl:text></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            br"<doc><entry><name><last>Zulu</last></name></entry><entry><primary><name><last>Alpha</last></name></primary></entry><entry><primary><name><last>Mike</last></name></primary></entry></doc>".to_vec(),
        )
        .expect("admit path-union sorting source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit path-union sorting stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile path-union sort key");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("path-union-sort", "result", SOURCE))
        .expect("admit path-union sorting request");

    let results = execute_transform_set(builder.seal()).expect("execute path-union sort");
    assert_eq!(
        results.by_request["path-union-sort"].serialized,
        "<out>Alpha|Mike|Zulu|</out>"
    );
}

#[test]
fn apply_templates_path_union_normalizes_identity_and_document_order() {
    const SOURCE: &str = "urn:fastxslt:apply-union:source";
    const STYLESHEET: &str = "urn:fastxslt:apply-union:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br#"<doc a="A" b="B"/>"#.to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc"/></out></xsl:template><xsl:template match="doc"><xsl:apply-templates select="@b | @a | @b"/></xsl:template><xsl:template match="@*"><xsl:value-of select="."/><xsl:if test="position()!=last()">,</xsl:if></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile apply union");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("apply-union", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute apply union");
    assert_eq!(
        results.by_request["apply-union"].serialized,
        "<out>A,B</out>"
    );
}

#[test]
fn apply_templates_variable_path_union_normalizes_identity_and_document_order() {
    const SOURCE: &str = "urn:fastxslt:apply-variable-union:source";
    const STYLESHEET: &str = "urn:fastxslt:apply-variable-union:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            br"<doc><first>A</first><second>B</second></doc>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:variable name="chosen" select="doc/second"/><out><xsl:apply-templates select="$chosen | doc/first | $chosen"/></out></xsl:template><xsl:template match="first"><xsl:value-of select="."/><xsl:if test="position()!=last()">,</xsl:if></xsl:template><xsl:template match="second"><xsl:value-of select="."/><xsl:if test="position()!=last()">,</xsl:if></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile variable/path apply union");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("apply-variable-union", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute variable/path apply union");
    assert_eq!(
        results.by_request["apply-variable-union"].serialized,
        "<out>A,B</out>"
    );
}

#[test]
fn xslt10_value_of_path_union_uses_first_node_in_document_order() {
    const SOURCE: &str = "urn:fastxslt:value-union:source";
    const STYLESHEET: &str = "urn:fastxslt:value-union:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            br#"<doc xmlns:p="urn:test"><p:title>first</p:title><title>second</title></doc>"#
                .to_vec(),
        )
        .expect("admit value-union source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:test"><xsl:output method="text"/><xsl:template match="doc"><xsl:value-of select="title/text() | p:title/text()"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit value-union stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile XSLT 1.0 value-of path union");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(16_384));
    builder
        .add(request("value-union", "result", SOURCE))
        .expect("admit value-union request");

    let results = execute_transform_set(builder.seal()).expect("execute value-of path union");
    assert_eq!(results.by_request["value-union"].serialized, "first");
}

#[test]
fn xslt10_grouped_path_unions_preserve_suffix_focus_global_last_and_control_cleanup() {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    resources
        .admit(
            SOURCE_ID,
            br#"<r att1="outer"><middle att1="mid"><leaf att1="inner"/></middle></r>"#.to_vec(),
        )
        .unwrap();
    resources.admit(STYLESHEET_ID,
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:for-each select="//leaf"><xsl:value-of select="(ancestor::*|self::*)/@att1[last()]"/>|<xsl:value-of select="((ancestor::*|self::*)/@att1)[last()]"/>|<xsl:value-of select="(self::*|ancestor::*)/@att1"/>|<xsl:value-of select="(ancestor::*|ancestor::*|self::*)/@att1"/>|<xsl:value-of select="(ancestor::*|self::*)[last()]"/></xsl:for-each></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET_ID).unwrap();
    let mut limited = policy(4096);
    limited.work_limits.xpath_node_visits = 0;
    let mut builder = TransformSetBuilder::new(snapshot.clone(), program.clone(), 1, limited);
    builder
        .add(request("limited-union", "result", SOURCE_ID))
        .unwrap();
    assert_eq!(
        execute_transform_set(builder.seal()).unwrap_err().code,
        "FXCT0002"
    );
    let cancelled = request("cancelled-union", "result", SOURCE_ID);
    cancelled.cancellation.cancel();
    let mut builder = TransformSetBuilder::new(snapshot.clone(), program.clone(), 1, policy(4096));
    builder.add(cancelled).unwrap();
    assert_eq!(
        execute_transform_set(builder.seal()).unwrap_err().code,
        "FXCT0001"
    );
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4096));
    builder
        .add(request("grouped-union", "result", SOURCE_ID))
        .unwrap();
    let result = execute_transform_set(builder.seal()).unwrap();
    assert_eq!(
        result.by_request["grouped-union"].serialized,
        "outer|inner|outer|outer|"
    );
}

#[test]
fn copy_of_path_union_deduplicates_and_restores_document_order() {
    const SOURCE: &str = "urn:fastxslt:union-copy:source";
    const STYLESHEET: &str = "urn:fastxslt:union-copy:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 12_288, 24_576));
    resources
        .admit(
            SOURCE,
            br#"<doc xmlns:p="urn:items" z="last-expression" x="first-expression"><p:a/><a/></doc>"#.to_vec(),
        )
        .expect("admit source document");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:items" exclude-result-prefixes="p"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates select="doc"/></xsl:template><xsl:template match="doc"><out><xsl:copy-of select="@x | p:a | @z | p:a | a"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile copy-of path union");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(24_576));
    builder
        .add(request("union-copy", "union-copy-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute copy-of path union");

    assert_eq!(
        results.by_request["union-copy"].serialized,
        "<out z=\"last-expression\" x=\"first-expression\"><p:a xmlns:p=\"urn:items\"></p:a><a xmlns:p=\"urn:items\"></a></out>"
    );
}

#[test]
fn grouped_apply_path_union_composes_one_common_suffix() {
    const SOURCE: &str = "urn:fastxslt:grouped-union:source";
    const STYLESHEET: &str = "urn:fastxslt:grouped-union:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 12_288, 24_576));
    resources
        .admit(
            SOURCE,
            b"<root><north><before/><center/><after/></north><south/></root>".to_vec(),
        )
        .expect("admit source document");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:for-each select="//center"><xsl:apply-templates select="(preceding-sibling::*|following-sibling::*)/ancestor::*[last()]/*[last()]"/></xsl:for-each></xsl:template><xsl:template match="*"><xsl:value-of select="name(.)"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile grouped path union");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(24_576));
    builder
        .add(request("grouped-union", "grouped-union-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute grouped path union");

    assert_eq!(results.by_request["grouped-union"].serialized, "south");
}

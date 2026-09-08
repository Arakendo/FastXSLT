use std::sync::Arc;

use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::binary_numeric_experiment::BinaryNumericExpression;
use crate::xpath::case_conversion_experiment::CaseConversionExpression;
use crate::xpath::castable_experiment::{CastExpression, CastableExpression};
use crate::xpath::constant_boolean_experiment::ScalarExpression;
use crate::xpath::decimal_sum_for_experiment::DecimalSumForExpression;
use crate::xpath::deep_equal_boolean_experiment::DeepEqualBooleanExpression;
use crate::xpath::default_collation_experiment::DefaultCollationExpression;
use crate::xpath::duration_component_experiment::DurationComponentExpression;
use crate::xpath::effective_boolean_value_experiment::DocumentBooleanExpression;
use crate::xpath::empty_experiment::SequenceCardinalityExpression;
use crate::xpath::encode_for_uri_expression::EncodeForUriExpression;
use crate::xpath::escape_html_uri_expression::EscapeHtmlUriExpression;
use crate::xpath::focus_sum_for_experiment::FocusSumForExpression;
use crate::xpath::for_distinct_values_experiment::ForDistinctValuesExpression;
use crate::xpath::format_number_experiment::FormatNumberExpression;
use crate::xpath::integer_for_experiment::IntegerForExpression;
use crate::xpath::iri_to_uri_expression::IriToUriExpression;
use crate::xpath::path_experiment::LocationPath;
use crate::xpath::string_length_experiment::StringLengthExpression;

pub(crate) const STANDARD_INITIAL_TEMPLATE_NAME: &str =
    "Q{http://www.w3.org/1999/XSL/Transform}initial-template";

#[cfg(feature = "workbench")]
#[path = "golden_semantics_retention.rs"]
mod retention;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StylesheetProgram {
    pub(crate) declared_version: String,
    pub(crate) default_initial_mode: Option<String>,
    pub(crate) source_whitespace: SourceWhitespacePolicy,
    pub(crate) typed_mode_requirements: Vec<TypedModeRequirement>,
    pub(crate) private_initial_modes: Vec<PrivateInitialMode>,
    pub(crate) mode_policies: Vec<ModePolicy>,
    pub(crate) output: OutputSettings,
    pub(crate) output_specified_properties: Vec<String>,
    pub(crate) character_maps: Vec<CharacterMapDefinition>,
    pub(crate) output_character_map_names: Vec<ExpandedName>,
    pub(crate) output_character_map_location: Option<SourceLocation>,
    pub(crate) root_template: Option<Template>,
    pub(crate) root_template_modes: Vec<String>,
    pub(crate) matched_templates: Vec<MatchedTemplate>,
    pub(crate) named_templates: Vec<NamedTemplate>,
    pub(crate) global_bindings: Vec<GlobalBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CharacterMapDefinition {
    pub(crate) name: ExpandedName,
    pub(crate) referenced_map_names: Vec<ExpandedName>,
    pub(crate) entries: Vec<(char, String)>,
    pub(crate) location: SourceLocation,
}

impl StylesheetProgram {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypedModeRequirement {
    pub(crate) name: String,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PrivateInitialMode {
    pub(crate) name: String,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModePolicy {
    pub(crate) name: Option<String>,
    pub(crate) on_no_match: Option<OnNoMatchPolicy>,
    pub(crate) on_multiple_match: Option<OnMultipleMatchPolicy>,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnMultipleMatchPolicy {
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnNoMatchPolicy {
    Fail,
    ShallowCopy,
    ShallowSkip,
    TextOnlyCopy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceWhitespacePolicy {
    Preserve,
    StripAllElementWhitespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GlobalBindingKind {
    Variable,
    Parameter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GlobalBinding {
    pub(crate) kind: GlobalBindingKind,
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) default: GlobalBindingDefault,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GlobalBindingDefault {
    EmptySequence,
    Text(String),
    Atomic(AtomicValue),
    Integer(i64),
    DoubleDivision {
        numerator: LocationPath,
        denominator: LocationPath,
    },
    CountLocationPath(LocationPath),
    LocationPath(LocationPath),
    SourceNodeIdentity(LocationPath),
    Variable(String),
    TemporaryTree(Vec<ConstructedElement>),
    TemporaryText(String),
    Xslt10TemporarySourceString(LocationPath),
    TemporaryAttribute {
        name: ExpandedName,
        value: String,
    },
    TemporaryComment(String),
    TemporaryProcessingInstruction {
        target: String,
        value: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConstructedElement {
    pub(crate) name: ExpandedName,
    pub(crate) namespaces: Vec<NamespaceBinding>,
    pub(crate) attributes: Vec<ConstructedAttribute>,
    pub(crate) children: Vec<ConstructedNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConstructedAttribute {
    pub(crate) name: ExpandedName,
    pub(crate) value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConstructedNode {
    Element(ConstructedElement),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutputSettings {
    pub(crate) method: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) html_version: Option<String>,
    pub(crate) encoding: Option<String>,
    pub(crate) media_type: Option<String>,
    pub(crate) doctype_system: Option<String>,
    pub(crate) doctype_public: Option<String>,
    pub(crate) include_content_type: Option<bool>,
    pub(crate) escape_uri_attributes: Option<bool>,
    pub(crate) byte_order_mark: Option<bool>,
    pub(crate) normalization_form: Option<String>,
    pub(crate) character_map: Vec<(char, String)>,
    pub(crate) undeclare_prefixes: Option<bool>,
    pub(crate) standalone: Option<String>,
    pub(crate) cdata_section_elements: Vec<ExpandedName>,
    pub(crate) suppress_indentation_elements: Vec<ExpandedName>,
    pub(crate) omit_xml_declaration: bool,
    pub(crate) indent: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Template {
    pub(crate) parameters: Vec<TemplateParameter>,
    pub(crate) body: Vec<Instruction>,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemplateParameter {
    pub(crate) name: String,
    pub(crate) tunnel: bool,
    pub(crate) required: bool,
    pub(crate) default: TemplateParameterDefault,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TemplateParameterDefault {
    Text(String),
    Integer(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MatchedTemplate {
    pub(crate) pattern: MatchPattern,
    pub(crate) import_precedence: i32,
    pub(crate) priority: TemplatePriority,
    pub(crate) modes: Vec<String>,
    pub(crate) template: Template,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TemplatePriority(i64);

impl TemplatePriority {
    // The private comparison domain stores exact millionths. This retains the
    // standard half- and quarter-step defaults plus bounded explicit decimals
    // without binary floating-point ordering.
    const SCALE: i64 = 1_000_000;
    pub(crate) const PATH_DEFAULT: Self = Self(500_000);
    pub(crate) const EXACT_NAME_DEFAULT: Self = Self(0);
    pub(crate) const NAMESPACE_WILDCARD_DEFAULT: Self = Self(-250_000);
    pub(crate) const ROOT_DEFAULT: Self = Self(-500_000);
    pub(crate) const NODE_TEST_DEFAULT: Self = Self(-500_000);

    pub(crate) fn explicit_integer(value: i32) -> Self {
        Self(i64::from(value) * Self::SCALE)
    }

    pub(crate) fn explicit_millionths(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NamedTemplate {
    pub(crate) name: String,
    pub(crate) parameters: Vec<String>,
    pub(crate) template: Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MatchPattern {
    AtomicIntegerGreaterOrEqual(i64),
    Document,
    DocumentElement(Option<ExpandedName>),
    Element(ExpandedName),
    ElementLocal(String),
    ElementNamespace(String),
    DescendantAnyElement,
    ElementWithAttribute {
        element: ExpandedName,
        attribute: ExpandedName,
    },
    ElementWithAttributeValue {
        element: ExpandedName,
        attribute: ExpandedName,
        value: String,
    },
    ElementWithChild {
        element: ExpandedName,
        child: ChildPresenceTest,
    },
    AnyElementWithAttributeVariable {
        attribute: ExpandedName,
        variable: String,
    },
    VariableFilteredElementPath(VariableFilteredElementPath),
    ElementWithSameNamedChild,
    ElementWithSameNamedParent,
    ElementWithSameNamedParentAtPosition(usize),
    ElementAtNamedSiblingBoundary {
        element: ExpandedName,
        boundary: NamedSiblingBoundary,
    },
    QualifiedElementPathAlternatives(Vec<Vec<ExpandedName>>),
    UnionAlternatives(Vec<MatchPattern>),
    Path(LocationPath),
    Attribute(ExpandedName),
    AnyAttribute,
    Comment,
    Text,
    ProcessingInstruction,
    ProcessingInstructionNamed(String),
    AnyNode,
    AnyElement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ChildPresenceTest {
    Element(ExpandedName),
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ApplySelection {
    AtomicIntegerRange {
        start: i64,
        end: i64,
    },
    LocationPath(LocationPath),
    PathUnion(Vec<LocationPath>),
    VariablePathUnion {
        variable: String,
        alternatives: Vec<LocationPath>,
    },
    SourceVariablePath {
        variable: String,
        path: LocationPath,
    },
    ChildElement(ExpandedName),
    DescendantElement(ExpandedName),
    ChildNodes(NodeTest),
    Attribute(ExpandedName),
    GlobalTemporaryChildren(String),
    VariableSequence(String),
    TemporaryPath {
        variable: String,
        steps: Vec<ExpandedName>,
    },
    VariableFilteredElementPath(VariableFilteredElementPath),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VariableFilteredElementPath {
    pub(crate) parent_steps: Vec<ExpandedName>,
    pub(crate) attribute: ExpandedName,
    pub(crate) variable: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeTest {
    Comment,
    ProcessingInstruction,
    AnyNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ElementConstructorOrigin {
    Literal,
    ComputedStatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SortDataType {
    Text,
    Number,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SortKey {
    pub(crate) select: SortSelect,
    pub(crate) data_type: SortDataType,
    pub(crate) xslt10_numeric_conversion: bool,
    pub(crate) order: SortOrder,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SortSelect {
    LocationPath(LocationPath),
    Literal(String),
    ContextPosition,
    ContextSize,
    ContextNodeName,
    ContextStringLength,
    CountPath(LocationPath),
    NumberPath(LocationPath),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Instruction {
    LiteralElement {
        origin: ElementConstructorOrigin,
        name: ExpandedName,
        namespaces: Arc<[NamespaceBinding]>,
        attributes: Vec<LiteralAttribute>,
        computed_attributes: Vec<ComputedAttribute>,
        body: Vec<Instruction>,
        location: SourceLocation,
    },
    Text {
        value: String,
        location: SourceLocation,
    },
    ProcessingInstructionNode {
        target: String,
        value: String,
        location: SourceLocation,
    },
    CommentNode {
        value: String,
        location: SourceLocation,
    },
    Attribute {
        attribute: ComputedAttribute,
        location: SourceLocation,
    },
    ValueOf {
        select: ValueExpression,
        separator: String,
        location: SourceLocation,
    },
    Number {
        value: Option<NumberValue>,
        level: NumberLevel,
        count: Option<NumberPattern>,
        from: Option<NumberPattern>,
        format: NumberFormat,
        location: SourceLocation,
    },
    Variable {
        name: String,
        select: Box<CastExpression>,
        location: SourceLocation,
    },
    StaticAtomicVariable {
        name: String,
        value: AtomicValue,
        location: SourceLocation,
    },
    AtomicVariableAlias {
        name: String,
        source: String,
        location: SourceLocation,
    },
    ContextPositionVariable {
        name: String,
        location: SourceLocation,
    },
    SourceNodeVariable {
        name: String,
        select: LocationPath,
        location: SourceLocation,
    },
    SourceNodeUnionVariable {
        name: String,
        sources: Vec<String>,
        location: SourceLocation,
    },
    IntegerRangeVariable {
        name: String,
        start: i64,
        end: i64,
        location: SourceLocation,
    },
    TemporaryTreeVariable {
        name: String,
        elements: Vec<ConstructedElement>,
        location: SourceLocation,
    },
    SequenceNodes {
        select: Box<ForDistinctValuesExpression>,
        location: SourceLocation,
    },
    SequenceItems {
        select: Vec<SequenceItemExpression>,
        location: SourceLocation,
    },
    ApplyTemplates {
        select: Option<ApplySelection>,
        sorts: Vec<SortKey>,
        mode: Option<String>,
        arguments: Vec<TemplateArgument>,
        location: SourceLocation,
    },
    ForEachVariable {
        variable: String,
        body: Vec<Instruction>,
        location: SourceLocation,
    },
    ForEachStaticIntegerRange {
        start: i64,
        end: i64,
        body: Vec<Instruction>,
        location: SourceLocation,
    },
    ForEachNodes {
        select: ApplySelection,
        sorts: Vec<SortKey>,
        body: Vec<Instruction>,
        location: SourceLocation,
    },
    NextMatch {
        arguments: Vec<TemplateArgument>,
        location: SourceLocation,
    },
    ApplyImports {
        arguments: Vec<TemplateArgument>,
        location: SourceLocation,
    },
    CopyOfCurrent {
        location: SourceLocation,
    },
    CopyOfChildElements {
        location: SourceLocation,
    },
    CopyOfAncestorOrSelfElements {
        location: SourceLocation,
    },
    CopyOfLocationPath {
        select: LocationPath,
        location: SourceLocation,
    },
    CopyOfPathUnion {
        alternatives: Vec<LocationPath>,
        location: SourceLocation,
    },
    CopyOfStaticAtomicText {
        value: String,
        location: SourceLocation,
    },
    CopyOfVariable {
        variable: String,
        location: SourceLocation,
    },
    If {
        test: BooleanExpression,
        body: Vec<Instruction>,
        location: SourceLocation,
    },
    Choose {
        branches: Vec<ChooseBranch>,
        otherwise: Vec<Instruction>,
        location: SourceLocation,
    },
    CallTemplate {
        name: String,
        arguments: Vec<TemplateArgument>,
        location: SourceLocation,
    },
    Copy {
        attributes: Vec<LiteralAttribute>,
        body: Vec<Instruction>,
        location: SourceLocation,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NumberValue {
    Literal(String),
    ContextPosition,
    ContextItem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NumberFormat {
    pub(crate) prefix: String,
    pub(crate) tokens: Vec<NumberFormatToken>,
    pub(crate) separators: Vec<String>,
    pub(crate) grouping: Option<NumberGrouping>,
    pub(crate) suffix: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NumberGrouping {
    pub(crate) separator: char,
    pub(crate) size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NumberFormatToken {
    pub(crate) minimum_width: usize,
    pub(crate) style: NumberTokenStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NumberTokenStyle {
    Decimal,
    AlphabeticUpper,
    AlphabeticLower,
    RomanUpper,
    RomanLower,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NumberPattern {
    Document,
    AnyNode,
    AnyElement,
    AnyAttribute,
    Element(ExpandedName),
    ElementWithAttributeValue {
        element: ExpandedName,
        attribute: ExpandedName,
        value: String,
    },
    ElementAtSiblingPosition {
        element: ExpandedName,
        predicate: NumberPositionPredicate,
    },
    ChildOf {
        parent: Box<NumberPattern>,
        child: Box<NumberPattern>,
    },
    Alternatives(Vec<NumberPattern>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NumberPositionPredicate {
    Exact(usize),
    Modulo { divisor: usize, remainder: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NumberLevel {
    Single,
    Multiple,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FocusEqualityOperand {
    Position,
    Size,
    CeilingHalfSize,
    Static(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FocusComparison {
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValueExpression {
    LiteralString(String),
    LocationPath(LocationPath),
    Xslt10FirstNodeLocationPath(LocationPath),
    Xslt10CurrentPredicatePath(LocationPath),
    CountLocationPath(LocationPath),
    CountSourceNodeVariable(String),
    RootPath(LocationPath),
    RootVariable(String),
    GeneratedNodeIdentity(LocationPath),
    GeneratedRootIdentity(LocationPath),
    GeneratedTemporaryRootIdentity {
        variable: String,
        descendant_local: Option<String>,
    },
    GeneratedDocumentRootIdentity(DocumentRootReference),
    NodeIdentityEqual {
        left: LocationPath,
        right: LocationPath,
    },
    ContextNodeName,
    NodeNamePath(LocationPath),
    ContextNodeLocalName,
    NodeLocalNamePath(LocationPath),
    ContextNodeNamespaceUri,
    NodeNamespaceUriPath(LocationPath),
    ContextLanguageMatches(String),
    ContextNodeNormalizedString,
    Xslt10NormalizedSourceNodeVariable(String),
    NormalizedStringPath(LocationPath),
    StringPath(LocationPath),
    IntegralFunctionPath {
        function: crate::xpath::constant_numeric_experiment::IntegralFunction,
        path: LocationPath,
    },
    NumberPath(LocationPath),
    BinaryNumeric(Box<BinaryNumericExpression>),
    ContextNodeStringLength(SourceLocation),
    ContextPosition(SourceLocation),
    ContextSize(SourceLocation),
    ContextFocusEquals {
        left: FocusEqualityOperand,
        right: FocusEqualityOperand,
        location: SourceLocation,
    },
    ContextRequiredOnly(SourceLocation),
    UpperCaseContextString,
    CaseConversion(Box<CaseConversionExpression>),
    Variable(String),
    LiteralVariableConcat {
        literal: String,
        variable: String,
    },
    IntegerFor(Box<IntegerForExpression>),
    FocusSumFor(Box<FocusSumForExpression>),
    DecimalSumFor(Box<DecimalSumForExpression>),
    FormatNumber(Box<FormatNumberExpression>),
    Castable(Box<CastableExpression>),
    DeepEqual(Box<DeepEqualBooleanExpression>),
    DefaultCollation(Box<DefaultCollationExpression>),
    DurationComponent(Box<DurationComponentExpression>),
    SequenceCardinality(Box<SequenceCardinalityExpression>),
    EmptyLocationPath(LocationPath),
    VariableEffectiveBooleanValue(String),
    Xslt10VariableString(String),
    Xslt10VariableNumber(String),
    Xslt10VariablePositionPath {
        path: LocationPath,
        variable: String,
    },
    Xslt10VariablePath {
        variable: String,
        path: LocationPath,
    },
    Xslt10PathStringFunction(Box<Xslt10PathStringFunction>),
    Xslt10SumPath(LocationPath),
    Xslt10PathSubstring(Box<Xslt10PathSubstring>),
    Xslt10PathTranslate(Box<Xslt10PathTranslate>),
    Xslt10Concat(Box<Xslt10ConcatExpression>),
    Xslt10VariableBooleanComparison {
        variable: String,
        value: bool,
        equal: bool,
    },
    Xslt10VariableStringComparison {
        variable: String,
        value: String,
        equal: bool,
        negate: bool,
    },
    Xslt10VariableStringVariablesComparison {
        left: String,
        right: String,
        equal: bool,
    },
    Xslt10VariableNumberComparison {
        variable: String,
        value: i32,
        equal: bool,
        negate: bool,
    },
    SourceFreeScalar(Box<ScalarExpression>),
    DocumentBoolean(Box<DocumentBooleanExpression>),
    EncodeForUri(Box<EncodeForUriExpression>),
    EscapeHtmlUri(Box<EscapeHtmlUriExpression>),
    IriToUri(Box<IriToUriExpression>),
    StringLength(Box<StringLengthExpression>),
    ConditionalInteger(Box<ConditionalIntegerExpression>),
    ConditionalPath(Box<ConditionalPathExpression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Xslt10PathStringFunctionKind {
    Contains,
    StartsWith,
    SubstringBefore,
    SubstringAfter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Xslt10PathStringFunction {
    pub(crate) kind: Xslt10PathStringFunctionKind,
    pub(crate) path: LocationPath,
    pub(crate) operand: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Xslt10PathSubstring {
    pub(crate) path: LocationPath,
    pub(crate) start_bits: u64,
    pub(crate) length_bits: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Xslt10PathTranslate {
    pub(crate) path: LocationPath,
    pub(crate) search: String,
    pub(crate) replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Xslt10ConcatExpression {
    pub(crate) parts: Vec<Xslt10ConcatPart>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Xslt10ConcatPart {
    Literal(String),
    Variable(String),
    Path(LocationPath),
    SumPath(LocationPath),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConditionalIntegerExpression {
    pub(crate) condition: ConditionalIntegerCondition,
    pub(crate) when_true: ConditionalIntegerBranch,
    pub(crate) when_false: ConditionalIntegerBranch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConditionalIntegerCondition {
    Constant(bool),
    Contains {
        path: Box<LocationPath>,
        needle: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConditionalIntegerBranch {
    Integer(i64),
    Conditional(Box<ConditionalIntegerExpression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConditionalPathExpression {
    pub(crate) condition: IntegerPathComparison,
    pub(crate) when_true: ConditionalPathBranch,
    pub(crate) when_false: ConditionalPathBranch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntegerPathComparison {
    pub(crate) left: LocationPath,
    pub(crate) right: LocationPath,
    pub(crate) operator: IntegerComparisonOperator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntegerComparisonOperator {
    Equal,
    GreaterThan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConditionalPathBranch {
    Path(LocationPath),
    Division {
        numerator: LocationPath,
        denominator: LocationPath,
    },
    Conditional(Box<ConditionalPathExpression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SequenceItemExpression {
    ChildElements,
    Variable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EqualityTest {
    pub(crate) variable: String,
    pub(crate) integer: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StringComparison {
    Codepoint,
    HtmlAsciiCaseInsensitive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BooleanExpression {
    VariableEqualsInteger(EqualityTest),
    VariableEqualsEmptySequence(String),
    VariableEffectiveBooleanValue(String),
    VariableStringEquals {
        left: String,
        right: String,
        comparison: StringComparison,
    },
    Xslt10VariableStringLiteralEquals {
        variable: String,
        literal: String,
        equal: bool,
    },
    ConditionalInteger(Box<ConditionalIntegerExpression>),
    NodeExists(LocationPath),
    NodeStringEquals {
        path: LocationPath,
        value: String,
    },
    NodeIntegerLessThan {
        path: LocationPath,
        value: i64,
    },
    CountPathEquals {
        path: LocationPath,
        expected: usize,
    },
    UnqualifiedNodeNameEquals {
        path: LocationPath,
        local: String,
        comparison: StringComparison,
    },
    ContextStringEquals(String),
    ContextStringLengthEquals(usize),
    ContextPositionNotEqualSize(SourceLocation),
    ContextFocusEquals {
        left: FocusEqualityOperand,
        right: FocusEqualityOperand,
        location: SourceLocation,
    },
    ContextFocusCompares {
        left: FocusEqualityOperand,
        operator: FocusComparison,
        right: FocusEqualityOperand,
        location: SourceLocation,
    },
    ContextLanguageMatches(String),
    Or {
        left: Box<BooleanExpression>,
        right: Box<BooleanExpression>,
    },
    And {
        left: Box<BooleanExpression>,
        right: Box<BooleanExpression>,
    },
    Not(Box<BooleanExpression>),
    NodeIdentityEqual {
        left: LocationPath,
        right: LocationPath,
    },
    RootIdentityEqualsVariable {
        path: LocationPath,
        variable: String,
    },
    TemporaryRootIdentityEqual {
        variable: String,
        descendant_local: String,
    },
    DocumentRootIdentityEqual {
        left: DocumentRootReference,
        right: DocumentRootReference,
    },
    Constant(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentRootReference {
    pub(crate) base: String,
    pub(crate) reference: String,
    pub(crate) descendant_local: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChooseBranch {
    pub(crate) test: BooleanExpression,
    pub(crate) body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemplateArgument {
    pub(crate) name: String,
    pub(crate) value: TemplateArgumentValue,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TemplateArgumentValue {
    Text(String),
    Integer(i64),
    Boolean(bool),
    ContextPosition,
    ContextSize,
    CurrentSourceNode,
    Variable(String),
    SourcePath(LocationPath),
    Xslt10SumPath(LocationPath),
    SourcePathStringComparison {
        left: LocationPath,
        right: Box<LocationPath>,
        equal: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiteralAttribute {
    pub(crate) name: ExpandedName,
    pub(crate) value: LiteralAttributeValue,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComputedAttribute {
    pub(crate) name: ExpandedName,
    pub(crate) value: LiteralAttributeValue,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LiteralAttributeValue {
    Text(String),
    Variable(String),
    CountSourceNodeVariable(String),
    Xslt10Concat(Box<Xslt10ConcatExpression>),
    SourceAttribute(ExpandedName),
    ContextPosition,
    ContextSize,
    ContextLocalName,
    ContextStringValue,
    ContextIntegerIncrement(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NamedSiblingBoundary {
    Exact(usize),
    Before(usize),
    BeforeLast,
    Last,
}

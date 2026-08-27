use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::castable_experiment::{CastExpression, CastableExpression};
use crate::xpath::decimal_sum_for_experiment::DecimalSumForExpression;
use crate::xpath::deep_equal_experiment::DeepEqualExpression;
use crate::xpath::focus_sum_for_experiment::FocusSumForExpression;
use crate::xpath::for_distinct_values_experiment::ForDistinctValuesExpression;
use crate::xpath::format_number_experiment::FormatNumberExpression;
use crate::xpath::integer_for_experiment::IntegerForExpression;
use crate::xpath::path_experiment::ChildPath;

pub(crate) const STANDARD_INITIAL_TEMPLATE_NAME: &str =
    "Q{http://www.w3.org/1999/XSL/Transform}initial-template";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StylesheetProgram {
    pub(crate) declared_version: String,
    pub(crate) output: OutputSettings,
    pub(crate) root_template: Option<Template>,
    pub(crate) root_template_modes: Vec<String>,
    pub(crate) matched_templates: Vec<MatchedTemplate>,
    pub(crate) named_templates: Vec<NamedTemplate>,
    pub(crate) global_bindings: Vec<GlobalBinding>,
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
    Text(String),
    ChildPath(ChildPath),
    Variable(String),
    TemporaryTree(Vec<ConstructedElement>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConstructedElement {
    pub(crate) name: ExpandedName,
    pub(crate) namespaces: Vec<NamespaceBinding>,
    pub(crate) children: Vec<ConstructedElement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutputSettings {
    pub(crate) method: Option<String>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MatchedTemplate {
    pub(crate) pattern: MatchPattern,
    pub(crate) modes: Vec<String>,
    pub(crate) template: Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NamedTemplate {
    pub(crate) name: String,
    pub(crate) parameters: Vec<String>,
    pub(crate) template: Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MatchPattern {
    Element(ExpandedName),
    Path(ChildPath),
    Attribute(ExpandedName),
    Comment,
    ProcessingInstruction,
    AnyNode,
    AnyElement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ApplySelection {
    ChildPath(ChildPath),
    ChildNodes(NodeTest),
    Attribute(ExpandedName),
    GlobalTemporaryChildren(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeTest {
    Comment,
    ProcessingInstruction,
    AnyNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Instruction {
    LiteralElement {
        name: ExpandedName,
        namespaces: Vec<NamespaceBinding>,
        body: Vec<Instruction>,
        location: SourceLocation,
    },
    Text {
        value: String,
        location: SourceLocation,
    },
    ValueOf {
        select: ValueExpression,
        separator: String,
        location: SourceLocation,
    },
    Variable {
        name: String,
        select: Box<CastExpression>,
        location: SourceLocation,
    },
    IntegerRangeVariable {
        name: String,
        start: i64,
        end: i64,
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
        mode: Option<String>,
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
        location: SourceLocation,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValueExpression {
    ChildPath(ChildPath),
    Variable(String),
    IntegerFor(Box<IntegerForExpression>),
    FocusSumFor(Box<FocusSumForExpression>),
    DecimalSumFor(Box<DecimalSumForExpression>),
    FormatNumber(Box<FormatNumberExpression>),
    Castable(Box<CastableExpression>),
    DeepEqual(Box<DeepEqualExpression>),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BooleanExpression {
    VariableEqualsInteger(EqualityTest),
    Constant(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChooseBranch {
    pub(crate) test: BooleanExpression,
    pub(crate) body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemplateArgument {
    pub(crate) name: String,
    pub(crate) value: String,
}

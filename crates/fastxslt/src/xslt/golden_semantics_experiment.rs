use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xpath::path_experiment::ChildPath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StylesheetProgram {
    pub(crate) declared_version: String,
    pub(crate) output: OutputSettings,
    pub(crate) root_template: Option<Template>,
    pub(crate) matched_templates: Vec<MatchedTemplate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutputSettings {
    pub(crate) method: Option<String>,
    pub(crate) omit_xml_declaration: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Template {
    pub(crate) body: Vec<Instruction>,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MatchedTemplate {
    pub(crate) pattern: MatchPattern,
    pub(crate) mode: Option<String>,
    pub(crate) template: Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MatchPattern {
    Element(ExpandedName),
    Attribute(ExpandedName),
    Comment,
    ProcessingInstruction,
    AnyNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ApplySelection {
    ChildPath(ChildPath),
    ChildNodes(NodeTest),
    Attribute(ExpandedName),
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
        body: Vec<Instruction>,
        location: SourceLocation,
    },
    Text {
        value: String,
        location: SourceLocation,
    },
    ValueOf {
        select: ChildPath,
        location: SourceLocation,
    },
    ApplyTemplates {
        select: Option<ApplySelection>,
        mode: Option<String>,
        location: SourceLocation,
    },
}

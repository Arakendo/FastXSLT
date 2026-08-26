use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xpath::path_experiment::ChildPath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StylesheetProgram {
    pub(crate) declared_version: String,
    pub(crate) output: OutputSettings,
    pub(crate) root_template: Template,
    pub(crate) element_templates: Vec<ElementTemplate>,
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
pub(crate) struct ElementTemplate {
    pub(crate) match_name: ExpandedName,
    pub(crate) template: Template,
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
        select: Option<ChildPath>,
        location: SourceLocation,
    },
}

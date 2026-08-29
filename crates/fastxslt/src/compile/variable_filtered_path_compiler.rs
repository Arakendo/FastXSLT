use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::VariableFilteredElementPath;

use super::is_ascii_ncname;

pub(super) fn parse(expression: &str) -> Option<VariableFilteredElementPath> {
    let (parents, predicate) = expression.split_once("/*[@")?;
    let predicate = predicate.strip_suffix(']')?;
    let (attribute, variable) = predicate.split_once("=$")?;
    let parent_steps = parents
        .split('/')
        .map(|step| {
            is_ascii_ncname(step).then(|| ExpandedName {
                namespace: None,
                local: step.to_owned(),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    if parent_steps.is_empty() || !is_ascii_ncname(attribute) || !is_ascii_ncname(variable) {
        return None;
    }
    Some(VariableFilteredElementPath {
        parent_steps,
        attribute: ExpandedName {
            namespace: None,
            local: attribute.to_owned(),
        },
        variable: variable.to_owned(),
    })
}

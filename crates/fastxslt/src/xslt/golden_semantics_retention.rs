use std::{mem::size_of, sync::Arc};

use super::{
    ApplySelection, AtomicValue, BooleanExpression, CastExpression, CastableExpression,
    CharacterMapDefinition, ChildPresenceTest, ChooseBranch, ComputedAttribute,
    ConditionalIntegerBranch, ConditionalIntegerCondition, ConditionalIntegerExpression,
    ConditionalPathBranch, ConditionalPathExpression, ConstructedAttribute, ConstructedElement,
    ConstructedNode, DecimalSumForExpression, DeepEqualBooleanExpression, ExpandedName,
    FocusSumForExpression, ForDistinctValuesExpression, FormatNumberExpression, GlobalBinding,
    GlobalBindingDefault, Instruction, IntegerForExpression, LiteralAttribute,
    LiteralAttributeValue, MatchPattern, MatchedTemplate, NamedTemplate, NamespaceBinding,
    OutputSettings, SequenceItemExpression, SortKey, SortSelect, SourceLocation, StylesheetProgram,
    Template, TemplateArgument, TemplateArgumentValue, TemplateParameter, TemplateParameterDefault,
    ValueExpression, VariableFilteredElementPath, Xslt10ConcatPart,
};

impl StylesheetProgram {
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        self.declared_version.capacity()
            + option_string_owned(self.default_initial_mode.as_ref())
            + vec_owned(&self.typed_mode_requirements, |item| {
                item.name.capacity() + location_owned(&item.location)
            })
            + vec_owned(&self.private_initial_modes, |item| {
                item.name.capacity() + location_owned(&item.location)
            })
            + vec_owned(&self.mode_policies, |item| {
                option_string_owned(item.name.as_ref()) + location_owned(&item.location)
            })
            + output_owned(&self.output)
            + vec_owned(&self.output_specified_properties, String::capacity)
            + vec_owned(&self.character_maps, character_map_owned)
            + vec_owned(&self.output_character_map_names, name_owned)
            + self
                .output_character_map_location
                .as_ref()
                .map_or(0, location_owned)
            + self.root_template.as_ref().map_or(0, template_owned)
            + vec_owned(&self.root_template_modes, String::capacity)
            + vec_owned(&self.matched_templates, matched_template_owned)
            + vec_owned(&self.named_templates, named_template_owned)
            + vec_owned(&self.global_bindings, global_binding_owned)
    }
}

fn character_map_owned(value: &CharacterMapDefinition) -> usize {
    name_owned(&value.name)
        + vec_owned(&value.referenced_map_names, name_owned)
        + vec_owned(&value.entries, |(_, replacement)| replacement.capacity())
        + location_owned(&value.location)
}

fn vec_owned<T>(values: &Vec<T>, nested: impl Fn(&T) -> usize) -> usize {
    values.capacity() * size_of::<T>() + values.iter().map(nested).sum::<usize>()
}

fn option_string_owned(value: Option<&String>) -> usize {
    value.map_or(0, String::capacity)
}

fn location_owned(value: &SourceLocation) -> usize {
    value.resource.capacity()
}

fn name_owned(value: &ExpandedName) -> usize {
    value.local.capacity() + option_string_owned(value.namespace.as_ref())
}

fn namespace_owned(value: &NamespaceBinding) -> usize {
    option_string_owned(value.prefix.as_ref()) + value.namespace.capacity()
}

fn output_owned(value: &OutputSettings) -> usize {
    [
        &value.method,
        &value.version,
        &value.html_version,
        &value.encoding,
        &value.media_type,
        &value.doctype_system,
        &value.doctype_public,
        &value.normalization_form,
        &value.standalone,
    ]
    .into_iter()
    .map(|value| option_string_owned(value.as_ref()))
    .sum::<usize>()
        + vec_owned(&value.cdata_section_elements, name_owned)
        + vec_owned(&value.character_map, |(_, replacement)| {
            replacement.capacity()
        })
}

fn template_owned(value: &Template) -> usize {
    vec_owned(&value.parameters, template_parameter_owned)
        + vec_owned(&value.body, instruction_owned)
        + location_owned(&value.location)
}

fn template_parameter_owned(value: &TemplateParameter) -> usize {
    value.name.capacity()
        + match &value.default {
            TemplateParameterDefault::Text(text) => text.capacity(),
            TemplateParameterDefault::Integer(_) => 0,
        }
}

fn matched_template_owned(value: &MatchedTemplate) -> usize {
    match_pattern_owned(&value.pattern)
        + vec_owned(&value.modes, String::capacity)
        + template_owned(&value.template)
}

fn named_template_owned(value: &NamedTemplate) -> usize {
    value.name.capacity()
        + vec_owned(&value.parameters, String::capacity)
        + template_owned(&value.template)
}

fn global_binding_owned(value: &GlobalBinding) -> usize {
    value.name.capacity()
        + match &value.default {
            GlobalBindingDefault::Text(text) | GlobalBindingDefault::Variable(text) => {
                text.capacity()
            }
            GlobalBindingDefault::Atomic(value) => value.known_owned_capacity_bytes(),
            GlobalBindingDefault::EmptySequence | GlobalBindingDefault::Integer(_) => 0,
            GlobalBindingDefault::DoubleDivision {
                numerator,
                denominator,
            } => numerator.known_owned_capacity_bytes() + denominator.known_owned_capacity_bytes(),
            GlobalBindingDefault::CountLocationPath(path)
            | GlobalBindingDefault::LocationPath(path)
            | GlobalBindingDefault::SourceNodeIdentity(path) => path.known_owned_capacity_bytes(),
            GlobalBindingDefault::TemporaryTree(elements) => {
                vec_owned(elements, constructed_element_owned)
            }
            GlobalBindingDefault::TemporaryText(value)
            | GlobalBindingDefault::TemporaryComment(value) => value.capacity(),
            GlobalBindingDefault::Xslt10TemporarySourceString(path) => {
                path.known_owned_capacity_bytes()
            }
            GlobalBindingDefault::TemporaryAttribute { name, value } => {
                name_owned(name) + value.capacity()
            }
            GlobalBindingDefault::TemporaryProcessingInstruction { target, value } => {
                target.capacity() + value.capacity()
            }
        }
}

fn constructed_element_owned(value: &ConstructedElement) -> usize {
    name_owned(&value.name)
        + vec_owned(&value.namespaces, namespace_owned)
        + vec_owned(&value.attributes, constructed_attribute_owned)
        + vec_owned(&value.children, constructed_node_owned)
}

fn constructed_attribute_owned(value: &ConstructedAttribute) -> usize {
    name_owned(&value.name) + value.value.capacity()
}

fn constructed_node_owned(value: &ConstructedNode) -> usize {
    match value {
        ConstructedNode::Element(element) => constructed_element_owned(element),
        ConstructedNode::Text(text) => text.capacity(),
    }
}

fn match_pattern_owned(value: &MatchPattern) -> usize {
    match value {
        MatchPattern::AtomicIntegerGreaterOrEqual(_)
        | MatchPattern::Document
        | MatchPattern::DescendantAnyElement
        | MatchPattern::ElementWithSameNamedChild
        | MatchPattern::ElementWithSameNamedParent
        | MatchPattern::ElementWithSameNamedParentAtPosition(_)
        | MatchPattern::Comment
        | MatchPattern::Text
        | MatchPattern::ProcessingInstruction
        | MatchPattern::AnyNode
        | MatchPattern::AnyElement
        | MatchPattern::AnyAttribute => 0,
        MatchPattern::DocumentElement(name) => name.as_ref().map_or(0, name_owned),
        MatchPattern::Element(name) | MatchPattern::Attribute(name) => name_owned(name),
        MatchPattern::ProcessingInstructionNamed(target) => target.capacity(),
        MatchPattern::ElementLocal(value) | MatchPattern::ElementNamespace(value) => {
            value.capacity()
        }
        MatchPattern::ElementWithAttribute { element, attribute } => {
            name_owned(element) + name_owned(attribute)
        }
        MatchPattern::ElementWithAttributeValue {
            element,
            attribute,
            value,
        } => name_owned(element) + name_owned(attribute) + value.capacity(),
        MatchPattern::ElementWithChild { element, child } => {
            name_owned(element)
                + match child {
                    ChildPresenceTest::Element(name) => name_owned(name),
                    ChildPresenceTest::Text => 0,
                }
        }
        MatchPattern::AnyElementWithAttributeVariable {
            attribute,
            variable,
        } => name_owned(attribute) + variable.capacity(),
        MatchPattern::VariableFilteredElementPath(path) => variable_filtered_path_owned(path),
        MatchPattern::ElementAtNamedSiblingBoundary { element, .. } => name_owned(element),
        MatchPattern::QualifiedElementPathAlternatives(paths) => {
            vec_owned(paths, |path| vec_owned(path, name_owned))
        }
        MatchPattern::UnionAlternatives(patterns) => vec_owned(patterns, match_pattern_owned),
        MatchPattern::Path(path) => path.known_owned_capacity_bytes(),
    }
}

fn variable_filtered_path_owned(value: &VariableFilteredElementPath) -> usize {
    vec_owned(&value.parent_steps, name_owned)
        + name_owned(&value.attribute)
        + value.variable.capacity()
}

fn apply_selection_owned(value: &ApplySelection) -> usize {
    match value {
        ApplySelection::LocationPath(path) => path.known_owned_capacity_bytes(),
        ApplySelection::PathUnion(alternatives) => vec_owned(
            alternatives,
            crate::xpath::path_experiment::LocationPath::known_owned_capacity_bytes,
        ),
        ApplySelection::VariablePathUnion {
            variable,
            alternatives,
        } => {
            variable.capacity()
                + vec_owned(
                    alternatives,
                    crate::xpath::path_experiment::LocationPath::known_owned_capacity_bytes,
                )
        }
        ApplySelection::ChildElement(name)
        | ApplySelection::DescendantElement(name)
        | ApplySelection::Attribute(name) => name_owned(name),
        ApplySelection::AtomicIntegerRange { .. } | ApplySelection::ChildNodes(_) => 0,
        ApplySelection::GlobalTemporaryChildren(name) | ApplySelection::VariableSequence(name) => {
            name.capacity()
        }
        ApplySelection::TemporaryPath { variable, steps } => {
            variable.capacity() + vec_owned(steps, name_owned)
        }
        ApplySelection::VariableFilteredElementPath(path) => variable_filtered_path_owned(path),
    }
}

fn instruction_owned(value: &Instruction) -> usize {
    match value {
        Instruction::LiteralElement { .. } => literal_element_instruction_owned(value),
        Instruction::Text { value, location } | Instruction::CommentNode { value, location } => {
            value.capacity() + location_owned(location)
        }
        Instruction::ProcessingInstructionNode {
            target,
            value,
            location,
        } => target.capacity() + value.capacity() + location_owned(location),
        Instruction::Attribute {
            attribute,
            location,
        } => computed_attribute_owned(attribute) + location_owned(location),
        Instruction::ValueOf {
            select,
            separator,
            location,
        } => value_expression_owned(select) + separator.capacity() + location_owned(location),
        instruction @ Instruction::Number { .. } => number_instruction_owned(instruction),
        Instruction::Variable {
            name,
            select,
            location,
        } => local_atomic_variable_owned(name, select, location),
        instruction @ (Instruction::StaticAtomicVariable { .. }
        | Instruction::AtomicVariableAlias { .. }) => scalar_binding_owned(instruction),
        Instruction::ContextPositionVariable { name, location }
        | Instruction::IntegerRangeVariable { name, location, .. } => {
            name.capacity() + location_owned(location)
        }
        Instruction::SourceNodeVariable {
            name,
            select,
            location,
        } => name.capacity() + select.known_owned_capacity_bytes() + location_owned(location),
        Instruction::TemporaryTreeVariable {
            name,
            elements,
            location,
        } => {
            name.capacity()
                + vec_owned(elements, constructed_element_owned)
                + location_owned(location)
        }
        Instruction::SequenceNodes { select, location } => {
            size_of::<ForDistinctValuesExpression>()
                + select.known_owned_capacity_bytes()
                + location_owned(location)
        }
        Instruction::SequenceItems { select, location } => {
            vec_owned(select, sequence_item_owned) + location_owned(location)
        }
        instruction @ Instruction::ApplyTemplates { .. } => {
            apply_templates_instruction_owned(instruction)
        }
        Instruction::ForEachVariable { .. }
        | Instruction::ForEachStaticIntegerRange { .. }
        | Instruction::ForEachNodes { .. } => for_each_owned(value),
        Instruction::NextMatch {
            arguments,
            location,
        }
        | Instruction::ApplyImports {
            arguments,
            location,
        } => vec_owned(arguments, template_argument_owned) + location_owned(location),
        instruction @ (Instruction::CopyOfCurrent { .. }
        | Instruction::CopyOfChildElements { .. }
        | Instruction::CopyOfAncestorOrSelfElements { .. }
        | Instruction::CopyOfLocationPath { .. }
        | Instruction::CopyOfPathUnion { .. }
        | Instruction::CopyOfStaticAtomicText { .. }
        | Instruction::CopyOfVariable { .. }) => copy_of_owned(instruction),
        Instruction::If {
            test,
            body,
            location,
        } => conditional_owned(test, body, location),
        Instruction::Choose {
            branches,
            otherwise,
            location,
        } => choose_owned(branches, otherwise, location),
        Instruction::CallTemplate {
            name,
            arguments,
            location,
        } => call_template_owned(name, arguments, location),
        Instruction::Copy {
            attributes,
            body,
            location,
        } => copy_owned(attributes, body, location),
    }
}

fn scalar_binding_owned(value: &Instruction) -> usize {
    match value {
        Instruction::StaticAtomicVariable {
            name,
            value,
            location,
        } => static_atomic_variable_owned(name, value, location),
        Instruction::AtomicVariableAlias {
            name,
            source,
            location,
        } => name.capacity() + source.capacity() + location_owned(location),
        _ => unreachable!("scalar binding accounting receives one scalar binding"),
    }
}

fn static_atomic_variable_owned(
    name: &String,
    value: &AtomicValue,
    location: &SourceLocation,
) -> usize {
    name.capacity() + value.known_owned_capacity_bytes() + location_owned(location)
}

fn local_atomic_variable_owned(
    name: &String,
    select: &CastExpression,
    location: &SourceLocation,
) -> usize {
    name.capacity()
        + size_of::<CastExpression>()
        + select.known_owned_capacity_bytes()
        + location_owned(location)
}

fn number_value_owned(value: Option<&super::NumberValue>) -> usize {
    value.map_or(0, |value| match value {
        super::NumberValue::Literal(value) => value.capacity(),
        super::NumberValue::ContextPosition | super::NumberValue::ContextItem => 0,
    })
}

fn number_instruction_owned(instruction: &Instruction) -> usize {
    let Instruction::Number {
        value,
        level: _,
        count,
        from,
        format,
        location,
    } = instruction
    else {
        unreachable!("number accounting receives only number instructions")
    };
    number_value_owned(value.as_ref())
        + count.as_ref().map_or(0, number_pattern_owned)
        + from.as_ref().map_or(0, number_pattern_owned)
        + format.prefix.capacity()
        + vec_owned(&format.tokens, |_| 0)
        + vec_owned(&format.separators, String::capacity)
        + format.suffix.capacity()
        + location_owned(location)
}

fn number_pattern_owned(pattern: &super::NumberPattern) -> usize {
    match pattern {
        super::NumberPattern::Element(name) => name_owned(name),
        super::NumberPattern::ElementWithAttributeValue {
            element,
            attribute,
            value,
        } => name_owned(element) + name_owned(attribute) + value.capacity(),
        super::NumberPattern::ElementAtSiblingPosition { element, .. } => name_owned(element),
        super::NumberPattern::ChildOf { parent, child } => {
            size_of::<super::NumberPattern>()
                + number_pattern_owned(parent)
                + size_of::<super::NumberPattern>()
                + number_pattern_owned(child)
        }
        super::NumberPattern::Alternatives(alternatives) => {
            vec_owned(alternatives, number_pattern_owned)
        }
        super::NumberPattern::Document
        | super::NumberPattern::AnyNode
        | super::NumberPattern::AnyElement
        | super::NumberPattern::AnyAttribute => 0,
    }
}

fn apply_templates_instruction_owned(instruction: &Instruction) -> usize {
    let Instruction::ApplyTemplates {
        select,
        sorts,
        mode,
        arguments,
        location,
    } = instruction
    else {
        unreachable!("apply-templates accounting receives only apply-templates instructions")
    };
    apply_templates_owned(select.as_ref(), mode.as_ref(), arguments, location)
        + vec_owned(sorts, sort_key_owned)
}

fn copy_of_owned(instruction: &Instruction) -> usize {
    match instruction {
        Instruction::CopyOfCurrent { location }
        | Instruction::CopyOfChildElements { location }
        | Instruction::CopyOfAncestorOrSelfElements { location } => location_owned(location),
        Instruction::CopyOfLocationPath { select, location } => {
            select.known_owned_capacity_bytes() + location_owned(location)
        }
        Instruction::CopyOfPathUnion {
            alternatives,
            location,
        } => {
            vec_owned(
                alternatives,
                crate::xpath::path_experiment::LocationPath::known_owned_capacity_bytes,
            ) + location_owned(location)
        }
        Instruction::CopyOfStaticAtomicText { value, location } => {
            value.capacity() + location_owned(location)
        }
        Instruction::CopyOfVariable { variable, location } => {
            variable.capacity() + location_owned(location)
        }
        _ => unreachable!("copy-of retention receives one copy-of instruction"),
    }
}

fn literal_element_instruction_owned(value: &Instruction) -> usize {
    let Instruction::LiteralElement {
        origin: _,
        name,
        namespaces,
        attributes,
        computed_attributes,
        body,
        location,
    } = value
    else {
        unreachable!("literal-element retention requires a literal-element instruction")
    };
    literal_element_owned(
        name,
        namespaces,
        attributes,
        computed_attributes,
        body,
        location,
    )
}

fn for_each_owned(value: &Instruction) -> usize {
    match value {
        Instruction::ForEachVariable {
            variable,
            body,
            location,
        } => variable.capacity() + vec_owned(body, instruction_owned) + location_owned(location),
        Instruction::ForEachStaticIntegerRange { body, location, .. } => {
            vec_owned(body, instruction_owned) + location_owned(location)
        }
        Instruction::ForEachNodes {
            select,
            sorts,
            body,
            location,
        } => {
            apply_selection_owned(select)
                + vec_owned(sorts, sort_key_owned)
                + vec_owned(body, instruction_owned)
                + location_owned(location)
        }
        _ => unreachable!("for-each accounting receives only for-each instructions"),
    }
}

fn sort_key_owned(sort: &SortKey) -> usize {
    let select = match &sort.select {
        SortSelect::LocationPath(path)
        | SortSelect::CountPath(path)
        | SortSelect::NumberPath(path) => path.known_owned_capacity_bytes(),
        SortSelect::Literal(value) => value.capacity(),
        SortSelect::ContextPosition
        | SortSelect::ContextSize
        | SortSelect::ContextNodeName
        | SortSelect::ContextStringLength => 0,
    };
    select + location_owned(&sort.location)
}

fn apply_templates_owned(
    select: Option<&ApplySelection>,
    mode: Option<&String>,
    arguments: &Vec<TemplateArgument>,
    location: &SourceLocation,
) -> usize {
    select.map_or(0, apply_selection_owned)
        + option_string_owned(mode)
        + vec_owned(arguments, template_argument_owned)
        + location_owned(location)
}

fn conditional_owned(
    test: &BooleanExpression,
    body: &Vec<Instruction>,
    location: &SourceLocation,
) -> usize {
    boolean_expression_owned(test) + vec_owned(body, instruction_owned) + location_owned(location)
}

fn choose_owned(
    branches: &Vec<ChooseBranch>,
    otherwise: &Vec<Instruction>,
    location: &SourceLocation,
) -> usize {
    vec_owned(branches, choose_branch_owned)
        + vec_owned(otherwise, instruction_owned)
        + location_owned(location)
}

fn call_template_owned(
    name: &String,
    arguments: &Vec<TemplateArgument>,
    location: &SourceLocation,
) -> usize {
    name.capacity() + vec_owned(arguments, template_argument_owned) + location_owned(location)
}

fn copy_owned(
    attributes: &Vec<LiteralAttribute>,
    body: &Vec<Instruction>,
    location: &SourceLocation,
) -> usize {
    vec_owned(attributes, literal_attribute_owned)
        + vec_owned(body, instruction_owned)
        + location_owned(location)
}

fn literal_element_owned(
    name: &ExpandedName,
    namespaces: &Arc<[NamespaceBinding]>,
    attributes: &Vec<LiteralAttribute>,
    computed_attributes: &Vec<ComputedAttribute>,
    body: &Vec<Instruction>,
    location: &SourceLocation,
) -> usize {
    name_owned(name)
        + arc_slice_owned(namespaces, namespace_owned)
        + vec_owned(attributes, literal_attribute_owned)
        + vec_owned(computed_attributes, computed_attribute_owned)
        + vec_owned(body, instruction_owned)
        + location_owned(location)
}

fn arc_slice_owned<T>(values: &Arc<[T]>, nested: impl Fn(&T) -> usize) -> usize {
    2 * size_of::<usize>()
        + values.len() * size_of::<T>()
        + values.iter().map(nested).sum::<usize>()
}

#[expect(
    clippy::too_many_lines,
    reason = "the exhaustive prepared-value ownership accounting is one cohesive responsibility"
)]
fn value_expression_owned(value: &ValueExpression) -> usize {
    match value {
        ValueExpression::LiteralString(value) => value.capacity(),
        ValueExpression::LocationPath(path)
        | ValueExpression::Xslt10FirstNodeLocationPath(path)
        | ValueExpression::CountLocationPath(path)
        | ValueExpression::RootPath(path)
        | ValueExpression::NodeNamePath(path)
        | ValueExpression::NodeLocalNamePath(path)
        | ValueExpression::NodeNamespaceUriPath(path)
        | ValueExpression::NormalizedStringPath(path)
        | ValueExpression::StringPath(path)
        | ValueExpression::GeneratedNodeIdentity(path)
        | ValueExpression::GeneratedRootIdentity(path)
        | ValueExpression::EmptyLocationPath(path)
        | ValueExpression::NumberPath(path)
        | ValueExpression::Xslt10SumPath(path)
        | ValueExpression::IntegralFunctionPath { path, .. } => path.known_owned_capacity_bytes(),
        ValueExpression::BinaryNumeric(expression) => {
            size_of_val(expression.as_ref()) + expression.known_owned_capacity_bytes()
        }
        ValueExpression::ContextNodeName
        | ValueExpression::ContextNodeLocalName
        | ValueExpression::ContextNodeNamespaceUri
        | ValueExpression::ContextNodeNormalizedString
        | ValueExpression::UpperCaseContextString => 0,
        ValueExpression::ContextLanguageMatches(language) => language.capacity(),
        ValueExpression::ContextNodeStringLength(location)
        | ValueExpression::ContextPosition(location)
        | ValueExpression::ContextSize(location)
        | ValueExpression::ContextRequiredOnly(location)
        | ValueExpression::ContextFocusEquals { location, .. } => location.resource.capacity(),
        ValueExpression::CaseConversion(expression) => {
            size_of_val(expression.as_ref()) + expression.known_owned_capacity_bytes()
        }
        ValueExpression::Variable(name)
        | ValueExpression::RootVariable(name)
        | ValueExpression::VariableEffectiveBooleanValue(name)
        | ValueExpression::Xslt10VariableString(name)
        | ValueExpression::Xslt10VariableNumber(name) => name.capacity(),
        ValueExpression::Xslt10VariablePositionPath { path, variable } => {
            path.known_owned_capacity_bytes() + variable.capacity()
        }
        ValueExpression::Xslt10VariablePath { variable, path } => {
            variable.capacity() + path.known_owned_capacity_bytes()
        }
        ValueExpression::Xslt10VariableBooleanComparison { variable, .. }
        | ValueExpression::Xslt10VariableNumberComparison { variable, .. } => variable.capacity(),
        ValueExpression::Xslt10VariableStringComparison {
            variable, value, ..
        } => variable.capacity() + value.capacity(),
        ValueExpression::Xslt10VariableStringVariablesComparison { left, right, .. } => {
            left.capacity() + right.capacity()
        }
        ValueExpression::Xslt10PathStringFunction(expression) => {
            size_of_val(expression.as_ref())
                + expression.path.known_owned_capacity_bytes()
                + expression.operand.capacity()
        }
        ValueExpression::Xslt10PathSubstring(expression) => {
            size_of_val(expression.as_ref()) + expression.path.known_owned_capacity_bytes()
        }
        ValueExpression::Xslt10PathTranslate(expression) => {
            size_of_val(expression.as_ref())
                + expression.path.known_owned_capacity_bytes()
                + expression.search.capacity()
                + expression.replacement.capacity()
        }
        ValueExpression::Xslt10Concat(expression) => {
            size_of_val(expression.as_ref())
                + vec_owned(&expression.parts, |part| match part {
                    Xslt10ConcatPart::Literal(value) | Xslt10ConcatPart::Variable(value) => {
                        value.capacity()
                    }
                    Xslt10ConcatPart::Path(path) | Xslt10ConcatPart::SumPath(path) => {
                        path.known_owned_capacity_bytes()
                    }
                })
        }
        ValueExpression::LiteralVariableConcat { literal, variable } => {
            literal.capacity() + variable.capacity()
        }
        ValueExpression::GeneratedTemporaryRootIdentity {
            variable,
            descendant_local,
        } => variable.capacity() + descendant_local.as_ref().map_or(0, String::capacity),
        ValueExpression::GeneratedDocumentRootIdentity(reference) => {
            reference.base.capacity()
                + reference.reference.capacity()
                + reference
                    .descendant_local
                    .as_ref()
                    .map_or(0, String::capacity)
        }
        ValueExpression::NodeIdentityEqual { left, right } => path_pair_owned(left, right),
        ValueExpression::IntegerFor(expression) => {
            size_of::<IntegerForExpression>() + expression.known_owned_capacity_bytes()
        }
        ValueExpression::FocusSumFor(expression) => {
            size_of::<FocusSumForExpression>() + expression.known_owned_capacity_bytes()
        }
        ValueExpression::DecimalSumFor(expression) => {
            size_of::<DecimalSumForExpression>() + expression.known_owned_capacity_bytes()
        }
        ValueExpression::FormatNumber(expression) => {
            size_of::<FormatNumberExpression>() + expression.known_owned_capacity_bytes()
        }
        ValueExpression::Castable(expression) => {
            size_of::<CastableExpression>() + expression.known_owned_capacity_bytes()
        }
        ValueExpression::DeepEqual(expression) => {
            size_of::<DeepEqualBooleanExpression>() + expression.known_owned_capacity_bytes()
        }
        ValueExpression::DefaultCollation(expression) => {
            size_of_val(expression.as_ref()) + match expression.as_ref() {
                crate::xpath::default_collation_experiment::DefaultCollationExpression::Equals(
                    expected,
                ) => expected.capacity(),
                crate::xpath::default_collation_experiment::DefaultCollationExpression::Value
                | crate::xpath::default_collation_experiment::DefaultCollationExpression::Count
                | crate::xpath::default_collation_experiment::DefaultCollationExpression::Boolean => {
                    0
                }
            }
        }
        ValueExpression::DurationComponent(expression) => {
            size_of_val(expression.as_ref()) + expression.known_owned_capacity_bytes()
        }
        ValueExpression::SequenceCardinality(expression) => {
            size_of_val(expression.as_ref()) + expression.known_owned_capacity_bytes()
        }
        ValueExpression::SourceFreeScalar(expression) => {
            size_of_val(expression.as_ref()) + expression.known_owned_capacity_bytes()
        }
        ValueExpression::DocumentBoolean(expression) => {
            size_of_val(expression.as_ref()) + expression.known_owned_capacity_bytes()
        }
        ValueExpression::EncodeForUri(expression) => {
            size_of_val(expression.as_ref()) + expression.known_owned_capacity_bytes()
        }
        ValueExpression::EscapeHtmlUri(expression) => {
            size_of_val(expression.as_ref()) + expression.known_owned_capacity_bytes()
        }
        ValueExpression::IriToUri(expression) => {
            size_of_val(expression.as_ref()) + expression.known_owned_capacity_bytes()
        }
        ValueExpression::StringLength(expression) => {
            size_of_val(expression.as_ref()) + expression.known_owned_capacity_bytes()
        }
        ValueExpression::ConditionalInteger(expression) => conditional_integer_owned(expression),
        ValueExpression::ConditionalPath(expression) => conditional_path_owned(expression),
    }
}

fn sequence_item_owned(value: &SequenceItemExpression) -> usize {
    match value {
        SequenceItemExpression::ChildElements => 0,
        SequenceItemExpression::Variable(name) => name.capacity(),
    }
}

fn boolean_expression_owned(value: &BooleanExpression) -> usize {
    match value {
        BooleanExpression::VariableEqualsInteger(test) => test.variable.capacity(),
        BooleanExpression::VariableEqualsEmptySequence(variable)
        | BooleanExpression::VariableEffectiveBooleanValue(variable) => variable.capacity(),
        BooleanExpression::VariableStringEquals {
            left,
            right,
            comparison: _,
        } => left.capacity() + right.capacity(),
        BooleanExpression::Xslt10VariableStringLiteralEquals {
            variable,
            literal,
            equal: _,
        } => variable.capacity() + literal.capacity(),
        BooleanExpression::ConditionalInteger(expression) => conditional_integer_owned(expression),
        BooleanExpression::NodeExists(path)
        | BooleanExpression::NodeIntegerLessThan { path, .. }
        | BooleanExpression::CountPathEquals { path, .. } => path.known_owned_capacity_bytes(),
        BooleanExpression::NodeStringEquals { path, value } => {
            path.known_owned_capacity_bytes() + value.capacity()
        }
        BooleanExpression::UnqualifiedNodeNameEquals {
            path,
            local,
            comparison: _,
        } => path.known_owned_capacity_bytes() + local.capacity(),
        BooleanExpression::ContextStringEquals(value)
        | BooleanExpression::ContextLanguageMatches(value) => value.capacity(),
        BooleanExpression::ContextPositionNotEqualSize(location)
        | BooleanExpression::ContextFocusEquals { location, .. }
        | BooleanExpression::ContextFocusCompares { location, .. } => location_owned(location),
        BooleanExpression::Or { left, right } | BooleanExpression::And { left, right } => {
            boolean_expression_owned(left) + boolean_expression_owned(right)
        }
        BooleanExpression::Not(expression) => boolean_expression_owned(expression),
        BooleanExpression::NodeIdentityEqual { left, right } => path_pair_owned(left, right),
        BooleanExpression::RootIdentityEqualsVariable { path, variable } => {
            path.known_owned_capacity_bytes() + variable.capacity()
        }
        BooleanExpression::TemporaryRootIdentityEqual {
            variable,
            descendant_local,
        } => variable.capacity() + descendant_local.capacity(),
        BooleanExpression::DocumentRootIdentityEqual { left, right } => {
            left.base.capacity()
                + left.reference.capacity()
                + left.descendant_local.as_ref().map_or(0, String::capacity)
                + right.base.capacity()
                + right.reference.capacity()
                + right.descendant_local.as_ref().map_or(0, String::capacity)
        }
        BooleanExpression::ContextStringLengthEquals(_) | BooleanExpression::Constant(_) => 0,
    }
}

fn path_pair_owned(
    left: &crate::xpath::path_experiment::LocationPath,
    right: &crate::xpath::path_experiment::LocationPath,
) -> usize {
    left.known_owned_capacity_bytes() + right.known_owned_capacity_bytes()
}

fn conditional_integer_owned(value: &ConditionalIntegerExpression) -> usize {
    let condition = match &value.condition {
        ConditionalIntegerCondition::Constant(_) => 0,
        ConditionalIntegerCondition::Contains { path, needle } => {
            path.known_owned_capacity_bytes() + needle.capacity()
        }
    };
    condition
        + conditional_integer_branch_owned(&value.when_true)
        + conditional_integer_branch_owned(&value.when_false)
}

fn conditional_integer_branch_owned(value: &ConditionalIntegerBranch) -> usize {
    match value {
        ConditionalIntegerBranch::Integer(_) => 0,
        ConditionalIntegerBranch::Conditional(expression) => conditional_integer_owned(expression),
    }
}

fn conditional_path_owned(value: &ConditionalPathExpression) -> usize {
    value.condition.left.known_owned_capacity_bytes()
        + value.condition.right.known_owned_capacity_bytes()
        + conditional_path_branch_owned(&value.when_true)
        + conditional_path_branch_owned(&value.when_false)
}

fn conditional_path_branch_owned(value: &ConditionalPathBranch) -> usize {
    match value {
        ConditionalPathBranch::Path(path) => path.known_owned_capacity_bytes(),
        ConditionalPathBranch::Division {
            numerator,
            denominator,
        } => numerator.known_owned_capacity_bytes() + denominator.known_owned_capacity_bytes(),
        ConditionalPathBranch::Conditional(expression) => conditional_path_owned(expression),
    }
}

fn choose_branch_owned(value: &ChooseBranch) -> usize {
    boolean_expression_owned(&value.test) + vec_owned(&value.body, instruction_owned)
}

fn template_argument_owned(value: &TemplateArgument) -> usize {
    value.name.capacity()
        + match &value.value {
            TemplateArgumentValue::Text(text) | TemplateArgumentValue::Variable(text) => {
                text.capacity()
            }
            TemplateArgumentValue::Integer(_)
            | TemplateArgumentValue::Boolean(_)
            | TemplateArgumentValue::ContextPosition
            | TemplateArgumentValue::ContextSize => 0,
            TemplateArgumentValue::SourcePath(path)
            | TemplateArgumentValue::Xslt10SumPath(path) => path.known_owned_capacity_bytes(),
            TemplateArgumentValue::SourcePathStringComparison { left, right, .. } => {
                path_pair_owned(left, right) + size_of_val(right.as_ref())
            }
        }
        + location_owned(&value.location)
}

fn literal_attribute_owned(value: &LiteralAttribute) -> usize {
    name_owned(&value.name)
        + literal_attribute_value_owned(&value.value)
        + location_owned(&value.location)
}

fn computed_attribute_owned(value: &ComputedAttribute) -> usize {
    name_owned(&value.name)
        + literal_attribute_value_owned(&value.value)
        + location_owned(&value.location)
}

fn literal_attribute_value_owned(value: &LiteralAttributeValue) -> usize {
    match value {
        LiteralAttributeValue::Text(text) | LiteralAttributeValue::Variable(text) => {
            text.capacity()
        }
        LiteralAttributeValue::Xslt10Concat(expression) => {
            size_of_val(expression.as_ref())
                + vec_owned(&expression.parts, |part| match part {
                    Xslt10ConcatPart::Literal(value) | Xslt10ConcatPart::Variable(value) => {
                        value.capacity()
                    }
                    Xslt10ConcatPart::Path(path) | Xslt10ConcatPart::SumPath(path) => {
                        path.known_owned_capacity_bytes()
                    }
                })
        }
        LiteralAttributeValue::SourceAttribute(name) => name_owned(name),
        LiteralAttributeValue::ContextPosition
        | LiteralAttributeValue::ContextSize
        | LiteralAttributeValue::ContextLocalName
        | LiteralAttributeValue::ContextStringValue
        | LiteralAttributeValue::ContextIntegerIncrement(_) => 0,
    }
}

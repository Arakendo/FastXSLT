use std::mem::size_of;

use super::{
    ApplySelection, BooleanExpression, CastExpression, CastableExpression, CharacterMapDefinition,
    ChildPresenceTest, ChooseBranch, ComputedAttribute, ConditionalIntegerBranch,
    ConditionalIntegerCondition, ConditionalIntegerExpression, ConditionalPathBranch,
    ConditionalPathExpression, ConstructedAttribute, ConstructedElement, ConstructedNode,
    DecimalSumForExpression, DeepEqualBooleanExpression, ExpandedName, FocusSumForExpression,
    ForDistinctValuesExpression, FormatNumberExpression, GlobalBinding, GlobalBindingDefault,
    Instruction, IntegerForExpression, LiteralAttribute, LiteralAttributeValue, MatchPattern,
    MatchedTemplate, NamedTemplate, NamespaceBinding, OutputSettings, SequenceItemExpression,
    SourceLocation, StylesheetProgram, Template, TemplateArgument, TemplateArgumentValue,
    TemplateParameter, TemplateParameterDefault, ValueExpression, VariableFilteredElementPath,
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
            GlobalBindingDefault::LocationPath(path)
            | GlobalBindingDefault::SourceNodeIdentity(path) => path.known_owned_capacity_bytes(),
            GlobalBindingDefault::TemporaryTree(elements) => {
                vec_owned(elements, constructed_element_owned)
            }
            GlobalBindingDefault::TemporaryText(value)
            | GlobalBindingDefault::TemporaryComment(value) => value.capacity(),
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
        ApplySelection::ChildElement(name)
        | ApplySelection::DescendantElement(name)
        | ApplySelection::Attribute(name) => name_owned(name),
        ApplySelection::AtomicIntegerRange { .. } | ApplySelection::ChildNodes(_) => 0,
        ApplySelection::GlobalTemporaryChildren(name) | ApplySelection::TemporaryRoot(name) => {
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
        Instruction::Variable {
            name,
            select,
            location,
        } => {
            name.capacity()
                + size_of::<CastExpression>()
                + select.known_owned_capacity_bytes()
                + location_owned(location)
        }
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
        Instruction::ApplyTemplates {
            select,
            mode,
            arguments,
            location,
        } => apply_templates_owned(select.as_ref(), mode.as_ref(), arguments, location),
        Instruction::ForEachTemporaryRoot { .. }
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
        Instruction::CopyOfCurrent { location } | Instruction::CopyOfChildElements { location } => {
            location_owned(location)
        }
        Instruction::CopyOfAncestorOrSelfElements { location } => location_owned(location),
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

fn literal_element_instruction_owned(value: &Instruction) -> usize {
    let Instruction::LiteralElement {
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
        Instruction::ForEachTemporaryRoot {
            variable,
            body,
            location,
        } => variable.capacity() + vec_owned(body, instruction_owned) + location_owned(location),
        Instruction::ForEachStaticIntegerRange { body, location, .. } => {
            vec_owned(body, instruction_owned) + location_owned(location)
        }
        Instruction::ForEachNodes {
            select,
            body,
            location,
        } => {
            apply_selection_owned(select)
                + vec_owned(body, instruction_owned)
                + location_owned(location)
        }
        _ => unreachable!("for-each accounting receives only for-each instructions"),
    }
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
    namespaces: &Vec<NamespaceBinding>,
    attributes: &Vec<LiteralAttribute>,
    computed_attributes: &Vec<ComputedAttribute>,
    body: &Vec<Instruction>,
    location: &SourceLocation,
) -> usize {
    name_owned(name)
        + vec_owned(namespaces, namespace_owned)
        + vec_owned(attributes, literal_attribute_owned)
        + vec_owned(computed_attributes, computed_attribute_owned)
        + vec_owned(body, instruction_owned)
        + location_owned(location)
}

fn value_expression_owned(value: &ValueExpression) -> usize {
    match value {
        ValueExpression::LiteralString(value) => value.capacity(),
        ValueExpression::LocationPath(path)
        | ValueExpression::CountLocationPath(path)
        | ValueExpression::RootPath(path)
        | ValueExpression::GeneratedRootIdentity(path) => path.known_owned_capacity_bytes(),
        ValueExpression::ContextNodeName | ValueExpression::UpperCaseContextString => 0,
        ValueExpression::CaseConversion(expression) => {
            size_of_val(expression.as_ref()) + expression.known_owned_capacity_bytes()
        }
        ValueExpression::Variable(name) | ValueExpression::RootVariable(name) => name.capacity(),
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
        BooleanExpression::ConditionalInteger(expression) => conditional_integer_owned(expression),
        BooleanExpression::NodeExists(path)
        | BooleanExpression::NodeIntegerLessThan { path, .. } => path.known_owned_capacity_bytes(),
        BooleanExpression::NodeStringEquals { path, value } => {
            path.known_owned_capacity_bytes() + value.capacity()
        }
        BooleanExpression::UnqualifiedNodeNameEquals {
            path,
            local,
            comparison: _,
        } => path.known_owned_capacity_bytes() + local.capacity(),
        BooleanExpression::ContextStringEquals(value) => value.capacity(),
        BooleanExpression::Or { left, right } => {
            boolean_expression_owned(left) + boolean_expression_owned(right)
        }
        BooleanExpression::Not(expression) => boolean_expression_owned(expression),
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
            TemplateArgumentValue::Integer(_) => 0,
            TemplateArgumentValue::SourcePath(path) => path.known_owned_capacity_bytes(),
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
        LiteralAttributeValue::ContextPosition
        | LiteralAttributeValue::ContextSize
        | LiteralAttributeValue::ContextLocalName
        | LiteralAttributeValue::ContextIntegerIncrement(_) => 0,
    }
}

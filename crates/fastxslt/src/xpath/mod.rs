//! `XPath` lexical, syntactic, and evaluation semantics.

#[cfg(any(test, feature = "workbench"))]
pub(crate) mod case_conversion_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod castable_experiment;
#[cfg(test)]
pub(crate) mod constant_boolean_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod constant_integer_experiment;
pub(crate) mod constant_numeric_experiment;
#[cfg(test)]
pub(crate) mod context_requirement_experiment;
#[cfg(test)]
pub(crate) mod count_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod decimal_sum_for_experiment;
#[cfg(any(test, feature = "workbench"))]
mod deep_equal_atomic;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod deep_equal_boolean_experiment;
#[cfg(any(test, feature = "workbench"))]
mod deep_equal_composite;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod deep_equal_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod default_collation_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod duration_component_experiment;
#[cfg(test)]
pub(crate) mod effective_boolean_value_experiment;
#[cfg(test)]
pub(crate) mod empty_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod encode_for_uri_expression;
pub(crate) mod escape_html_uri_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod escape_html_uri_expression;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod focus_sum_for_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod for_distinct_values_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod format_number_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod integer_for_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod iri_to_uri_expression;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod path_experiment;
#[cfg(test)]
pub(crate) mod path_operand_type_experiment;
#[cfg(test)]
mod qt3_axis_tests;
#[cfg(test)]
mod qt3_boolean_constant_tests;
#[cfg(test)]
mod qt3_codepoint_equal_tests;
#[cfg(test)]
mod qt3_deep_equal_tests;
#[cfg(test)]
mod qt3_default_collation_tests;
#[cfg(test)]
mod qt3_duration_component_tests;
#[cfg(test)]
mod qt3_empty_sequence_tests;
#[cfg(test)]
mod qt3_encode_for_uri_tests;
#[cfg(test)]
mod qt3_escape_html_uri_tests;
#[cfg(test)]
mod qt3_iri_to_uri_tests;
#[cfg(test)]
mod qt3_lower_case_tests;
#[cfg(test)]
mod qt3_normalize_space_tests;
#[cfg(test)]
mod qt3_production_path_test_support;
#[cfg(test)]
mod qt3_string_length_tests;
#[cfg(test)]
mod qt3_upper_case_tests;
pub(crate) mod static_string_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod string_length_experiment;

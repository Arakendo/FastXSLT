//! `XPath` lexical, syntactic, and evaluation semantics.

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
#[cfg(test)]
pub(crate) mod empty_experiment;
pub(crate) mod escape_html_uri_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod focus_sum_for_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod for_distinct_values_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod format_number_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod integer_for_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod path_experiment;
#[cfg(test)]
pub(crate) mod path_operand_type_experiment;
#[cfg(test)]
mod qt3_axis_tests;
#[cfg(test)]
mod qt3_boolean_constant_tests;
#[cfg(test)]
mod qt3_deep_equal_tests;
#[cfg(test)]
mod qt3_empty_sequence_tests;
pub(crate) mod static_string_experiment;

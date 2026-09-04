//! Exact-decimal `format-number(sum(for ...))` seam used by XSLT30 `for-004`.

use std::borrow::Cow;

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};

use super::path_experiment::{
    LocationPath, PathFailure, evaluate_location_path_controlled, parse_location_path,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecimalSumForExpression {
    binding_path: LocationPath,
    left_attribute: String,
    right_attribute: String,
}

#[cfg(feature = "workbench")]
impl DecimalSumForExpression {
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        self.binding_path.known_owned_capacity_bytes()
            + self.left_attribute.capacity()
            + self.right_attribute.capacity()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecimalSumForFailure {
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DecimalSumEvaluationFailure {
    Control(ControlFailure),
    InvalidValue,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExactDecimal {
    units: i128,
    scale: u32,
}

pub(crate) fn parse(
    expression: &str,
    location: &SourceLocation,
) -> Result<DecimalSumForExpression, DecimalSumForFailure> {
    let normalized = expression.split_whitespace().collect::<Vec<_>>().join(" ");
    let arguments = normalized
        .strip_prefix("format-number(")
        .and_then(|value| value.strip_suffix(')'))
        .map(str::trim)
        .ok_or_else(|| unsupported(&normalized, location))?;
    let (sum_expression, picture) =
        split_top_level_comma(arguments).ok_or_else(|| unsupported(&normalized, location))?;
    if picture.trim() != "'0.00'" {
        return Err(unsupported(&normalized, location));
    }
    let inner = sum_expression
        .trim()
        .strip_prefix("sum(for $")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| unsupported(&normalized, location))?;
    let (variable, after_variable) = inner
        .split_once(" in ")
        .ok_or_else(|| unsupported(&normalized, location))?;
    if !is_ascii_ncname(variable) {
        return Err(unsupported(&normalized, location));
    }
    let (binding_path, returned) = after_variable
        .split_once(" return ")
        .ok_or_else(|| unsupported(&normalized, location))?;
    let (left, right) = returned
        .split_once(" * ")
        .ok_or_else(|| unsupported(&normalized, location))?;
    let left_attribute =
        bound_attribute(left, variable).ok_or_else(|| unsupported(&normalized, location))?;
    let right_attribute =
        bound_attribute(right, variable).ok_or_else(|| unsupported(&normalized, location))?;
    let binding_path = parse_location_path(binding_path, location.clone()).map_err(|failure| {
        let detail = match failure {
            PathFailure::Invalid { detail, .. } | PathFailure::Unsupported { detail, .. } => detail,
        };
        DecimalSumForFailure {
            detail,
            location: location.clone(),
        }
    })?;

    Ok(DecimalSumForExpression {
        binding_path,
        left_attribute: left_attribute.to_owned(),
        right_attribute: right_attribute.to_owned(),
    })
}

pub(crate) fn evaluate(
    expression: &DecimalSumForExpression,
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<String, DecimalSumEvaluationFailure> {
    evaluate_using(
        expression,
        document,
        context,
        control,
        AttributeStringStrategy::Borrowed,
    )
}

#[derive(Clone, Copy)]
enum AttributeStringStrategy {
    Borrowed,
    #[cfg(test)]
    CompleteOwnedReference,
}

fn evaluate_using(
    expression: &DecimalSumForExpression,
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
    attribute_strings: AttributeStringStrategy,
) -> Result<String, DecimalSumEvaluationFailure> {
    let tuples =
        evaluate_location_path_controlled(document, context, &expression.binding_path, control)
            .map_err(DecimalSumEvaluationFailure::Control)?;
    let mut total = ExactDecimal { units: 0, scale: 0 };
    for bound_item in tuples {
        let Some(left_node) =
            find_attribute(document, bound_item, &expression.left_attribute, control)?
        else {
            continue;
        };
        let Some(right_node) =
            find_attribute(document, bound_item, &expression.right_attribute, control)?
        else {
            continue;
        };
        let left = attribute_decimal(document, left_node, control, attribute_strings)?;
        let right = attribute_decimal(document, right_node, control, attribute_strings)?;
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(DecimalSumEvaluationFailure::Control)?;
        let product = left
            .checked_mul(right)
            .ok_or(DecimalSumEvaluationFailure::Unsupported)?;
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(DecimalSumEvaluationFailure::Control)?;
        total = total
            .checked_add(product)
            .ok_or(DecimalSumEvaluationFailure::Unsupported)?;
    }
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(DecimalSumEvaluationFailure::Control)?;
    total
        .format_two_decimals()
        .ok_or(DecimalSumEvaluationFailure::Unsupported)
}

fn attribute_decimal(
    document: &Document,
    attribute: NodeId,
    control: &mut InvocationControl,
    strategy: AttributeStringStrategy,
) -> Result<ExactDecimal, DecimalSumEvaluationFailure> {
    let lexical = match strategy {
        AttributeStringStrategy::Borrowed => Cow::Borrowed(
            document
                .attribute_string_value_controlled(attribute, control)
                .map_err(DecimalSumEvaluationFailure::Control)?
                .ok_or(DecimalSumEvaluationFailure::InvalidValue)?,
        ),
        #[cfg(test)]
        AttributeStringStrategy::CompleteOwnedReference => Cow::Owned(
            document
                .string_value_controlled(attribute, control)
                .map_err(DecimalSumEvaluationFailure::Control)?,
        ),
    };
    let parsed = match strategy {
        AttributeStringStrategy::Borrowed => ExactDecimal::parse(&lexical),
        #[cfg(test)]
        AttributeStringStrategy::CompleteOwnedReference => {
            ExactDecimal::parse_complete_reference(&lexical)
        }
    };
    parsed.ok_or(DecimalSumEvaluationFailure::InvalidValue)
}

#[cfg(test)]
fn evaluate_complete_owned_reference(
    expression: &DecimalSumForExpression,
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<String, DecimalSumEvaluationFailure> {
    evaluate_using(
        expression,
        document,
        context,
        control,
        AttributeStringStrategy::CompleteOwnedReference,
    )
}

fn find_attribute(
    document: &Document,
    context: NodeId,
    local: &str,
    control: &mut InvocationControl,
) -> Result<Option<NodeId>, DecimalSumEvaluationFailure> {
    for attribute in document.attributes(context).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(DecimalSumEvaluationFailure::Control)?;
        if document
            .name(attribute)
            .is_some_and(|name| name.namespace.is_none() && name.local == local)
        {
            return Ok(Some(attribute));
        }
    }
    Ok(None)
}

impl ExactDecimal {
    fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        let (negative, unsigned) = value
            .strip_prefix('-')
            .map_or((false, value), |rest| (true, rest));
        let (whole, fractional) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        if whole.is_empty()
            || !whole.bytes().all(|byte| byte.is_ascii_digit())
            || !fractional.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        let scale = u32::try_from(fractional.len()).ok()?;
        let mut units =
            whole
                .bytes()
                .chain(fractional.bytes())
                .try_fold(0_i128, |value, digit| {
                    value
                        .checked_mul(10)?
                        .checked_add(i128::from(digit.checked_sub(b'0')?))
                })?;
        if negative {
            units = units.checked_neg()?;
        }
        Some(Self { units, scale })
    }

    #[cfg(test)]
    fn parse_complete_reference(value: &str) -> Option<Self> {
        let value = value.trim();
        let (negative, unsigned) = value
            .strip_prefix('-')
            .map_or((false, value), |rest| (true, rest));
        let (whole, fractional) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        if whole.is_empty()
            || !whole.bytes().all(|byte| byte.is_ascii_digit())
            || !fractional.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        let scale = u32::try_from(fractional.len()).ok()?;
        let digits = format!("{whole}{fractional}");
        let mut units = digits.parse::<i128>().ok()?;
        if negative {
            units = units.checked_neg()?;
        }
        Some(Self { units, scale })
    }

    fn checked_mul(self, other: Self) -> Option<Self> {
        Some(Self {
            units: self.units.checked_mul(other.units)?,
            scale: self.scale.checked_add(other.scale)?,
        })
    }

    fn checked_add(self, other: Self) -> Option<Self> {
        let scale = self.scale.max(other.scale);
        let left = self
            .units
            .checked_mul(checked_power_of_ten(scale.checked_sub(self.scale)?)?)?;
        let right = other
            .units
            .checked_mul(checked_power_of_ten(scale.checked_sub(other.scale)?)?)?;
        Some(Self {
            units: left.checked_add(right)?,
            scale,
        })
    }

    fn format_two_decimals(self) -> Option<String> {
        let units = match self.scale.cmp(&2) {
            std::cmp::Ordering::Less => self
                .units
                .checked_mul(checked_power_of_ten(2_u32.checked_sub(self.scale)?)?)?,
            std::cmp::Ordering::Equal => self.units,
            std::cmp::Ordering::Greater => {
                let divisor = checked_power_of_ten(self.scale.checked_sub(2)?)?;
                if self.units % divisor != 0 {
                    return None;
                }
                self.units / divisor
            }
        };
        let negative = units.is_negative();
        let magnitude = units.checked_abs()?;
        let whole = magnitude / 100;
        let fractional = magnitude % 100;
        Some(format!(
            "{}{whole}.{fractional:02}",
            if negative { "-" } else { "" }
        ))
    }
}

fn checked_power_of_ten(exponent: u32) -> Option<i128> {
    10_i128.checked_pow(exponent)
}

fn bound_attribute<'a>(operand: &'a str, variable: &str) -> Option<&'a str> {
    operand
        .trim()
        .strip_prefix('$')?
        .strip_prefix(variable)?
        .strip_prefix("/@")
        .filter(|name| is_ascii_ncname(name))
}

fn split_top_level_comma(value: &str) -> Option<(&str, &str)> {
    let mut depth = 0_u32;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth = depth.checked_add(1)?,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => return Some((&value[..index], &value[index + 1..])),
            _ => {}
        }
    }
    None
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    characters.next().is_some_and(|first| {
        (first.is_ascii_alphabetic() || first == '_')
            && characters.all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
            })
    })
}

fn unsupported(expression: &str, location: &SourceLocation) -> DecimalSumForFailure {
    DecimalSumForFailure {
        detail: format!(
            "the private slice supports one exact-decimal format-number(sum(for ...)) expression shape: {expression}"
        ),
        location: location.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{
        DecimalSumEvaluationFailure, ExactDecimal, evaluate, evaluate_complete_owned_reference,
        parse,
    };
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xdm::owned_tree_experiment::{Document, SourceLocation};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn expression() -> super::DecimalSumForExpression {
        parse(
            "format-number(sum(for $i in order-item return $i/@price * $i/@qty), '0.00')",
            &SourceLocation {
                resource: "memory:decimal-sum".to_owned(),
                span: 0..78,
            },
        )
        .expect("admitted decimal sum expression should parse")
    }

    #[test]
    fn multiplies_and_sums_exact_decimal_values() {
        let parsed = parse_document(
            "memory:order",
            b"<order><order-item price='11.32' qty='1'/><order-item price='2.34' qty='3'/><order-item price='1.00' qty='5'/><order-item price='2.56' qty='3'/><order-item price='5.00' qty='1'/></order>",
            ParseLimits {
                max_events: 32,
                max_depth: 4,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let order = document.children(document.document_node())[0];
        let mut control = InvocationControl::unbounded();
        let mut reference_control = InvocationControl::unbounded();

        let result = evaluate(&expression(), &document, order, &mut control)
            .expect("admitted exact decimal values should evaluate");
        let reference = evaluate_complete_owned_reference(
            &expression(),
            &document,
            order,
            &mut reference_control,
        )
        .expect("complete owned-string reference should evaluate");

        assert_eq!(result, "36.02");
        assert_eq!(result, reference);
        assert_eq!(control.consumed(WorkDomain::XPathOperation), 11);
        for domain in [
            WorkDomain::XPathOperation,
            WorkDomain::XPathNodeVisit,
            WorkDomain::XdmStringValueNode,
        ] {
            assert_eq!(control.consumed(domain), reference_control.consumed(domain));
        }
    }

    #[test]
    fn refuses_to_invent_rounding_semantics() {
        let parsed = parse_document(
            "memory:order",
            b"<order><order-item price='1.111' qty='1'/></order>",
            ParseLimits {
                max_events: 8,
                max_depth: 4,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let order = document.children(document.document_node())[0];

        assert_eq!(
            evaluate(
                &expression(),
                &document,
                order,
                &mut InvocationControl::unbounded()
            ),
            Err(DecimalSumEvaluationFailure::Unsupported)
        );
    }

    #[test]
    fn allocation_free_decimal_parser_matches_the_complete_reference() {
        for lexical in [
            "0",
            "1",
            "1.",
            "001.2300",
            "-0.01",
            "  42.50\t",
            "170141183460469231731687303715884105727",
            "170141183460469231731687303715884105728",
            "-170141183460469231731687303715884105728",
            "",
            ".1",
            "+1",
            "--1",
            "1.2.3",
            "NaN",
        ] {
            assert_eq!(
                ExactDecimal::parse(lexical),
                ExactDecimal::parse_complete_reference(lexical),
                "lexical {lexical:?}"
            );
        }
    }

    #[cfg(feature = "allocation-observation")]
    #[test]
    #[ignore = "manual allocator-observed borrowed versus owned attribute string probe"]
    fn measures_borrowed_attribute_string_specialization() {
        const ITEMS: usize = 500;
        const ITERATIONS: u32 = 1_000;
        let mut source = String::from("<order>");
        for _ in 0..ITEMS {
            source.push_str("<order-item price='1.00' qty='1'/>");
        }
        source.push_str("</order>");
        let parsed = parse_document(
            "memory:borrowed-attribute-measurement",
            source.as_bytes(),
            ParseLimits {
                max_events: 4_096,
                max_depth: 4,
            },
        )
        .expect("measurement source should parse");
        let document = Document::from_parsed(parsed).expect("measurement XDM should build");
        let order = document.children(document.document_node())[0];
        let expression = expression();

        let borrowed_allocations = allocation_counter::measure(|| {
            black_box(
                evaluate(
                    &expression,
                    &document,
                    order,
                    &mut InvocationControl::unbounded(),
                )
                .expect("borrowed evaluation"),
            );
        });
        let reference_allocations = allocation_counter::measure(|| {
            black_box(
                evaluate_complete_owned_reference(
                    &expression,
                    &document,
                    order,
                    &mut InvocationControl::unbounded(),
                )
                .expect("owned reference evaluation"),
            );
        });

        let started = Instant::now();
        for _ in 0..ITERATIONS {
            black_box(
                evaluate(
                    &expression,
                    &document,
                    order,
                    &mut InvocationControl::unbounded(),
                )
                .expect("borrowed evaluation"),
            );
        }
        let borrowed_microseconds =
            started.elapsed().as_secs_f64() * 1_000_000.0 / f64::from(ITERATIONS);
        let started = Instant::now();
        for _ in 0..ITERATIONS {
            black_box(
                evaluate_complete_owned_reference(
                    &expression,
                    &document,
                    order,
                    &mut InvocationControl::unbounded(),
                )
                .expect("owned reference evaluation"),
            );
        }
        let reference_microseconds =
            started.elapsed().as_secs_f64() * 1_000_000.0 / f64::from(ITERATIONS);

        println!(
            "items={ITEMS} borrowed_allocations={borrowed_allocations:?} reference_allocations={reference_allocations:?} borrowed_us={borrowed_microseconds:.6} reference_us={reference_microseconds:.6}"
        );
        assert_eq!(
            reference_allocations.count_total - borrowed_allocations.count_total,
            ITEMS as u64 * 4
        );
    }
}

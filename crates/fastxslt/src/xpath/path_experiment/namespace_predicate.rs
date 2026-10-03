//! Bounded namespace scalar/literal and integer-position filters.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::qualified_nodes::QualifiedSourceNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Operand {
    Name,
    StringValue,
    NamespaceUri,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NamespacePredicate {
    Scalar {
        operand: Operand,
        unequal: bool,
        literal: String,
    },
    Position(usize),
}

impl NamespacePredicate {
    #[cfg(feature = "workbench")]
    pub(super) fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Scalar { literal, .. } => literal.capacity(),
            Self::Position(_) => 0,
        }
    }

    pub(super) fn matches(
        &self,
        node: QualifiedSourceNode<'_>,
        position: usize,
        control: &mut InvocationControl,
    ) -> Result<bool, ControlFailure> {
        control.charge(WorkDomain::XPathOperation, 1)?;
        let Self::Scalar {
            operand,
            unequal,
            literal,
        } = self
        else {
            let Self::Position(expected) = self else {
                unreachable!()
            };
            return Ok(position == *expected);
        };
        let namespace = node
            .namespace()
            .expect("namespace terminal produces only namespace nodes");
        let value = match operand {
            Operand::Name => {
                control.charge(WorkDomain::XPathNodeVisit, 1)?;
                namespace.prefix()
            }
            Operand::StringValue => {
                control.charge(WorkDomain::XdmStringValueNode, 1)?;
                namespace.string_value()
            }
            Operand::NamespaceUri => {
                control.charge(WorkDomain::XPathNodeVisit, 1)?;
                ""
            }
        };
        Ok((value == literal) != *unequal)
    }
}

pub(super) fn parse_terminal(terminal: &str) -> Option<(&str, Option<NamespacePredicate>)> {
    let Some((test, predicate)) = terminal.split_once('[') else {
        return Some((terminal.trim(), None));
    };
    let predicate = predicate.strip_suffix(']')?.trim();
    if !predicate.is_empty() && predicate.bytes().all(|byte| byte.is_ascii_digit()) {
        return Some((
            test.trim(),
            Some(NamespacePredicate::Position(predicate.parse().ok()?)),
        ));
    }
    let (operand, tail) = [
        ("name(.)", Operand::Name),
        ("name()", Operand::Name),
        ("local-name(.)", Operand::Name),
        ("local-name()", Operand::Name),
        ("string(.)", Operand::StringValue),
        ("string()", Operand::StringValue),
        ("namespace-uri(.)", Operand::NamespaceUri),
        ("namespace-uri()", Operand::NamespaceUri),
    ]
    .into_iter()
    .find_map(|(function, operand)| {
        predicate
            .strip_prefix(function)
            .map(|tail| (operand, tail.trim()))
    })?;
    let (unequal, literal) = if let Some(literal) = tail.strip_prefix("!=") {
        (true, literal)
    } else {
        (false, tail.strip_prefix('=')?)
    };
    let literal = literal.trim();
    let quote = literal
        .chars()
        .next()
        .filter(|quote| matches!(quote, '\'' | '"'))?;
    let literal = literal.strip_prefix(quote)?.strip_suffix(quote)?;
    if literal.contains(quote) {
        return None;
    }
    Some((
        test.trim(),
        Some(NamespacePredicate::Scalar {
            operand,
            unequal,
            literal: literal.to_owned(),
        }),
    ))
}

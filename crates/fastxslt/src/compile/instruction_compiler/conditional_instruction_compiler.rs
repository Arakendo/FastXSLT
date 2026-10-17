//! XSLT conditional instruction validation and assembly, not `XPath` policy.

use super::{
    BooleanExpression, CompileFailure, Document, Instruction, NodeId, SourceLocation,
    compile_boolean_test, compile_sequence, ensure_only_attributes, invalid,
    is_forward_compatible_processing, is_xslt_element, meaningful_children, optional_attribute,
    unsupported,
};
use crate::xslt::golden_semantics_experiment::ChooseBranch;

pub(super) fn compile_if(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    compile_if_with_lowerers(
        document,
        element,
        &mut compile_boolean_test,
        &mut compile_sequence,
    )
}

pub(super) fn compile_if_with_lowerers(
    document: &Document,
    element: NodeId,
    lower_test: &mut impl FnMut(
        &Document,
        NodeId,
        &str,
        &SourceLocation,
    ) -> Result<BooleanExpression, CompileFailure>,
    lower_body: &mut impl FnMut(&Document, NodeId) -> Result<Vec<Instruction>, CompileFailure>,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &["test", "default-mode", "xpath-default-namespace"],
        "xsl:if",
    )?;
    let location = document.location(element).clone();
    let expression = required_conditional_test(document, element)?;
    Ok(Instruction::If {
        test: lower_test(document, element, expression, &location)?,
        body: lower_body(document, element)?,
        location,
    })
}

pub(super) fn compile_choose(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    compile_choose_with_lowerers(
        document,
        element,
        &mut compile_boolean_test,
        &mut |document, child, test| {
            if test.is_some_and(|test| matches!(test, BooleanExpression::Constant(false)))
                && is_forward_compatible_processing(document, child)
            {
                Ok(Vec::new())
            } else {
                compile_sequence(document, child)
            }
        },
    )
}

pub(super) fn compile_choose_with_lowerers(
    document: &Document,
    element: NodeId,
    lower_test: &mut impl FnMut(
        &Document,
        NodeId,
        &str,
        &SourceLocation,
    ) -> Result<BooleanExpression, CompileFailure>,
    lower_body: &mut impl FnMut(
        &Document,
        NodeId,
        Option<&BooleanExpression>,
    ) -> Result<Vec<Instruction>, CompileFailure>,
) -> Result<Instruction, CompileFailure> {
    ensure_choose_attributes(document, element)?;
    let children = meaningful_children(document, element);
    validate_choose_structure(document, element, &children)?;
    let mut branches = Vec::new();
    let mut otherwise = None;
    for child in children {
        if is_xslt_element(document, child, "when") {
            ensure_only_attributes(
                document,
                child,
                &["test", "xpath-default-namespace"],
                "xsl:when",
            )?;
            let expression = required_conditional_test(document, child)?;
            let test = lower_test(document, child, expression, document.location(child))?;
            let body = lower_body(document, child, Some(&test))?;
            branches.push(ChooseBranch {
                #[cfg(test)]
                condition_reference: document.location(child).clone(),
                test,
                body,
            });
        } else if is_xslt_element(document, child, "otherwise") {
            ensure_only_attributes(
                document,
                child,
                &["xpath-default-namespace"],
                "xsl:otherwise",
            )?;
            otherwise = Some(lower_body(document, child, None)?);
        } else {
            unreachable!("validate_choose_structure rejects other children")
        }
    }
    Ok(Instruction::Choose {
        branches,
        otherwise: otherwise.unwrap_or_default(),
        location: document.location(element).clone(),
    })
}

fn ensure_choose_attributes(document: &Document, element: NodeId) -> Result<(), CompileFailure> {
    const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
    for attribute in document.attributes(element) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have expanded names");
        if name.namespace.is_none() && name.local == "xpath-default-namespace" {
            continue;
        }
        if name.namespace.is_none() && name.local == "default-collation" {
            continue;
        }
        if name.namespace.as_deref() == Some(XML_NAMESPACE) && name.local == "space" {
            match document.value(*attribute).unwrap_or_default() {
                "default" | "preserve" => continue,
                _ => {
                    return Err(invalid(
                        "XTSE0020",
                        "xml:space must be 'default' or 'preserve'",
                        document.location(*attribute),
                    ));
                }
            }
        }
        return Err(unsupported(
            "FXST1009",
            format!(
                "unsupported attribute on xsl:choose: {{{}}}{}",
                name.namespace.as_deref().unwrap_or(""),
                name.local
            ),
            document.location(*attribute),
        ));
    }
    Ok(())
}

fn validate_choose_structure(
    document: &Document,
    element: NodeId,
    children: &[NodeId],
) -> Result<(), CompileFailure> {
    let mut when_count = 0_usize;
    let mut saw_otherwise = false;
    for child in children {
        if is_xslt_element(document, *child, "when") {
            if saw_otherwise {
                return Err(invalid(
                    "XTSE0010",
                    "xsl:when cannot follow xsl:otherwise",
                    document.location(*child),
                ));
            }
            required_conditional_test(document, *child)?;
            when_count += 1;
        } else if is_xslt_element(document, *child, "otherwise") {
            if saw_otherwise {
                return Err(invalid(
                    "XTSE0010",
                    "xsl:choose permits at most one xsl:otherwise",
                    document.location(*child),
                ));
            }
            saw_otherwise = true;
        } else {
            return Err(invalid(
                "XTSE0010",
                "xsl:choose permits only xsl:when and xsl:otherwise children",
                document.location(*child),
            ));
        }
    }
    if when_count == 0 {
        return Err(invalid(
            "XTSE0010",
            "xsl:choose requires at least one xsl:when",
            document.location(element),
        ));
    }
    Ok(())
}

fn required_conditional_test(document: &Document, element: NodeId) -> Result<&str, CompileFailure> {
    optional_attribute(document, element, None, "test").ok_or_else(|| {
        invalid(
            "XTSE0010",
            "xsl:if and xsl:when require a test attribute",
            document.location(element),
        )
    })
}

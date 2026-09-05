//! Private namespace-scope composition for result serialization.

use std::collections::HashMap;

use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};

use super::{BudgetedString, ExecutionFailure, FailureCategory, escape_attribute, failure};

#[derive(Clone, Copy)]
pub(super) enum NamespaceMode {
    ScopedStack,
    CompleteClone,
}

pub(super) enum NamespaceScope<'a> {
    ScopedStack(ScopedNamespaceStack<'a>),
    CompleteClone(Vec<NamespaceBinding>),
}

#[derive(Default)]
pub(super) struct ScopedNamespaceStack<'a> {
    bindings: Vec<ScopedNamespaceBinding<'a>>,
    declarations: Vec<bool>,
    slots: Vec<NamespacePrefixSlot>,
    slot_by_prefix: HashMap<String, usize>,
    changes: Vec<(usize, Option<usize>)>,
}

struct ScopedNamespaceBinding<'a> {
    prefix: Option<&'a str>,
    namespace: &'a str,
    slot: usize,
}

struct NamespacePrefixSlot {
    active: Option<usize>,
}

pub(super) enum NamespaceFrame {
    ScopedStack {
        binding_len: usize,
        change_len: usize,
    },
    CompleteClone {
        inherited: Vec<NamespaceBinding>,
        declarations: Vec<NamespaceBinding>,
    },
}

impl<'a> NamespaceScope<'a> {
    pub(super) fn new(mode: NamespaceMode) -> Self {
        match mode {
            NamespaceMode::ScopedStack => Self::ScopedStack(ScopedNamespaceStack::default()),
            NamespaceMode::CompleteClone => Self::CompleteClone(Vec::new()),
        }
    }

    pub(super) fn enter_borrowed(
        &mut self,
        name: &ExpandedName,
        namespaces: &'a [NamespaceBinding],
    ) -> NamespaceFrame {
        match self {
            Self::ScopedStack(scope) => scope.enter_borrowed(name, namespaces),
            Self::CompleteClone(in_scope) => {
                let inherited = std::mem::take(in_scope);
                let (next, declarations) = complete_namespace_scope(name, namespaces, &inherited);
                *in_scope = next;
                NamespaceFrame::CompleteClone {
                    inherited,
                    declarations,
                }
            }
        }
    }

    pub(super) fn enter_transient(
        &mut self,
        name: &ExpandedName,
        namespaces: &[NamespaceBinding],
    ) -> NamespaceFrame {
        match self {
            Self::CompleteClone(in_scope) => {
                let inherited = std::mem::take(in_scope);
                let (next, declarations) = complete_namespace_scope(name, namespaces, &inherited);
                *in_scope = next;
                NamespaceFrame::CompleteClone {
                    inherited,
                    declarations,
                }
            }
            Self::ScopedStack(_) => {
                unreachable!("transient normalized bindings require complete clone mode")
            }
        }
    }

    pub(super) fn exit(&mut self, frame: NamespaceFrame) {
        match (self, frame) {
            (
                Self::ScopedStack(scope),
                NamespaceFrame::ScopedStack {
                    binding_len,
                    change_len,
                },
            ) => scope.exit(binding_len, change_len),
            (Self::CompleteClone(in_scope), NamespaceFrame::CompleteClone { inherited, .. }) => {
                *in_scope = inherited;
            }
            _ => unreachable!("namespace frame must match its serialization mode"),
        }
    }

    pub(super) fn element_prefix(
        &self,
        namespace: Option<&str>,
        output: &BudgetedString,
    ) -> Result<Option<String>, ExecutionFailure> {
        match self {
            Self::ScopedStack(scope) => scope.element_prefix(namespace, output),
            Self::CompleteClone(in_scope) => complete_element_prefix(namespace, in_scope, output)
                .map(|prefix| prefix.map(str::to_owned)),
        }
    }

    pub(super) fn attribute_prefix(
        &self,
        namespace: Option<&str>,
        output: &BudgetedString,
    ) -> Result<Option<String>, ExecutionFailure> {
        match self {
            Self::ScopedStack(scope) => scope.attribute_prefix(namespace, output),
            Self::CompleteClone(in_scope) => complete_attribute_prefix(namespace, in_scope, output)
                .map(|prefix| prefix.map(str::to_owned)),
        }
    }

    pub(super) fn write_declarations(
        &self,
        frame: &NamespaceFrame,
        output: &mut BudgetedString,
    ) -> Result<(), ExecutionFailure> {
        match (self, frame) {
            (Self::ScopedStack(scope), NamespaceFrame::ScopedStack { binding_len, .. }) => {
                for (binding, declaration) in scope.bindings[*binding_len..]
                    .iter()
                    .zip(&scope.declarations[*binding_len..])
                {
                    if *declaration {
                        write_scoped_declaration(binding, output)?;
                    }
                }
            }
            (Self::CompleteClone(_), NamespaceFrame::CompleteClone { declarations, .. }) => {
                for binding in declarations {
                    write_complete_declaration(binding, output)?;
                }
            }
            _ => unreachable!("namespace frame must match its serialization mode"),
        }
        Ok(())
    }
}

impl<'a> ScopedNamespaceStack<'a> {
    fn enter_borrowed(
        &mut self,
        name: &ExpandedName,
        namespaces: &'a [NamespaceBinding],
    ) -> NamespaceFrame {
        let binding_len = self.bindings.len();
        let change_len = self.changes.len();
        for binding in namespaces {
            self.push(binding.prefix.as_deref(), &binding.namespace);
        }
        self.undeclare_default_if_needed(name);
        NamespaceFrame::ScopedStack {
            binding_len,
            change_len,
        }
    }

    fn undeclare_default_if_needed(&mut self, name: &ExpandedName) {
        if name.namespace.is_none()
            && self
                .active_binding("")
                .is_some_and(|binding| !binding.namespace.is_empty())
        {
            self.push(None, "");
        }
    }

    fn exit(&mut self, binding_len: usize, change_len: usize) {
        while self.changes.len() > change_len {
            let (slot, previous) = self
                .changes
                .pop()
                .expect("namespace change length was checked");
            self.slots[slot].active = previous;
        }
        self.bindings.truncate(binding_len);
        self.declarations.truncate(binding_len);
    }

    fn push(&mut self, prefix: Option<&'a str>, namespace: &'a str) {
        let index = self.bindings.len();
        let prefix_key = prefix.unwrap_or("");
        let slot = if let Some(slot) = self.slot_by_prefix.get(prefix_key) {
            *slot
        } else {
            let slot = self.slots.len();
            self.slots.push(NamespacePrefixSlot { active: None });
            self.slot_by_prefix.insert(prefix_key.to_owned(), slot);
            slot
        };
        let previous = self.slots[slot].active.replace(index);
        let declaration = previous
            .and_then(|previous| self.bindings.get(previous))
            .is_none_or(|active| active.namespace != namespace);
        self.changes.push((slot, previous));
        self.bindings.push(ScopedNamespaceBinding {
            prefix,
            namespace,
            slot,
        });
        self.declarations.push(declaration);
    }

    fn active_binding(&self, prefix: &str) -> Option<&ScopedNamespaceBinding<'_>> {
        self.slot_by_prefix
            .get(prefix)
            .and_then(|slot| self.slots[*slot].active)
            .and_then(|index| self.bindings.get(index))
    }

    fn is_active(&self, index: usize) -> bool {
        self.slots[self.bindings[index].slot].active == Some(index)
    }

    fn element_prefix(
        &self,
        namespace: Option<&str>,
        output: &BudgetedString,
    ) -> Result<Option<String>, ExecutionFailure> {
        let Some(namespace) = namespace else {
            return Ok(None);
        };
        self.bindings
            .iter()
            .enumerate()
            .filter(|(index, binding)| self.is_active(*index) && binding.namespace == namespace)
            .min_by_key(|(_, binding)| usize::from(binding.prefix.is_some()))
            .map(|(_, binding)| binding.prefix.map(str::to_owned))
            .ok_or_else(|| missing_element_namespace(namespace, output))
    }

    fn attribute_prefix(
        &self,
        namespace: Option<&str>,
        output: &BudgetedString,
    ) -> Result<Option<String>, ExecutionFailure> {
        let Some(namespace) = namespace else {
            return Ok(None);
        };
        if namespace == "http://www.w3.org/XML/1998/namespace" {
            return Ok(Some("xml".to_owned()));
        }
        self.bindings
            .iter()
            .enumerate()
            .find(|(index, binding)| {
                self.is_active(*index) && binding.prefix.is_some() && binding.namespace == namespace
            })
            .and_then(|(_, binding)| binding.prefix.map(str::to_owned))
            .map(Some)
            .ok_or_else(|| missing_attribute_namespace(namespace, output))
    }
}

pub(super) fn complete_namespace_scope(
    name: &ExpandedName,
    namespaces: &[NamespaceBinding],
    inherited_namespaces: &[NamespaceBinding],
) -> (Vec<NamespaceBinding>, Vec<NamespaceBinding>) {
    let mut in_scope = inherited_namespaces.to_vec();
    let mut declarations = Vec::new();
    for binding in namespaces {
        let inherited = in_scope.iter().position(|candidate| {
            candidate.prefix == binding.prefix && candidate.namespace == binding.namespace
        });
        if inherited.is_none() {
            declarations.push(binding.clone());
        }
        in_scope.retain(|candidate| candidate.prefix != binding.prefix);
        in_scope.push(binding.clone());
    }
    if name.namespace.is_none()
        && in_scope
            .iter()
            .any(|binding| binding.prefix.is_none() && !binding.namespace.is_empty())
    {
        let undeclaration = NamespaceBinding {
            prefix: None,
            namespace: String::new(),
        };
        declarations.push(undeclaration.clone());
        in_scope.retain(|binding| binding.prefix.is_some());
        in_scope.push(undeclaration);
    }
    (in_scope, declarations)
}

fn complete_attribute_prefix<'a>(
    namespace: Option<&str>,
    in_scope: &'a [NamespaceBinding],
    output: &BudgetedString,
) -> Result<Option<&'a str>, ExecutionFailure> {
    let Some(namespace) = namespace else {
        return Ok(None);
    };
    if namespace == "http://www.w3.org/XML/1998/namespace" {
        return Ok(Some("xml"));
    }
    in_scope
        .iter()
        .find(|binding| binding.prefix.is_some() && binding.namespace == namespace)
        .and_then(|binding| binding.prefix.as_deref())
        .map(Some)
        .ok_or_else(|| missing_attribute_namespace(namespace, output))
}

pub(super) fn complete_element_prefix<'a>(
    namespace: Option<&str>,
    in_scope: &'a [NamespaceBinding],
    output: &BudgetedString,
) -> Result<Option<&'a str>, ExecutionFailure> {
    let Some(namespace) = namespace else {
        return Ok(None);
    };
    in_scope
        .iter()
        .filter(|binding| binding.namespace == namespace)
        .min_by_key(|binding| usize::from(binding.prefix.is_some()))
        .map(|binding| binding.prefix.as_deref())
        .ok_or_else(|| missing_element_namespace(namespace, output))
}

fn write_complete_declaration(
    binding: &NamespaceBinding,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    output.push_str(" xmlns")?;
    if let Some(prefix) = &binding.prefix {
        output.push(':')?;
        output.push_str(prefix)?;
    }
    output.push_str("=\"")?;
    escape_attribute(&binding.namespace, output)?;
    output.push('"')
}

fn write_scoped_declaration(
    binding: &ScopedNamespaceBinding<'_>,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    output.push_str(" xmlns")?;
    if let Some(prefix) = binding.prefix {
        output.push(':')?;
        output.push_str(prefix)?;
    }
    output.push_str("=\"")?;
    escape_attribute(binding.namespace, output)?;
    output.push('"')
}

fn missing_attribute_namespace(namespace: &str, output: &BudgetedString) -> ExecutionFailure {
    failure(
        "FXSR1002",
        FailureCategory::Unsupported,
        Some(&output.request_id),
        format!("result attribute namespace has no retained prefix binding: {namespace}"),
    )
}

fn missing_element_namespace(namespace: &str, output: &BudgetedString) -> ExecutionFailure {
    failure(
        "FXSR1002",
        FailureCategory::Unsupported,
        Some(&output.request_id),
        format!("result namespace has no retained prefix binding: {namespace}"),
    )
}

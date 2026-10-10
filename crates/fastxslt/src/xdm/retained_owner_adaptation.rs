//! Principal borrowed-document adaptation experiment for ADR-0022.
//! Shares immutable records; copies only effective relationships and ID metadata.
//! Test-only until compiled admission and complete lifecycle measurements pass.

use super::*;

impl Document {
    fn retained_owner_reference(
        &self,
        control: &mut InvocationControl,
    ) -> Result<Arc<Self>, ControlFailure> {
        // Reference formula only: header, then each retained metadata unit.
        // No payload traversal, reparsing, or origin assignment is performed.
        control.charge(WorkDomain::XdmNode, 1)?;
        let child_overrides = if let Some(overrides) = &self.child_overrides {
            let mut copied = HashMap::new();
            for (&parent, children) in overrides.iter() {
                control.charge(WorkDomain::XdmNode, 1)?;
                let mut visible = Vec::new();
                for &child in children {
                    control.charge(WorkDomain::XdmNode, 1)?;
                    visible.push(child);
                }
                copied.insert(parent, visible.into_boxed_slice());
            }
            Some(copied)
        } else {
            None
        };
        let mut id_index = HashMap::new();
        for (key, ids) in self.id_index.iter() {
            control.charge(WorkDomain::XdmNode, 1)?;
            control.charge(WorkDomain::XdmNode, key.len())?;
            let mut copied = Vec::new();
            for &id in ids {
                control.charge(WorkDomain::XdmNode, 1)?;
                copied.push(id);
            }
            id_index.insert(key.clone(), copied);
        }
        Ok(Arc::new(Self {
            origin: self.origin.clone(),
            nodes: self.nodes.clone(),
            root: self.root,
            child_overrides: child_overrides.map(Arc::new),
            id_index: Arc::new(id_index),
        }))
    }

    fn retained_owner_shared_reference(
        &self,
        control: &mut InvocationControl,
    ) -> Result<Arc<Self>, ControlFailure> {
        self.retain_effective_owner(control)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    use crate::xdm::qualified_nodes::QualifiedSourceNode;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn source(items: usize, typed_ids: bool) -> Document {
        let xml = format!("<r>{}</r>", " <a>text</a> ".repeat(items));
        let mut parsed = parse_document(
            "memory:principal.xml",
            xml.as_bytes(),
            ParseLimits {
                max_events: 10_000,
                max_depth: 20,
            },
        )
        .unwrap();
        // This shape probe isolates metadata retention, not DTD conformance.
        if typed_ids {
            for (index, event) in parsed.events.iter_mut().enumerate() {
                if let OwnedXmlEvent::Start { attributes, .. } = event {
                    attributes.push(crate::xml::quick_xml_experiment::XmlAttribute {
                        name: ExpandedName {
                            namespace: None,
                            local: "id".into(),
                        },
                        prefix: None,
                        value: format!("id{index}"),
                        span: 0..0,
                        is_id: true,
                    });
                }
            }
        }
        Document::from_parsed(parsed).unwrap()
    }

    #[test]
    fn borrowed_adaptation_retains_identity_effective_navigation_and_metadata() {
        let original = source(3, true);
        let view = original
            .view_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
            .unwrap();
        let retained = view
            .retained_owner_shared_reference(&mut InvocationControl::unbounded())
            .unwrap();
        let copied = view
            .retained_owner_reference(&mut InvocationControl::unbounded())
            .unwrap();
        assert!(Arc::ptr_eq(&retained.id_index, &view.id_index));
        assert!(!Arc::ptr_eq(&copied.id_index, &view.id_index));
        assert!(Arc::ptr_eq(
            retained.child_overrides.as_ref().unwrap(),
            view.child_overrides.as_ref().unwrap()
        ));
        assert!(retained.shares_node_storage_with(&original));
        assert!(retained.same_origin(&original));
        assert!(retained.same_effective_storage(&view));
        assert!(!retained.same_effective_storage(&original));
        // Equivalent visible relationships do not prove shared storage: this
        // conservative fast-path predicate is not semantic node equality.
        assert!(!retained.same_effective_storage(&copied));
        assert_ne!(
            retained.children(original.children(original.root)[0]),
            original.children(original.children(original.root)[0])
        );
        for index in 0..original.node_count() {
            let id = NodeId(index);
            assert!(
                QualifiedSourceNode::Tree {
                    document: &original,
                    id
                }
                .same_node(QualifiedSourceNode::Tree {
                    document: &retained,
                    id
                })
            );
            assert_eq!(retained.children(id), view.children(id));
            assert_eq!(retained.children(id), copied.children(id));
            assert_eq!(retained.location(id), view.location(id));
        }
        assert_eq!(retained.id_index, original.id_index);
        let reparse = source(3, true);
        assert!(!retained.same_origin(&reparse));
        assert!(!retained.same_effective_storage(&reparse));
        let node_storage = Arc::downgrade(&original.nodes);
        let id_metadata = Arc::downgrade(&original.id_index);
        let effective_relationships = Arc::downgrade(view.child_overrides.as_ref().unwrap());
        drop(copied);
        drop(view);
        drop(original);
        assert_eq!(Arc::strong_count(&retained.nodes), 1);
        assert_eq!(retained.string_value(retained.root), "texttexttext");
        drop(retained);
        assert!(node_storage.upgrade().is_none());
        assert!(id_metadata.upgrade().is_none());
        assert!(effective_relationships.upgrade().is_none());
    }

    #[test]
    fn adaptation_budget_and_cancellation_do_not_publish_or_retain_partial_owners() {
        let original = source(3, true);
        let view = original
            .view_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
            .unwrap();
        let baseline = Arc::strong_count(&view.nodes);
        let charge = 1
            + view
                .child_overrides
                .as_ref()
                .unwrap()
                .values()
                .map(|ids| 1 + ids.len())
                .sum::<usize>()
            + view
                .id_index
                .iter()
                .map(|(key, ids)| 1 + key.len() + ids.len())
                .sum::<usize>();
        for limit in [0, charge - 1, charge] {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xdm_nodes: limit,
                    ..WorkLimits::unbounded()
                },
            );
            let outcome = view.retained_owner_reference(&mut control);
            assert_eq!(outcome.is_ok(), limit == charge);
            drop(outcome);
            assert_eq!(Arc::strong_count(&view.nodes), baseline);
        }
        // The fault hook counts charge calls, not charged work units. A key's
        // byte charge is one call and may account for multiple units.
        let checkpoints = 1
            + view
                .child_overrides
                .as_ref()
                .unwrap()
                .values()
                .map(|ids| 1 + ids.len())
                .sum::<usize>()
            + view
                .id_index
                .values()
                .map(|ids| 2 + ids.len())
                .sum::<usize>();
        for cancellation in 0..checkpoints {
            let mut control = InvocationControl::unbounded()
                .cancelling_on_charge(WorkDomain::XdmNode, cancellation);
            assert!(matches!(
                view.retained_owner_reference(&mut control),
                Err(ControlFailure::Cancelled { .. })
            ));
            assert_eq!(Arc::strong_count(&view.nodes), baseline);
        }
        for limit in [0, 1] {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xdm_nodes: limit,
                    ..WorkLimits::unbounded()
                },
            );
            let result = view.retained_owner_shared_reference(&mut control);
            assert_eq!(result.is_ok(), limit == 1);
            drop(result);
            assert_eq!(Arc::strong_count(&view.nodes), baseline);
        }
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 0);
        assert!(matches!(
            view.retained_owner_shared_reference(&mut cancelled),
            Err(ControlFailure::Cancelled { .. })
        ));
    }

    #[cfg(feature = "allocation-observation")]
    #[test]
    fn borrowed_owner_adaptation_allocation_pressure() {
        for items in [50, 500] {
            for shape in ["preserve", "strip", "ids"] {
                let original = source(items, shape == "ids");
                let effective = if shape == "strip" {
                    original
                        .view_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
                        .unwrap()
                } else {
                    original
                };
                let baseline = Arc::strong_count(&effective.nodes);
                for lane in ["copy", "share"] {
                    let info = allocation_counter::measure(|| {
                        let retained = if lane == "copy" {
                            effective.retained_owner_reference(&mut InvocationControl::unbounded())
                        } else {
                            effective.retained_owner_shared_reference(
                                    &mut InvocationControl::unbounded(),
                                )
                        }
                        .unwrap();
                        std::hint::black_box(retained);
                    });
                    assert_eq!(info.bytes_current, 0);
                    assert_eq!(info.count_current, 0);
                    assert_eq!(Arc::strong_count(&effective.nodes), baseline);
                    println!(
                        "items={items} shape={shape} lane={lane} prepared_capacity={} adaptation_total={} adaptation_peak={} allocations={}",
                        effective.owned_capacity_bytes(),
                        info.bytes_total,
                        info.bytes_max,
                        info.count_total
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "manual release principal-owner adaptation timings; omit allocation-observation"]
    fn borrowed_owner_adaptation_timing() {
        for items in [50, 500] {
            for shape in ["preserve", "strip", "ids"] {
                let original = source(items, shape == "ids");
                let effective = if shape == "strip" {
                    original
                        .view_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
                        .unwrap()
                } else {
                    original
                };
                for trial in 0..5 {
                    for lane in if trial % 2 == 0 {
                        ["copy", "share"]
                    } else {
                        ["share", "copy"]
                    } {
                        let start = std::time::Instant::now();
                        let mut completed = 0_u32;
                        loop {
                            for _ in 0..100 {
                                std::hint::black_box(
                                    if lane == "copy" {
                                        effective.retained_owner_reference(
                                            &mut InvocationControl::unbounded(),
                                        )
                                    } else {
                                        effective.retained_owner_shared_reference(
                                            &mut InvocationControl::unbounded(),
                                        )
                                    }
                                    .unwrap(),
                                );
                            }
                            completed += 100;
                            if start.elapsed() >= std::time::Duration::from_millis(20) {
                                break;
                            }
                        }
                        let ns =
                            start.elapsed().as_secs_f64() * 1_000_000_000.0 / f64::from(completed);
                        println!(
                            "items={items} shape={shape} trial={trial} lane={lane} adaptation_construct_drop_ns={ns:.1}"
                        );
                    }
                }
            }
        }
    }
}

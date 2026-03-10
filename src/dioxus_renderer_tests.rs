use super::*;
use crate::strata::FrameStrata;
use dioxus_core::TemplateAttribute;

#[test]
fn renderer_new_succeeds() {
    let renderer = GameUiRenderer::new();
    assert!(renderer.nodes.is_empty());
    assert!(renderer.stack.is_empty());
}

#[test]
fn frame_id_returns_none_for_unknown() {
    let renderer = GameUiRenderer::new();
    assert_eq!(renderer.frame_id(ElementId(0)), None);
    assert_eq!(renderer.frame_id(ElementId(999)), None);
}

#[test]
fn create_text_node_creates_fontstring() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let mut applier = MutationApplier::new(&mut renderer, &mut registry);
    let eid = ElementId(1);
    applier.create_text_node("Hello", eid);
    let fid = applier.renderer.frame_id(eid).unwrap();
    let frame = applier.registry.get(fid).unwrap();
    assert_eq!(frame.widget_type, WidgetType::FontString);
}

#[test]
fn create_placeholder_has_no_frame() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let mut applier = MutationApplier::new(&mut renderer, &mut registry);
    let eid = ElementId(1);
    applier.create_placeholder(eid);
    assert_eq!(applier.renderer.frame_id(eid), None);
}

#[test]
fn remove_node_clears_slot() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let mut applier = MutationApplier::new(&mut renderer, &mut registry);
    let eid = ElementId(1);
    applier.create_text_node("test", eid);
    assert!(applier.renderer.frame_id(eid).is_some());
    applier.remove_node(eid);
    assert_eq!(applier.renderer.frame_id(eid), None);
}

#[test]
fn set_attribute_width_height() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let fid = registry.next_id();
    registry.insert_frame(Frame::new(fid, None, WidgetType::Frame));
    renderer.ensure_slot(ElementId(1));
    renderer.nodes[1] = Some(NodeKind::Element { frame_id: fid });
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.set_attribute("width", None, &AttributeValue::Float(200.0), ElementId(1));
        applier.set_attribute("height", None, &AttributeValue::Float(100.0), ElementId(1));
    }
    let frame = registry.get(fid).unwrap();
    assert!((frame.width - 200.0).abs() < f32::EPSILON);
    assert!((frame.height - 100.0).abs() < f32::EPSILON);
}

#[test]
fn set_attribute_strata() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let fid = registry.next_id();
    registry.insert_frame(Frame::new(fid, None, WidgetType::Frame));
    renderer.ensure_slot(ElementId(1));
    renderer.nodes[1] = Some(NodeKind::Element { frame_id: fid });
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.set_attribute(
            "strata",
            None,
            &AttributeValue::Text("DIALOG".into()),
            ElementId(1),
        );
    }
    assert_eq!(registry.get(fid).unwrap().strata, FrameStrata::Dialog);
}

#[test]
fn append_children_wires_parent_child() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let pfid = registry.next_id();
    registry.insert_frame(Frame::new(pfid, None, WidgetType::Frame));
    renderer.ensure_slot(ElementId(1));
    renderer.nodes[1] = Some(NodeKind::Element { frame_id: pfid });
    let cfid = registry.next_id();
    registry.insert_frame(Frame::new(cfid, None, WidgetType::Button));
    renderer.ensure_slot(ElementId(2));
    renderer.nodes[2] = Some(NodeKind::Element { frame_id: cfid });
    renderer.stack.push(ElementId(2));
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.append_children(ElementId(1), 1);
    }
    assert_eq!(registry.get(cfid).unwrap().parent_id, Some(pfid));
    assert!(registry.get(pfid).unwrap().children.contains(&cfid));
}

#[test]
fn parse_strata_all_variants() {
    use crate::strata::FrameStrata;
    assert_eq!(FrameStrata::from_str("WORLD"), Some(FrameStrata::World));
    assert_eq!(FrameStrata::from_str("DIALOG"), Some(FrameStrata::Dialog));
    assert_eq!(FrameStrata::from_str("UNKNOWN"), None);
}

#[test]
fn apply_attribute_text_on_button() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let fid = renderer.create_frame_for_tag("Button", ElementId(1), &mut registry);
    apply_attribute(
        &mut registry,
        fid,
        "text",
        &AttributeValue::Text("Click".into()),
        &mut HashSet::new(),
        &mut HashSet::new(),
    );
    let frame = registry.get(fid).unwrap();
    match &frame.widget_data {
        Some(WidgetData::Button(bd)) => assert_eq!(bd.text, "Click"),
        other => panic!("expected Button widget_data, got {:?}", other),
    }
}

fn make_anchor_node(attrs: &'static [dioxus_core::TemplateAttribute]) -> TemplateNode {
    TemplateNode::Element {
        tag: "Anchor",
        namespace: None,
        attrs,
        children: &[],
    }
}

static ANCHOR_CENTER_10_20: &[dioxus_core::TemplateAttribute] = &[
    dioxus_core::TemplateAttribute::Static {
        name: "point",
        value: "CENTER",
        namespace: None,
    },
    dioxus_core::TemplateAttribute::Static {
        name: "relative_point",
        value: "CENTER",
        namespace: None,
    },
    dioxus_core::TemplateAttribute::Static {
        name: "x",
        value: "10",
        namespace: None,
    },
    dioxus_core::TemplateAttribute::Static {
        name: "y",
        value: "20",
        namespace: None,
    },
];

#[test]
fn apply_anchor_element_resolves_parent() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let parent_fid = renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    let child_fid = renderer.create_frame_for_tag("Frame", ElementId(2), &mut registry);
    wire_parent_child(&mut registry, parent_fid, child_fid);
    let node = make_anchor_node(ANCHOR_CENTER_10_20);
    let pending = apply_anchor_element(&node, child_fid, &mut registry);
    assert!(pending.is_none());
    let child = registry.get(child_fid).unwrap();
    assert_eq!(child.anchors.len(), 1);
    assert_eq!(
        child.anchors[0].point,
        crate::anchor::AnchorPoint::Center
    );
    assert_eq!(child.anchors[0].relative_to, Some(parent_fid));
    assert_eq!(child.anchors[0].x_offset, 10.0);
    assert_eq!(child.anchors[0].y_offset, 20.0);
}

#[test]
fn anchor_element_does_not_create_frame() {
    #[allow(unused_imports)]
    use crate::dioxus_elements;
    use dioxus::prelude::*;

    fn comp() -> Element {
        rsx! {
            r#frame { name: "Parent", width: 100.0, height: 100.0,
                r#frame { name: "Child", width: 50.0, height: 50.0,
                    anchor { point: "CENTER", relative_point: "CENTER" }
                }
            }
        }
    }
    let mut dom = dioxus_core::VirtualDom::new(comp);
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let mut renderer = GameUiRenderer::new();
    let mut applier = MutationApplier::new(&mut renderer, &mut registry);
    dom.rebuild(&mut applier);

    let child_id = registry.get_by_name("Child").unwrap();
    let child = registry.get(child_id).unwrap();
    assert_eq!(child.anchors.len(), 1);
    assert_eq!(
        child.children.len(),
        0,
        "anchor element should not create a child frame"
    );
}

#[test]
fn apply_attribute_stretch() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let parent_fid = renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    let child_fid = renderer.create_frame_for_tag("Frame", ElementId(2), &mut registry);
    wire_parent_child(&mut registry, parent_fid, child_fid);
    apply_attribute(
        &mut registry,
        child_fid,
        "stretch",
        &AttributeValue::Bool(true),
        &mut HashSet::new(),
        &mut HashSet::new(),
    );
    let child = registry.get(child_fid).unwrap();
    assert_eq!(child.anchors.len(), 2);
    assert_eq!(child.anchors[0].relative_to, Some(parent_fid));
    assert_eq!(child.anchors[1].relative_to, Some(parent_fid));
}

#[test]
fn create_frame_for_tag_auto_inits_widget_data() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let button_fid = renderer.create_frame_for_tag("Button", ElementId(1), &mut registry);
    assert!(matches!(
        registry.get(button_fid).unwrap().widget_data,
        Some(WidgetData::Button(_))
    ));
    let editbox_fid = renderer.create_frame_for_tag("EditBox", ElementId(2), &mut registry);
    assert!(matches!(
        registry.get(editbox_fid).unwrap().widget_data,
        Some(WidgetData::EditBox(_))
    ));
    let fontstring_fid = renderer.create_frame_for_tag("FontString", ElementId(3), &mut registry);
    assert!(matches!(
        registry.get(fontstring_fid).unwrap().widget_data,
        Some(WidgetData::FontString(_))
    ));
    let texture_fid = renderer.create_frame_for_tag("Texture", ElementId(4), &mut registry);
    assert!(matches!(
        registry.get(texture_fid).unwrap().widget_data,
        Some(WidgetData::Texture(_))
    ));
    let frame_fid = renderer.create_frame_for_tag("Frame", ElementId(5), &mut registry);
    assert!(registry.get(frame_fid).unwrap().widget_data.is_none());
}

#[test]
fn registry_children_of_returns_ordered_children() {
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let parent = registry.create_frame("P", None);
    let c1 = registry.create_frame("C1", Some(parent));
    let c2 = registry.create_frame("C2", Some(parent));
    assert_eq!(registry.children_of(parent), vec![c1, c2]);
}

#[test]
fn registry_children_of_empty_for_leaf() {
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let leaf = registry.create_frame("Leaf", None);
    assert!(registry.children_of(leaf).is_empty());
}

#[test]
fn registry_children_of_nonexistent_returns_empty() {
    let registry = FrameRegistry::new(1024.0, 768.0);
    assert!(registry.children_of(999).is_empty());
}

// --- reuse + replace_node_with integration tests ---

/// Build a multi-root template. Each entry is (Option<name>, width).
/// Mirrors the real hotreload scenario: 4 roots, only one named.
fn make_multi_root_template(roots: &[(&str, &str)]) -> Template {
    let leaked_roots: Vec<TemplateNode> = roots
        .iter()
        .map(|(name, width)| {
            let width: &'static str = Box::leak(width.to_string().into_boxed_str());
            let mut attrs_vec: Vec<TemplateAttribute> = Vec::new();
            if !name.is_empty() {
                let name: &'static str = Box::leak(name.to_string().into_boxed_str());
                attrs_vec.push(TemplateAttribute::Static {
                    name: "name",
                    value: name,
                    namespace: None,
                });
            }
            attrs_vec.push(TemplateAttribute::Static {
                name: "width",
                value: width,
                namespace: None,
            });
            let attrs: &'static [TemplateAttribute] = Box::leak(attrs_vec.into_boxed_slice());
            TemplateNode::Element {
                tag: "Frame",
                namespace: None,
                attrs,
                children: &[],
            }
        })
        .collect();
    let roots: &'static [TemplateNode] = Box::leak(leaked_roots.into_boxed_slice());
    Template {
        roots,
        node_paths: Box::leak(Box::new([])),
        attr_paths: Box::leak(Box::new([])),
    }
}

/// Build a named template for reuse tests. Leaks to get 'static lifetime.
fn make_named_template(name: &str, width: &str) -> Template {
    let name: &'static str = Box::leak(name.to_string().into_boxed_str());
    let width: &'static str = Box::leak(width.to_string().into_boxed_str());
    let attrs: &'static [TemplateAttribute] = Box::leak(Box::new([
        TemplateAttribute::Static {
            name: "name",
            value: name,
            namespace: None,
        },
        TemplateAttribute::Static {
            name: "width",
            value: width,
            namespace: None,
        },
    ]));
    let roots: &'static [TemplateNode] = Box::leak(Box::new([TemplateNode::Element {
        tag: "Frame",
        namespace: None,
        attrs,
        children: &[],
    }]));
    Template {
        roots,
        node_paths: Box::leak(Box::new([])),
        attr_paths: Box::leak(Box::new([])),
    }
}

/// Simulate initial render: load_template + append_children under a parent.
fn initial_render(
    renderer: &mut GameUiRenderer,
    registry: &mut FrameRegistry,
    template: Template,
    parent_eid: ElementId,
    root_eid: ElementId,
) {
    let mut applier = MutationApplier::new(renderer, registry);
    applier.load_template(template, 0, root_eid, None);
    applier.append_children(parent_eid, 1);
}

/// Load all roots of a multi-root template, assigning ElementIds starting at `base`.
fn load_all_roots(applier: &mut MutationApplier, template: Template, eid_base: usize) {
    for i in 0..template.roots.len() {
        applier.load_template(template, i, ElementId(eid_base + i), None);
    }
}

/// Collect frame IDs for ElementIds in range [base, base+count).
fn collect_fids(renderer: &GameUiRenderer, base: usize, count: usize) -> Vec<u64> {
    (base..base + count)
        .filter_map(|i| renderer.frame_id(ElementId(i)))
        .collect()
}

#[test]
fn reuse_preserves_parent_after_replace() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    // Create a parent frame
    let parent_fid = renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    registry.set_name(parent_fid, "UIParent".to_string());
    // Initial render: named child under parent
    let t1 = make_named_template("Root", "100");
    initial_render(&mut renderer, &mut registry, t1, ElementId(1), ElementId(2));
    let root_fid = registry.get_by_name("Root").unwrap();
    assert_eq!(registry.get(root_fid).unwrap().parent_id, Some(parent_fid));

    // Hotreload: load new template (reuses by name), then replace old
    let t2 = make_named_template("Root", "200");
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.load_template(t2, 0, ElementId(3), None);
        applier.replace_node_with(ElementId(2), 1);
    }
    // Frame ID is stable
    assert_eq!(registry.get_by_name("Root"), Some(root_fid));
    // Parent preserved
    assert_eq!(
        registry.get(root_fid).unwrap().parent_id,
        Some(parent_fid),
        "reused frame must keep its parent_id"
    );
    // Width updated from new template
    assert!(
        (registry.get(root_fid).unwrap().width - 200.0).abs() < f32::EPSILON,
        "static attrs from new template must be applied"
    );
}

#[test]
fn reuse_cleans_old_children() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    let t1 = make_named_template("Root", "100");
    initial_render(&mut renderer, &mut registry, t1, ElementId(1), ElementId(2));
    let root_fid = registry.get_by_name("Root").unwrap();
    // Add a child manually to simulate template children
    let child_fid = registry.next_id();
    let mut child = Frame::new(child_fid, None, WidgetType::Frame);
    child.parent_id = Some(root_fid);
    registry.insert_frame(child);
    wire_parent_child(&mut registry, root_fid, child_fid);
    assert!(!registry.children_of(root_fid).is_empty());

    // Hotreload: reuse Root
    let t2 = make_named_template("Root", "200");
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.load_template(t2, 0, ElementId(3), None);
        applier.replace_node_with(ElementId(2), 1);
    }
    // Old child must be gone (not orphaned in registry)
    assert!(
        registry.get(child_fid).is_none(),
        "old children must be removed, not orphaned"
    );
}

#[test]
fn reuse_does_not_destroy_frame_on_replace() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    let t1 = make_named_template("Root", "100");
    initial_render(&mut renderer, &mut registry, t1, ElementId(1), ElementId(2));
    let root_fid = registry.get_by_name("Root").unwrap();

    // Hotreload
    let t2 = make_named_template("Root", "200");
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.load_template(t2, 0, ElementId(3), None);
        applier.replace_node_with(ElementId(2), 1);
    }
    assert!(
        registry.get(root_fid).is_some(),
        "reused frame must survive replace_node_with"
    );
}

#[test]
fn replace_without_reuse_still_destroys() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    // Create a frame that won't be reused (unnamed)
    let fid = renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    renderer.stack.push(ElementId(1));
    // Add a child
    let child_fid = registry.next_id();
    registry.insert_frame(Frame::new(child_fid, None, WidgetType::Frame));
    wire_parent_child(&mut registry, fid, child_fid);

    // Create replacement and push to stack
    renderer.create_frame_for_tag("Frame", ElementId(2), &mut registry);
    renderer.stack.push(ElementId(2));
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.replace_node_with(ElementId(1), 1);
    }
    assert!(
        registry.get(fid).is_none(),
        "non-reused frame must be destroyed by replace_node_with"
    );
    assert!(
        registry.get(child_fid).is_none(),
        "children of non-reused frame must also be destroyed"
    );
}

#[test]
fn reuse_cascade_destroys_reused_child() {
    // Reproduces the real hotreload bug: reused frame is a child of
    // the element being replaced. remove_frame_tree cascades into it.
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    // Component wrapper (ElementId 1, fid will be the parent)
    let wrapper_fid = renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    // Initial render: named root as child of wrapper
    let t1 = make_named_template("Root", "100");
    initial_render(&mut renderer, &mut registry, t1, ElementId(1), ElementId(2));
    let root_fid = registry.get_by_name("Root").unwrap();
    assert!(
        registry
            .get(wrapper_fid)
            .unwrap()
            .children
            .contains(&root_fid)
    );

    // Hotreload: load_template reuses Root, then replace_node_with on WRAPPER
    let t2 = make_named_template("Root", "200");
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.load_template(t2, 0, ElementId(3), None);
        // Replace the WRAPPER (parent of reused frame) — cascades!
        applier.replace_node_with(ElementId(1), 1);
    }
    // BUG: remove_frame_tree(wrapper) cascades into root_fid,
    // destroying the reused frame despite reused_frame_ids
    assert!(
        registry.get(root_fid).is_some(),
        "reused frame must survive cascading replace of its parent"
    );
}

#[test]
fn reuse_reparents_new_eid_under_old_parent() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);
    let parent_fid = renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    let t1 = make_named_template("Root", "100");
    initial_render(&mut renderer, &mut registry, t1, ElementId(1), ElementId(2));
    let root_fid = registry.get_by_name("Root").unwrap();

    // Hotreload
    let t2 = make_named_template("Root", "200");
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.load_template(t2, 0, ElementId(3), None);
        applier.replace_node_with(ElementId(2), 1);
    }
    // New ElementId(3) maps to same frame
    assert_eq!(renderer.frame_id(ElementId(3)), Some(root_fid));
    // Old ElementId(2) cleared
    assert_eq!(renderer.frame_id(ElementId(2)), None);
    // Parent still lists root as child
    assert!(
        registry
            .get(parent_fid)
            .unwrap()
            .children
            .contains(&root_fid),
        "parent must still reference reused frame"
    );
}

#[test]
fn replace_parent_with_reused_descendant_does_not_leave_duplicate_named_frames() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);

    let root_fid = renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    registry.set_name(root_fid, "LoginRoot".to_string());

    let old_container_fid = instantiate_element("Frame", root_fid, &mut registry);
    registry.set_name(old_container_fid, "LoginContainer".to_string());
    let old_status_fid = instantiate_element("FontString", old_container_fid, &mut registry);
    registry.set_name(old_status_fid, "LoginStatus".to_string());

    renderer.reused_frame_ids.insert(old_status_fid);

    let new_root_fid = renderer.create_frame_for_tag("Frame", ElementId(2), &mut registry);
    renderer.stack.push(ElementId(2));
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.replace_node_with(ElementId(1), 1);
    }

    let new_container_fid = instantiate_element("Frame", new_root_fid, &mut registry);
    registry.set_name(new_container_fid, "LoginContainer".to_string());
    let new_status_fid = instantiate_element("FontString", new_container_fid, &mut registry);
    registry.set_name(new_status_fid, "LoginStatus".to_string());

    let matching: Vec<u64> = registry
        .frames_iter()
        .filter(|frame| frame.name.as_deref() == Some("LoginStatus"))
        .map(|frame| frame.id)
        .collect();

    assert_eq!(
        matching,
        vec![new_status_fid],
        "expected the stale reused descendant to be detached or removed before recreating LoginStatus"
    );
}

#[test]
fn replace_placeholder_reparents_new_nodes_under_placeholder_parent() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);

    let parent_fid = renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    renderer.stack.push(ElementId(1));

    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.create_placeholder(ElementId(2));
        applier.append_children(ElementId(1), 1);
    }
    assert_eq!(renderer.frame_id(ElementId(2)), None);

    let child_fid = renderer.create_frame_for_tag("Button", ElementId(3), &mut registry);
    renderer.stack.push(ElementId(3));
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.replace_node_with(ElementId(2), 1);
    }

    assert_eq!(
        registry.get(child_fid).unwrap().parent_id,
        Some(parent_fid),
        "placeholder replacements must inherit the placeholder's parent"
    );
    assert!(
        registry.get(parent_fid).unwrap().children.contains(&child_fid),
        "parent must reference the replacement node"
    );
    assert_eq!(
        renderer.stack,
        vec![ElementId(1)],
        "replace_node_with must consume the replacement nodes it inserts"
    );
}

#[test]
fn replace_placeholder_with_nodes_reparents_dynamic_children_under_template_parent() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);

    let parent_fid = renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    renderer.stack.push(ElementId(1));

    let child_fid = renderer.create_frame_for_tag("Button", ElementId(2), &mut registry);
    renderer.stack.push(ElementId(2));
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.replace_placeholder_with_nodes(&[0], 1);
    }

    assert_eq!(
        registry.get(child_fid).unwrap().parent_id,
        Some(parent_fid),
        "dynamic placeholder replacements must attach under the template parent"
    );
    assert!(
        registry.get(parent_fid).unwrap().children.contains(&child_fid),
        "parent must reference the inserted dynamic child"
    );
    assert_eq!(
        renderer.stack,
        vec![ElementId(1)],
        "replace_placeholder_with_nodes must consume inserted nodes from the stack"
    );
}

#[test]
fn insert_nodes_before_reparents_new_siblings_under_existing_parent() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);

    let parent_fid = renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    let existing_fid = renderer.create_frame_for_tag("Button", ElementId(2), &mut registry);
    wire_parent_child(&mut registry, parent_fid, existing_fid);

    let inserted_fid = renderer.create_frame_for_tag("Button", ElementId(3), &mut registry);
    renderer.stack.push(ElementId(3));
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.insert_nodes_before(ElementId(2), 1);
    }

    let parent = registry.get(parent_fid).unwrap();
    assert_eq!(
        registry.get(inserted_fid).unwrap().parent_id,
        Some(parent_fid),
        "inserted siblings must inherit the existing node's parent"
    );
    assert_eq!(
        parent.children,
        vec![inserted_fid, existing_fid],
        "insert_nodes_before must preserve sibling order"
    );
    assert!(
        renderer.stack.is_empty(),
        "insert_nodes_before must consume inserted nodes from the stack"
    );
}

#[test]
fn nested_placeholder_replacement_inherits_outer_placeholder_parent() {
    let mut renderer = GameUiRenderer::new();
    let mut registry = FrameRegistry::new(1024.0, 768.0);

    let parent_fid = renderer.create_frame_for_tag("Frame", ElementId(1), &mut registry);
    renderer.stack.push(ElementId(1));

    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.create_placeholder(ElementId(2));
        applier.create_placeholder(ElementId(3));
    }

    let child_fid = renderer.create_frame_for_tag("Button", ElementId(4), &mut registry);
    renderer.stack.push(ElementId(4));
    {
        let mut applier = MutationApplier::new(&mut renderer, &mut registry);
        applier.replace_node_with(ElementId(3), 1);
    }

    assert_eq!(
        registry.get(child_fid).unwrap().parent_id,
        Some(parent_fid),
        "nested placeholder replacements must still resolve to the nearest real frame parent"
    );
    assert!(
        registry.get(parent_fid).unwrap().children.contains(&child_fid),
        "replacement child must be attached under the outer frame"
    );
}

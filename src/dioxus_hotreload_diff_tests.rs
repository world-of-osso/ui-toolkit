#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use dioxus_core::{TemplateAttribute, TemplateNode};

    use crate::dioxus_renderer::GameUiRenderer;
    use crate::registry::FrameRegistry;

    fn wire_parent_child(registry: &mut FrameRegistry, parent: u64, child: u64) {
        if let Some(p) = registry.get_mut(parent) {
            p.children.push(child);
        }
        if let Some(c) = registry.get_mut(child) {
            c.parent_id = Some(parent);
        }
    }

    fn fid_map(entries: &[(usize, u64)]) -> HashMap<usize, u64> {
        entries.iter().copied().collect()
    }

    // -- Named frame templates --

    static EXISTING_FRAME: &[TemplateNode] = &[TemplateNode::Element {
        tag: "Frame",
        namespace: None,
        attrs: &[TemplateAttribute::Static {
            name: "name",
            value: "TestFrame",
            namespace: None,
        }],
        children: &[],
    }];

    static NEW_FRAME_300: &[TemplateNode] = &[TemplateNode::Element {
        tag: "Frame",
        namespace: None,
        attrs: &[
            TemplateAttribute::Static {
                name: "name",
                value: "TestFrame",
                namespace: None,
            },
            TemplateAttribute::Static {
                name: "width",
                value: "300",
                namespace: None,
            },
        ],
        children: &[],
    }];

    #[test]
    fn diff_named_frame_updates_width() {
        let mut renderer = GameUiRenderer::new();
        let mut registry = FrameRegistry::new(1024.0, 768.0);
        let fid = registry.create_frame("TestFrame", None);
        let map = fid_map(&[(0, fid)]);
        renderer.diff_template(NEW_FRAME_300, EXISTING_FRAME, &map, &[], &mut registry);
        assert!((registry.get(fid).unwrap().width - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn diff_named_frame_preserves_id() {
        let mut renderer = GameUiRenderer::new();
        let mut registry = FrameRegistry::new(1024.0, 768.0);
        let fid = registry.create_frame("TestFrame", None);
        let id_before = registry.get(fid).unwrap().id;
        let map = fid_map(&[(0, fid)]);
        renderer.diff_template(NEW_FRAME_300, EXISTING_FRAME, &map, &[], &mut registry);
        assert_eq!(registry.get(fid).unwrap().id, id_before);
    }

    // -- Parent with child --

    static EXISTING_PARENT: &[TemplateNode] = &[TemplateNode::Element {
        tag: "Frame",
        namespace: None,
        attrs: &[TemplateAttribute::Static {
            name: "name",
            value: "Parent",
            namespace: None,
        }],
        children: &[TemplateNode::Element {
            tag: "Frame",
            namespace: None,
            attrs: &[],
            children: &[],
        }],
    }];

    static NEW_PARENT_WITH_CHILD: &[TemplateNode] = &[TemplateNode::Element {
        tag: "Frame",
        namespace: None,
        attrs: &[
            TemplateAttribute::Static {
                name: "name",
                value: "Parent",
                namespace: None,
            },
            TemplateAttribute::Static {
                name: "width",
                value: "400",
                namespace: None,
            },
        ],
        children: &[TemplateNode::Element {
            tag: "Frame",
            namespace: None,
            attrs: &[TemplateAttribute::Static {
                name: "height",
                value: "50",
                namespace: None,
            }],
            children: &[],
        }],
    }];

    #[test]
    fn diff_recurses_into_children() {
        let mut renderer = GameUiRenderer::new();
        let mut registry = FrameRegistry::new(1024.0, 768.0);
        let parent_fid = registry.create_frame("Parent", None);
        let child_fid = registry.create_frame("", None);
        wire_parent_child(&mut registry, parent_fid, child_fid);
        let map = fid_map(&[(0, parent_fid)]);
        renderer.diff_template(NEW_PARENT_WITH_CHILD, EXISTING_PARENT, &map, &[], &mut registry);
        assert!((registry.get(parent_fid).unwrap().width - 400.0).abs() < f32::EPSILON);
        assert!((registry.get(child_fid).unwrap().height - 50.0).abs() < f32::EPSILON);
    }

    // -- Unnamed tag matching --

    static EXISTING_BUTTONS: &[TemplateNode] = &[
        TemplateNode::Element {
            tag: "button",
            namespace: None,
            attrs: &[],
            children: &[],
        },
        TemplateNode::Element {
            tag: "button",
            namespace: None,
            attrs: &[],
            children: &[],
        },
    ];

    static NEW_BUTTONS_WITH_WIDTH: &[TemplateNode] = &[
        TemplateNode::Element {
            tag: "button",
            namespace: None,
            attrs: &[TemplateAttribute::Static {
                name: "width",
                value: "100",
                namespace: None,
            }],
            children: &[],
        },
        TemplateNode::Element {
            tag: "button",
            namespace: None,
            attrs: &[TemplateAttribute::Static {
                name: "width",
                value: "200",
                namespace: None,
            }],
            children: &[],
        },
    ];

    #[test]
    fn diff_unnamed_nodes_match_by_tag() {
        let mut renderer = GameUiRenderer::new();
        let mut registry = FrameRegistry::new(1024.0, 768.0);
        let fid1 = registry.create_frame("", None);
        let fid2 = registry.create_frame("", None);
        let map = fid_map(&[(0, fid1), (1, fid2)]);
        renderer.diff_template(NEW_BUTTONS_WITH_WIDTH, EXISTING_BUTTONS, &map, &[], &mut registry);
        assert!((registry.get(fid1).unwrap().width - 100.0).abs() < f32::EPSILON);
        assert!((registry.get(fid2).unwrap().width - 200.0).abs() < f32::EPSILON);
    }

    // -- Mixed named + unnamed --

    static EXISTING_MIXED: &[TemplateNode] = &[
        TemplateNode::Element {
            tag: "Frame",
            namespace: None,
            attrs: &[TemplateAttribute::Static {
                name: "name",
                value: "Header",
                namespace: None,
            }],
            children: &[],
        },
        TemplateNode::Element {
            tag: "button",
            namespace: None,
            attrs: &[],
            children: &[],
        },
        TemplateNode::Element {
            tag: "Frame",
            namespace: None,
            attrs: &[TemplateAttribute::Static {
                name: "name",
                value: "Footer",
                namespace: None,
            }],
            children: &[],
        },
    ];

    static NEW_MIXED: &[TemplateNode] = &[
        TemplateNode::Element {
            tag: "Frame",
            namespace: None,
            attrs: &[
                TemplateAttribute::Static {
                    name: "name",
                    value: "Footer",
                    namespace: None,
                },
                TemplateAttribute::Static {
                    name: "width",
                    value: "500",
                    namespace: None,
                },
            ],
            children: &[],
        },
        TemplateNode::Element {
            tag: "button",
            namespace: None,
            attrs: &[TemplateAttribute::Static {
                name: "height",
                value: "30",
                namespace: None,
            }],
            children: &[],
        },
        TemplateNode::Element {
            tag: "Frame",
            namespace: None,
            attrs: &[
                TemplateAttribute::Static {
                    name: "name",
                    value: "Header",
                    namespace: None,
                },
                TemplateAttribute::Static {
                    name: "height",
                    value: "60",
                    namespace: None,
                },
            ],
            children: &[],
        },
    ];

    #[test]
    fn diff_mixed_named_and_unnamed_reorders() {
        let mut renderer = GameUiRenderer::new();
        let mut registry = FrameRegistry::new(1024.0, 768.0);
        let header_fid = registry.create_frame("Header", None);
        let btn_fid = registry.create_frame("", None);
        let footer_fid = registry.create_frame("Footer", None);
        let map = fid_map(&[(0, header_fid), (1, btn_fid), (2, footer_fid)]);
        renderer.diff_template(NEW_MIXED, EXISTING_MIXED, &map, &[], &mut registry);
        // Named nodes match by name regardless of order
        assert!((registry.get(footer_fid).unwrap().width - 500.0).abs() < f32::EPSILON);
        assert!((registry.get(header_fid).unwrap().height - 60.0).abs() < f32::EPSILON);
        // Unnamed button matched by tag
        assert!((registry.get(btn_fid).unwrap().height - 30.0).abs() < f32::EPSILON);
    }

    // -- Removed node gets cleaned up --

    static EXISTING_TWO: &[TemplateNode] = &[
        TemplateNode::Element {
            tag: "Frame",
            namespace: None,
            attrs: &[TemplateAttribute::Static {
                name: "name",
                value: "Keep",
                namespace: None,
            }],
            children: &[],
        },
        TemplateNode::Element {
            tag: "Frame",
            namespace: None,
            attrs: &[TemplateAttribute::Static {
                name: "name",
                value: "Remove",
                namespace: None,
            }],
            children: &[],
        },
    ];

    static NEW_ONE: &[TemplateNode] = &[TemplateNode::Element {
        tag: "Frame",
        namespace: None,
        attrs: &[TemplateAttribute::Static {
            name: "name",
            value: "Keep",
            namespace: None,
        }],
        children: &[],
    }];

    #[test]
    fn diff_removes_unmatched_nodes() {
        let mut renderer = GameUiRenderer::new();
        let mut registry = FrameRegistry::new(1024.0, 768.0);
        let keep_fid = registry.create_frame("Keep", None);
        let remove_fid = registry.create_frame("Remove", None);
        renderer.created_frames.push(remove_fid);
        let map = fid_map(&[(0, keep_fid), (1, remove_fid)]);
        renderer.diff_template(NEW_ONE, EXISTING_TWO, &map, &[], &mut registry);
        assert!(registry.get(keep_fid).is_some());
        assert!(registry.get(remove_fid).is_none());
    }
}

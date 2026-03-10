use std::collections::HashMap;

use dioxus_core::internal::{HotReloadAttributeValue, HotReloadDynamicAttribute, HotReloadLiteral};
use dioxus_core::TemplateNode;

use crate::dioxus_attrs::apply_static_attribute;
use crate::dioxus_renderer::{trace_enabled, GameUiRenderer};
use crate::registry::FrameRegistry;

impl GameUiRenderer {
    /// Apply a hotreload template diff directly to existing frames, bypassing Dioxus.
    pub(crate) fn diff_template(
        &mut self,
        new_roots: &[TemplateNode],
        existing_roots: &[TemplateNode],
        fid_map: &HashMap<usize, u64>,
        hr_dynamic_attrs: &[HotReloadDynamicAttribute],
        registry: &mut FrameRegistry,
    ) {
        let trace = trace_enabled();
        let mut remaining: Vec<Option<(&TemplateNode, u64)>> = existing_roots
            .iter()
            .enumerate()
            .map(|(i, node)| fid_map.get(&i).map(|&fid| (node, fid)))
            .collect();
        for (i, new_node) in new_roots.iter().enumerate() {
            if matches!(new_node, TemplateNode::Dynamic { .. }) {
                continue;
            }
            if let Some(matched) = self.consume_match(new_node, &mut remaining, registry) {
                self.diff_node(new_node, matched, hr_dynamic_attrs, registry);
            } else if trace {
                eprintln!("[hotreload] no match at index {i}: {}", describe_node(new_node));
            }
        }
        for slot in &remaining {
            if let Some((_node, fid)) = slot {
                if trace {
                    eprintln!("[hotreload] removing unmatched fid={fid}");
                }
                self.remove_frame_tree(*fid, registry);
            }
        }
    }

    fn insert_node(
        &mut self,
        node: &TemplateNode,
        parent_fid: u64,
        _index: usize,
        registry: &mut FrameRegistry,
    ) {
        let TemplateNode::Element { attrs, children, .. } = node else {
            return;
        };
        let name = extract_static_name(node).unwrap_or("");
        let fid = registry.create_frame(name, Some(parent_fid));
        self.apply_static_attrs_for_node(attrs, fid, registry);
        for child in *children {
            self.insert_node(child, fid, 0, registry);
        }
    }

    fn apply_static_attrs_for_node(
        &mut self,
        attrs: &[dioxus_core::TemplateAttribute],
        fid: u64,
        registry: &mut FrameRegistry,
    ) {
        for attr in attrs {
            if let dioxus_core::TemplateAttribute::Static {
                name, value, namespace,
            } = attr
            {
                apply_static_attribute(
                    registry, fid, name, *namespace, value,
                    &mut self.pending_anchors,
                    &mut self.validated_paths,
                    &mut self.missing_paths,
                );
            }
        }
    }

    fn consume_match<'a>(
        &self,
        new_node: &TemplateNode,
        remaining: &mut [Option<(&'a TemplateNode, u64)>],
        registry: &FrameRegistry,
    ) -> Option<(&'a TemplateNode, u64)> {
        let trace = trace_enabled();
        if let Some(name) = extract_static_name(new_node) {
            for slot in remaining.iter_mut() {
                let Some((existing, fid)) = slot else { continue };
                if extract_static_name(existing) == Some(name) {
                    let result = (*existing, *fid);
                    *slot = None;
                    return Some(result);
                }
            }
            if trace {
                eprintln!("[hotreload] name mismatch: {name:?}");
            }
            return None;
        }
        let new_tag = node_tag(new_node);
        for slot in remaining.iter_mut() {
            let Some((existing, fid)) = slot else { continue };
            if extract_static_name(existing).is_some() {
                continue;
            }
            if tags_match(new_tag, node_tag(existing)) {
                let result = (*existing, *fid);
                *slot = None;
                return Some(result);
            }
        }
        None
    }

    fn diff_node(
        &mut self,
        new_node: &TemplateNode,
        (existing_node, fid): (&TemplateNode, u64),
        hr_dynamic_attrs: &[HotReloadDynamicAttribute],
        registry: &mut FrameRegistry,
    ) {
        let TemplateNode::Element {
            attrs: new_attrs,
            children: new_children,
            ..
        } = new_node
        else {
            return;
        };
        let TemplateNode::Element {
            attrs: existing_attrs,
            children: existing_children,
            ..
        } = existing_node
        else {
            return;
        };
        self.diff_attrs(new_attrs, existing_attrs, fid, hr_dynamic_attrs, registry);
        self.diff_children(new_children, existing_children, fid, hr_dynamic_attrs, registry);
    }

    fn diff_attrs(
        &mut self,
        new_attrs: &[dioxus_core::TemplateAttribute],
        existing_attrs: &[dioxus_core::TemplateAttribute],
        fid: u64,
        hr_dynamic_attrs: &[HotReloadDynamicAttribute],
        registry: &mut FrameRegistry,
    ) {
        let trace = trace_enabled();
        for attr in new_attrs {
            match attr {
                dioxus_core::TemplateAttribute::Static { name, value, namespace } => {
                    let old_value = find_static_attr(existing_attrs, name);
                    if old_value != Some(value) && trace {
                        let frame_name = registry.get(fid).and_then(|f| f.name.as_deref()).unwrap_or("?");
                        eprintln!("[hotreload] {frame_name}.{name}: {:?} -> {value:?}", old_value.unwrap_or("(none)"));
                    }
                    apply_static_attribute(
                        registry, fid, name, *namespace, value,
                        &mut self.pending_anchors,
                        &mut self.validated_paths,
                        &mut self.missing_paths,
                    );
                }
                dioxus_core::TemplateAttribute::Dynamic { id } => {
                    self.apply_dynamic_attr(*id, fid, hr_dynamic_attrs, registry);
                }
            }
        }
    }

    fn apply_dynamic_attr(
        &mut self,
        id: usize,
        fid: u64,
        hr_dynamic_attrs: &[HotReloadDynamicAttribute],
        registry: &mut FrameRegistry,
    ) {
        let Some(hr_attr) = hr_dynamic_attrs.get(id) else {
            return;
        };
        let HotReloadDynamicAttribute::Named(named) = hr_attr else {
            return; // Dynamic(id) refers to a runtime value we can't resolve
        };
        let value_str = match &named.value {
            HotReloadAttributeValue::Literal(lit) => literal_to_string(lit),
            HotReloadAttributeValue::Dynamic(_) => return,
        };
        let attr_name = named.name.replace('-', "_");
        let cache_key = (fid, attr_name.clone());
        if self.dynamic_attr_cache.get(&cache_key).map(|s| s.as_str()) == Some(value_str.as_str()) {
            return;
        }
        let trace = trace_enabled();
        if trace {
            let old = self.dynamic_attr_cache.get(&cache_key).map(|s| s.as_str());
            let frame_name = registry.get(fid).and_then(|f| f.name.as_deref()).unwrap_or("?");
            eprintln!("[hotreload] {frame_name}.{}: {:?} -> {value_str:?} (dynamic)",
                attr_name, old.unwrap_or("(none)"));
        }
        self.dynamic_attr_cache.insert(cache_key, value_str.clone());
        apply_static_attribute(
            registry, fid, &attr_name, named.namespace,
            &value_str,
            &mut self.pending_anchors,
            &mut self.validated_paths,
            &mut self.missing_paths,
        );
    }

    fn diff_children(
        &mut self,
        new_children: &[TemplateNode],
        existing_children: &[TemplateNode],
        fid: u64,
        hr_dynamic_attrs: &[HotReloadDynamicAttribute],
        registry: &mut FrameRegistry,
    ) {
        let child_fids = registry.children_of(fid);
        for (i, (new_child, existing_child)) in
            new_children.iter().zip(existing_children.iter()).enumerate()
        {
            if let Some(&child_fid) = child_fids.get(i) {
                self.diff_node(new_child, (existing_child, child_fid), hr_dynamic_attrs, registry);
            }
        }
    }
}

fn literal_to_string(lit: &HotReloadLiteral) -> String {
    match lit {
        HotReloadLiteral::Float(f) => f.to_string(),
        HotReloadLiteral::Int(i) => i.to_string(),
        HotReloadLiteral::Bool(b) => b.to_string(),
        HotReloadLiteral::Fmted(segments) => format!("{segments:?}"),
    }
}

/// Case-insensitive tag comparison, stripping `r#` prefix.
fn tags_match(a: Option<&str>, b: Option<&str>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => strip_raw(a).eq_ignore_ascii_case(strip_raw(b)),
        (None, None) => true,
        _ => false,
    }
}

fn strip_raw(tag: &str) -> &str {
    tag.strip_prefix("r#").unwrap_or(tag)
}

fn find_static_attr<'a>(attrs: &'a [dioxus_core::TemplateAttribute], name: &str) -> Option<&'a str> {
    attrs.iter().find_map(|a| match a {
        dioxus_core::TemplateAttribute::Static { name: n, value, .. } if *n == name => Some(*value),
        _ => None,
    })
}

fn describe_node(node: &TemplateNode) -> String {
    match node {
        TemplateNode::Element { tag, attrs, .. } => {
            let static_attrs: Vec<String> = attrs
                .iter()
                .filter_map(|a| match a {
                    dioxus_core::TemplateAttribute::Static { name, value, .. } => {
                        Some(format!("{name}={value:?}"))
                    }
                    _ => None,
                })
                .collect();
            let dyn_count = attrs.iter().filter(|a| matches!(a, dioxus_core::TemplateAttribute::Dynamic { .. })).count();
            let mut desc = format!("<{tag}");
            if !static_attrs.is_empty() {
                desc.push_str(&format!(" {}", static_attrs.join(" ")));
            }
            if dyn_count > 0 {
                desc.push_str(&format!(" +{dyn_count} dynamic"));
            }
            desc.push('>');
            desc
        }
        TemplateNode::Dynamic { id } => format!("Dynamic({id})"),
        TemplateNode::Text { text } => format!("Text({text:?})"),
    }
}

/// Extract the tag from a template node, if it's an element.
fn node_tag(node: &TemplateNode) -> Option<&str> {
    match node {
        TemplateNode::Element { tag, .. } => Some(tag),
        _ => None,
    }
}

/// Extract the static "name" attribute from a template element node.
pub(crate) fn extract_static_name(node: &TemplateNode) -> Option<&str> {
    let TemplateNode::Element { attrs, .. } = node else {
        return None;
    };
    for attr in *attrs {
        if let dioxus_core::TemplateAttribute::Static {
            name: "name",
            value,
            ..
        } = attr
        {
            return Some(value);
        }
    }
    None
}

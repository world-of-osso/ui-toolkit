use std::sync::mpsc;

use dioxus_core::{Element, ScopeId, VirtualDom};
use dioxus_devtools::DevserverMsg;

use crate::dioxus_renderer::{GameUiRenderer, MutationApplier};
use crate::frame::WidgetData;
use crate::registry::FrameRegistry;
use crate::text_measure::measure_text;

/// Generic Dioxus-to-FrameRegistry bridge. One per screen.
pub struct DioxusScreen {
    dom: VirtualDom,
    renderer: GameUiRenderer,
    initialized: bool,
    hot_reload_rx: mpsc::Receiver<DevserverMsg>,
}

impl DioxusScreen {
    pub fn new(component: fn() -> Element) -> Self {
        let (tx, rx) = mpsc::channel();
        dioxus_devtools::connect(move |msg| {
            let _ = tx.send(msg);
        });
        Self {
            dom: VirtualDom::new(component),
            renderer: GameUiRenderer::new(),
            initialized: false,
            hot_reload_rx: rx,
        }
    }

    pub fn sync(&mut self, registry: &mut FrameRegistry) {
        while let Ok(msg) = self.hot_reload_rx.try_recv() {
            if let DevserverMsg::HotReload(hr_msg) = msg {
                self.apply_hotreload(hr_msg, registry);
            }
        }
        {
            let mut applier = MutationApplier::new(&mut self.renderer, registry);
            if self.initialized {
                self.dom.render_immediate(&mut applier);
            } else {
                self.dom.rebuild(&mut applier);
                self.initialized = true;
            }
        }
        self.renderer.resolve_pending_anchors(registry);
        auto_size_fontstrings(&self.renderer, registry);
        auto_size_editboxes(&self.renderer, registry);
    }

    pub fn renderer(&self) -> &GameUiRenderer {
        &self.renderer
    }

    pub fn provide_root_context<T: Clone + 'static>(&self, context: T) {
        self.dom.provide_root_context(context);
    }

    pub fn mark_dirty_root(&mut self) {
        self.dom.mark_dirty(ScopeId::APP);
    }

    /// Apply hotreload: diff each template against existing frames.
    fn apply_hotreload(
        &mut self,
        hr_msg: dioxus_devtools::HotReloadMsg,
        registry: &mut FrameRegistry,
    ) {
        for hr_template in &hr_msg.templates {
            if let Some((existing_roots, fid_map)) = self.renderer.templates_by_key.get(&hr_template.key) {
                let existing_roots = *existing_roots;
                let fid_map = fid_map.clone();
                self.renderer.diff_template(
                    hr_template.template.roots,
                    existing_roots,
                    &fid_map,
                    &hr_template.template.dynamic_attributes,
                    registry,
                );
                self.renderer
                    .templates_by_key
                    .entry(hr_template.key.clone())
                    .and_modify(|(roots, _fids)| *roots = hr_template.template.roots);
            }
        }
    }

    pub fn teardown(&mut self, registry: &mut FrameRegistry) {
        for fid in self.renderer.all_frame_ids() {
            registry.remove_frame(fid);
        }
        self.renderer = GameUiRenderer::new();
        self.initialized = false;
    }
}

/// Auto-size editbox frames that have height == 0 based on font size + vertical insets.
fn auto_size_editboxes(renderer: &GameUiRenderer, registry: &mut FrameRegistry) {
    for fid in renderer.all_frame_ids() {
        let Some(frame) = registry.get(fid) else {
            continue;
        };
        if frame.height > 0.0 {
            continue;
        }
        let Some(WidgetData::EditBox(eb)) = &frame.widget_data else {
            continue;
        };
        let font_size = eb.font_size;
        let v_inset = if eb.text_insets != [0.0; 4] {
            eb.text_insets[2] + eb.text_insets[3]
        } else {
            0.0
        };
        let frame = registry.get_mut(fid).unwrap();
        frame.height = font_size + font_size * 0.5 + v_inset;
    }
}

/// Auto-size fontstring frames that have width == 0 by measuring their text.
fn auto_size_fontstrings(renderer: &GameUiRenderer, registry: &mut FrameRegistry) {
    for fid in renderer.all_frame_ids() {
        let Some(frame) = registry.get(fid) else {
            continue;
        };
        let Some(WidgetData::FontString(fs)) = &frame.widget_data else {
            continue;
        };
        if frame.width > 0.0 || fs.text.is_empty() {
            continue;
        }
        let text = fs.text.clone();
        let font = fs.font;
        let font_size = fs.font_size;
        if let Some((w, h)) = measure_text(&text, font, font_size) {
            let frame = registry.get_mut(fid).unwrap();
            frame.width = w;
            frame.height = h;
        }
    }
}


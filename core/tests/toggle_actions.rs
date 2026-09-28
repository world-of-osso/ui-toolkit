use ui_toolkit_core::frame::WidgetData;
use ui_toolkit_core::registry::FrameRegistry;
use ui_toolkit_core::screen::{Screen, SharedContext};
use ui_toolkit_core::widget_def::Element;
use ui_toolkit_core::widgets::toggle::{ToggleWidget, toggle_widget};

fn mute_toggle(shared: &SharedContext) -> Element {
    toggle_widget(ToggleWidget {
        name: "Mute",
        action: "options_toggle:muted",
        right_selected: *shared.get::<bool>().unwrap(),
        width: 170.0,
        height: 28.0,
        left_label: "Off",
        right_label: "On",
        background_color: "0,0,0,1",
        active_color: "1,1,1,1",
        border: "1px solid 1,1,1,1",
        active_text_color: "1,1,1,1",
        idle_text_color: "0.5,0.5,0.5,1",
        x: "-8",
    })
}

#[test]
fn only_inactive_toggle_segment_emits_action_after_state_changes() {
    let mut registry = FrameRegistry::new(1280.0, 720.0);
    let mut shared = SharedContext::new();
    let mut screen = Screen::new(mute_toggle);
    for selected in [false, true, false] {
        shared.insert(selected);
        screen.sync(&shared, &mut registry);
        let (inactive, active) = if selected {
            ("Left", "Right")
        } else {
            ("Right", "Left")
        };
        let hit = registry.get_by_name(&format!("Mute{inactive}Hit")).unwrap();
        assert_eq!(
            registry.click_frame(hit).as_deref(),
            Some("options_toggle:muted")
        );
        assert!(registry.get_by_name(&format!("Mute{active}Hit")).is_none());
        for (side, expected) in [("Left", "Off"), ("Right", "On")] {
            let frame = registry
                .get(registry.get_by_name(&format!("Mute{side}Label")).unwrap())
                .unwrap();
            assert!(
                matches!(&frame.widget_data, Some(WidgetData::FontString(text)) if text.text == expected)
            );
        }
    }
}

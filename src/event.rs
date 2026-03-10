use std::collections::{HashMap, HashSet};

/// A single argument passed with a UI event.
#[derive(Debug, Clone)]
pub enum EventArg {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

/// A UI event with a name and optional arguments.
#[derive(Debug, Clone)]
pub struct UiEvent {
    pub name: String,
    pub args: Vec<EventArg>,
}

/// All WoW-style script handler hooks a frame can register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptHandler {
    OnLoad,
    OnShow,
    OnHide,
    OnUpdate,
    OnSizeChanged,
    OnClick,
    PreClick,
    PostClick,
    OnDoubleClick,
    OnEnter,
    OnLeave,
    OnMouseDown,
    OnMouseUp,
    OnMouseWheel,
    OnDragStart,
    OnDragStop,
    OnReceiveDrag,
    OnKeyDown,
    OnKeyUp,
    OnChar,
    OnEnterPressed,
    OnEscapePressed,
    OnTabPressed,
    OnSpacePressed,
    OnEditFocusGained,
    OnEditFocusLost,
    OnTextChanged,
    OnValueChanged,
    OnMinMaxChanged,
    OnVerticalScroll,
    OnHorizontalScroll,
    OnScrollRangeChanged,
    OnEvent,
    OnAttributeChanged,
    OnTooltipSetItem,
    OnTooltipSetUnit,
    OnTooltipSetSpell,
    OnTooltipCleared,
    OnCooldownDone,
    OnModelLoaded,
    OnModelCleared,
    OnAnimFinished,
    OnAnimLoop,
    OnAnimPlay,
    OnAnimStop,
    OnPostShow,
    OnPostHide,
    OnPostUpdate,
}

/// Central bus for dispatching named events to registered frame listeners.
pub struct EventBus {
    registrations: HashMap<String, HashSet<u64>>,
    queue: Vec<UiEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            registrations: HashMap::new(),
            queue: Vec::new(),
        }
    }

    /// Register a frame to listen for a named event.
    pub fn register(&mut self, frame_id: u64, event: &str) {
        self.registrations
            .entry(event.to_string())
            .or_default()
            .insert(frame_id);
    }

    /// Remove a frame from listeners of a named event.
    pub fn unregister(&mut self, frame_id: u64, event: &str) {
        if let Some(set) = self.registrations.get_mut(event) {
            set.remove(&frame_id);
        }
    }

    /// Remove a frame from all event registrations.
    pub fn unregister_all(&mut self, frame_id: u64) {
        for set in self.registrations.values_mut() {
            set.remove(&frame_id);
        }
    }

    /// Return a sorted list of frame IDs listening for the given event.
    pub fn listeners(&self, event: &str) -> Vec<u64> {
        let Some(set) = self.registrations.get(event) else {
            return Vec::new();
        };
        let mut ids: Vec<u64> = set.iter().copied().collect();
        ids.sort_unstable();
        ids
    }

    /// Enqueue an event for later processing.
    pub fn push(&mut self, event: UiEvent) {
        self.queue.push(event);
    }

    /// Take all pending events, leaving the queue empty.
    pub fn drain(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.queue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_frames_same_event() {
        let mut bus = EventBus::new();
        bus.register(10, "PLAYER_LOGIN");
        bus.register(5, "PLAYER_LOGIN");
        assert_eq!(bus.listeners("PLAYER_LOGIN"), vec![5, 10]);
    }

    #[test]
    fn one_frame_two_events() {
        let mut bus = EventBus::new();
        bus.register(1, "PLAYER_LOGIN");
        bus.register(1, "UNIT_HEALTH");
        assert_eq!(bus.listeners("PLAYER_LOGIN"), vec![1]);
        assert_eq!(bus.listeners("UNIT_HEALTH"), vec![1]);
    }

    #[test]
    fn unregister_one_frame() {
        let mut bus = EventBus::new();
        bus.register(1, "PLAYER_LOGIN");
        bus.register(2, "PLAYER_LOGIN");
        bus.unregister(1, "PLAYER_LOGIN");
        assert_eq!(bus.listeners("PLAYER_LOGIN"), vec![2]);
    }

    #[test]
    fn unregister_all_removes_from_all_events() {
        let mut bus = EventBus::new();
        bus.register(1, "PLAYER_LOGIN");
        bus.register(1, "UNIT_HEALTH");
        bus.unregister_all(1);
        assert!(bus.listeners("PLAYER_LOGIN").is_empty());
        assert!(bus.listeners("UNIT_HEALTH").is_empty());
    }

    #[test]
    fn push_drain_ordering() {
        let mut bus = EventBus::new();
        bus.push(UiEvent {
            name: "A".to_string(),
            args: vec![],
        });
        bus.push(UiEvent {
            name: "B".to_string(),
            args: vec![EventArg::Number(42.0)],
        });
        let events = bus.drain();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].name, "A");
        assert_eq!(events[1].name, "B");
        assert!(bus.drain().is_empty());
    }
}

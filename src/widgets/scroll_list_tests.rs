use super::*;
use crate::frame::WidgetData;
use crate::registry::FrameRegistry;
use crate::screen::Screen;

/// Row count read by the list screen; inserting it rebuilds the screen.
struct Rows(usize);

fn list_screen() -> Screen {
    Screen::new(|ctx| {
        let count = ctx.get::<Rows>().map_or(0, |rows| rows.0);
        scroll_list(
            ctx,
            ScrollList {
                name: "Quests",
                width: 200.0,
                height: 100.0,
                row_height: 20.0,
                row_count: count,
                track_width: 16.0,
                track_fdid: None,
                thumb_fdid: Some(130_841),
            },
            |index| {
                rsx! {
                    fontstring {
                        name: DynName(format!("QuestLabel{index}")),
                        text: {format!("Quest {index}")},
                    }
                }
            },
        )
    })
}

fn setup(rows: usize) -> (Screen, SharedContext, FrameRegistry) {
    let mut ctx = SharedContext::new();
    ctx.insert(Rows(rows));
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let mut screen = list_screen();
    screen.sync(&ctx, &mut registry);
    (screen, ctx, registry)
}

fn visible_rows(registry: &FrameRegistry) -> Vec<usize> {
    let list = registry.get_by_name("Quests").unwrap();
    let mut rows: Vec<usize> = registry
        .children_of(list)
        .into_iter()
        .filter_map(|id| {
            registry
                .get(id)?
                .name
                .as_deref()?
                .strip_prefix("QuestsRow")?
                .parse()
                .ok()
        })
        .collect();
    rows.sort_unstable();
    rows
}

fn label_text(registry: &FrameRegistry, index: usize) -> String {
    let id = registry.get_by_name(&format!("QuestLabel{index}")).unwrap();
    match &registry.get(id).unwrap().widget_data {
        Some(WidgetData::FontString(fs)) => fs.text.clone(),
        _ => panic!("label is not a fontstring"),
    }
}

fn frame_top(registry: &FrameRegistry, name: &str) -> bevy::prelude::Val {
    registry
        .get(registry.get_by_name(name).unwrap())
        .unwrap()
        .position
        .top
}

#[test]
fn only_viewport_rows_exist_as_frames() {
    let (_, _, registry) = setup(100);
    assert_eq!(visible_rows(&registry), vec![0, 1, 2, 3, 4]);
    assert_eq!(label_text(&registry, 4), "Quest 4");
    assert!(registry.get_by_name("QuestLabel5").is_none());
    assert_eq!(
        frame_top(&registry, "QuestsRow3"),
        bevy::prelude::Val::Px(60.0)
    );
    let thumb = registry
        .get(registry.get_by_name("QuestsScrollThumb").unwrap())
        .unwrap();
    assert_eq!(thumb.height, crate::frame::Dimension::Fixed(16.0));
    assert!(registry.get_by_name("QuestsScrollThumbArt").is_some());
}

#[test]
fn scrolling_rebuilds_rows_at_new_offset() {
    let (mut screen, ctx, mut registry) = setup(100);
    assert!(registry.scroll_lists.scroll_to("Quests", 3));
    screen.sync(&ctx, &mut registry);
    assert_eq!(visible_rows(&registry), vec![3, 4, 5, 6, 7]);
    assert!(registry.get_by_name("QuestLabel0").is_none());
    assert_eq!(label_text(&registry, 7), "Quest 7");
    assert_eq!(
        frame_top(&registry, "QuestsRow3"),
        bevy::prelude::Val::Px(0.0)
    );
    // 84px of thumb travel over 95 scrollable rows.
    let bevy::prelude::Val::Px(top) = frame_top(&registry, "QuestsScrollThumb") else {
        panic!("thumb top is not pixels");
    };
    assert!((top - 84.0 * 3.0 / 95.0).abs() < 0.001);
}

#[test]
fn offset_clamps_to_content_and_shrinking_content() {
    let (mut screen, mut ctx, mut registry) = setup(100);
    registry.scroll_lists.scroll_to("Quests", 1_000);
    screen.sync(&ctx, &mut registry);
    assert_eq!(visible_rows(&registry), vec![95, 96, 97, 98, 99]);
    assert!(!registry.scroll_lists.scroll_by("Quests", 1));

    ctx.insert(Rows(3));
    screen.sync(&ctx, &mut registry);
    assert_eq!(visible_rows(&registry), vec![0, 1, 2]);
    assert_eq!(registry.scroll_lists.get("Quests").unwrap().first_row, 0);
    let track = registry
        .get(registry.get_by_name("QuestsScrollTrack").unwrap())
        .unwrap();
    assert!(!track.visible, "track hides when all rows fit");
}

#[test]
fn position_survives_rebuild_and_screen_recreation() {
    let (mut screen, mut ctx, mut registry) = setup(100);
    registry.scroll_lists.scroll_to("Quests", 10);
    screen.sync(&ctx, &mut registry);
    ctx.insert(Rows(100));
    screen.sync(&ctx, &mut registry);
    assert_eq!(visible_rows(&registry), vec![10, 11, 12, 13, 14]);

    screen.teardown(&mut registry);
    assert!(registry.get_by_name("Quests").is_none());
    let mut reopened = list_screen();
    reopened.sync(&ctx, &mut registry);
    assert_eq!(visible_rows(&registry), vec![10, 11, 12, 13, 14]);
}

#[test]
fn unchanged_position_does_not_rebuild() {
    let (mut screen, ctx, mut registry) = setup(100);
    let row = registry.get_by_name("QuestsRow0").unwrap();
    assert!(!registry.scroll_lists.scroll_to("Quests", 0));
    registry.render_dirty.clear();
    screen.sync(&ctx, &mut registry);
    assert_eq!(registry.get_by_name("QuestsRow0"), Some(row));
    assert!(registry.render_dirty.is_empty());
}

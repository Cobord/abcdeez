// Core task visual representations

use xilem::view::*;
use xilem::{WidgetView, Style};
use xilem::{FontWeight, Color};
use abcdeez_core::tasks::core::TaskType as CoreTaskType;
use crate::state::AppState;

pub fn core_task_visual(
    core_task: &CoreTaskType,
    primary: Color,
    surface: Color,
) -> impl WidgetView<AppState> {
    use CoreTaskType::*;
    
    sized_box(match core_task {
        PairwiseOrder { a, b } => {
            flex_row((
                sized_box(label(a.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(surface)
                    .corner_radius(12.0),
                label("?").text_size(24.0).color(primary),
                sized_box(label(b.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(surface)
                    .corner_radius(12.0),
            ))
            .gap(20.0)
            .main_axis_alignment(MainAxisAlignment::Center)
        },
        Successor { item } | Predecessor { item } | Index { item } => {
            flex_row((
                sized_box(label(item.clone()).text_size(32.0).weight(FontWeight::BOLD))
                    .width(120.0)
                    .height(120.0)
                    .background(surface)
                    .border(primary, 2.0)
                    .corner_radius(15.0),
            ))
            .main_axis_alignment(MainAxisAlignment::Center)
        },
        KJump { start, k } => {
            let arrow = if *k > 0 { "→".repeat(*k as usize) } else { "←".repeat(k.abs() as usize) };
            flex_row((
                sized_box(label(start.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(surface)
                    .corner_radius(12.0),
                label(arrow).text_size(20.0).color(primary),
                sized_box(label("?").text_size(32.0).color(primary))
                    .width(100.0)
                    .height(80.0)
                    .border(primary, 3.0)
                    .corner_radius(12.0),
            ))
            .gap(15.0)
            .main_axis_alignment(MainAxisAlignment::Center)
        },
        Segment { start, count, reverse } => {
            let direction_icon = if *reverse { "⟵" } else { "⟶" };
            flex((
                label(format!("Starting from: {}", start))
                    .text_size(20.0)
                    .weight(FontWeight::MEDIUM),
                FlexSpacer::Fixed(10.0),
                label(format!("{} {} items", direction_icon, count))
                    .text_size(18.0)
                    .color(primary),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Center)
        },
        MissingItem { before, after } => {
            flex_row((
                sized_box(label(before.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(surface)
                    .corner_radius(12.0),
                sized_box(label("?").text_size(32.0).color(primary))
                    .width(100.0)
                    .height(80.0)
                    .border(primary, 3.0)
                    .corner_radius(12.0),
                sized_box(label(after.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(surface)
                    .corner_radius(12.0),
            ))
            .gap(15.0)
            .main_axis_alignment(MainAxisAlignment::Center)
        },
        ShortestDistance { from, to } | ShortestPath { from, to } => {
            flex_row((
                sized_box(label(from.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(primary.with_alpha(0.2))
                    .border(primary, 2.0)
                    .corner_radius(12.0),
                label("⟶").text_size(24.0).color(primary),
                sized_box(label(to.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(primary.with_alpha(0.2))
                    .border(primary, 2.0)
                    .corner_radius(12.0),
            ))
            .gap(20.0)
            .main_axis_alignment(MainAxisAlignment::Center)
        },
        Comparability { a, b } => {
            flex_row((
                sized_box(label(a.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(surface)
                    .corner_radius(12.0),
                label("⟷").text_size(24.0).color(primary),
                sized_box(label(b.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(surface)
                    .corner_radius(12.0),
            ))
            .gap(20.0)
            .main_axis_alignment(MainAxisAlignment::Center)
        },
        TopologicalSort { items } => {
            let item_views: Vec<_> = items.iter().take(5).map(|item| {
                sized_box(label(item.clone()).text_size(20.0))
                    .width(60.0)
                    .height(60.0)
                    .background(surface)
                    .corner_radius(10.0)
            }).collect();
            flex_row(item_views)
                .gap(10.0)
                .main_axis_alignment(MainAxisAlignment::Center)
        },
        MinimalElements | MaximalElements => {
            flex((
                label("Select from the options below")
                    .text_size(20.0)
                    .color(primary),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Center)
        },
    })
}
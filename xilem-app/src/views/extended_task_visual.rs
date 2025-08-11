// Extended task visual representations

use xilem::view::*;
use xilem::{WidgetView, AnyWidgetView};
use xilem::style::{Style, Background};
use xilem::{FontWeight, Color};
use xilem::core::one_of::OneOf;
use abcdeez_core::tasks::extended::ExtendedTaskType;
use crate::state::AppState;

pub fn extended_task_visual(
    extended_task: &ExtendedTaskType,
    primary: Color,
    surface: Color,
) -> impl WidgetView<AppState> {
    use ExtendedTaskType::*;
    
    // Use OneOf to handle different view types
    // MacroDiscovery, SemanticFilter, ProjectionSwitch, IsomorphicTransfer share OneOf::I
    match extended_task {
        BetweenQuery { a, b, c } => {
            OneOf::A(flex_row((
                sized_box(label(a.clone()).text_size(24.0))
                    .width(80.0)
                    .height(80.0)
                    .background(Background::Color(surface))
                    .corner_radius(10.0),
                label("?").text_size(20.0).color(primary),
                sized_box(label(b.clone()).text_size(24.0))
                    .width(80.0)
                    .height(80.0)
                    .background(Background::Color(primary.with_alpha(0.2)))
                    .border(primary, 2.0)
                    .corner_radius(10.0),
                label("?").text_size(20.0).color(primary),
                sized_box(label(c.clone()).text_size(24.0))
                    .width(80.0)
                    .height(80.0)
                    .background(Background::Color(surface))
                    .corner_radius(10.0),
            ))
            .gap(15.0)
            .main_axis_alignment(MainAxisAlignment::Center))
        },
        BoundaryBridging { start, count, boundaries } => {
            let boundary_markers = boundaries.iter().take(3)
                .map(|&b| format!("[{}]", b))
                .collect::<Vec<_>>()
                .join(" ");
            OneOf::B(flex((
                label(format!("Start: {}", start))
                    .text_size(20.0)
                    .weight(FontWeight::MEDIUM),
                FlexSpacer::Fixed(10.0),
                label(format!("Cross {} items", count))
                    .text_size(18.0),
                FlexSpacer::Fixed(10.0),
                label(format!("Boundaries: {}", boundary_markers))
                    .text_size(16.0)
                    .color(primary),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Center))
        },
        ReverseNTreadmill { start, n, steps } => {
            OneOf::C(flex((
                label(format!("Start: {}", start))
                    .text_size(20.0),
                FlexSpacer::Fixed(10.0),
                label(format!("Reverse every {} items", n))
                    .text_size(18.0)
                    .color(primary),
                FlexSpacer::Fixed(10.0),
                label(format!("Total steps: {}", steps))
                    .text_size(16.0),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Center))
        },
        DirectionalComparison { a, b, backward } => {
            let arrow = if *backward { "←" } else { "→" };
            OneOf::D(flex_row((
                sized_box(label(a.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(Background::Color(surface))
                    .corner_radius(12.0),
                label(arrow).text_size(24.0).color(primary),
                sized_box(label(b.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(Background::Color(surface))
                    .corner_radius(12.0),
            ))
            .gap(20.0)
            .main_axis_alignment(MainAxisAlignment::Center))
        },
        InsertionAdaptation { item, after, before } => {
            OneOf::E(flex_row((
                sized_box(label(after.clone()).text_size(24.0))
                    .width(80.0)
                    .height(80.0)
                    .background(Background::Color(surface))
                    .corner_radius(10.0),
                label("→").text_size(20.0).color(primary),
                sized_box(label(format!("[{}]", item)).text_size(20.0).color(primary))
                    .width(80.0)
                    .height(80.0)
                    .border(primary, 2.0)
                    .corner_radius(10.0),
                label("→").text_size(20.0).color(primary),
                sized_box(label(before.clone()).text_size(24.0))
                    .width(80.0)
                    .height(80.0)
                    .background(Background::Color(surface))
                    .corner_radius(10.0),
            ))
            .gap(10.0)
            .main_axis_alignment(MainAxisAlignment::Center))
        },
        LinearExtensionGeneration { partial_order } => {
            let constraints = partial_order.iter().take(3)
                .map(|(a, b)| format!("{}<{}", a, b))
                .collect::<Vec<_>>()
                .join(", ");
            OneOf::F(flex((
                label("Order constraints:")
                    .text_size(18.0),
                FlexSpacer::Fixed(10.0),
                label(constraints)
                    .text_size(20.0)
                    .color(primary)
                    .weight(FontWeight::MEDIUM),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Center))
        },
        NextStepPrediction { current, goal } => {
            OneOf::G(flex_row((
                sized_box(label(current.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(Background::Color(primary.with_alpha(0.3)))
                    .border(primary, 2.0)
                    .corner_radius(12.0),
                label("→ ? →").text_size(20.0).color(primary),
                sized_box(label(goal.clone()).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background(Background::Color(surface))
                    .corner_radius(12.0),
            ))
            .gap(20.0)
            .main_axis_alignment(MainAxisAlignment::Center))
        },
        LandmarkNavigation { start, end, landmark } => {
            OneOf::H(flex((
                flex_row((
                    sized_box(label(start.clone()).text_size(24.0))
                        .width(80.0)
                        .height(60.0)
                        .background(Background::Color(surface))
                        .corner_radius(10.0),
                    label("→").text_size(20.0).color(primary),
                    sized_box(label(landmark.clone()).text_size(24.0).weight(FontWeight::BOLD))
                        .width(80.0)
                        .height(60.0)
                        .background(Background::Color(primary.with_alpha(0.2)))
                        .border(primary, 2.0)
                        .corner_radius(10.0),
                    label("→").text_size(20.0).color(primary),
                    sized_box(label(end.clone()).text_size(24.0))
                        .width(80.0)
                        .height(60.0)
                        .background(Background::Color(surface))
                        .corner_radius(10.0),
                ))
                .gap(10.0)
                .main_axis_alignment(MainAxisAlignment::Center),
            ))
            .direction(Axis::Vertical))
        },
        MacroDiscovery { sequence } => {
            let seq_display = sequence.iter().take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join("-");
            OneOf::I(flex((
                label("Find the pattern:")
                    .text_size(18.0),
                FlexSpacer::Fixed(15.0),
                sized_box(label(seq_display).text_size(24.0).weight(FontWeight::MEDIUM))
                    .padding(20.0)
                    .background(Background::Color(surface))
                    .corner_radius(10.0),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .boxed())
        },
        SemanticFilter { category, position } => {
            // Share OneOf::I with boxed types
            OneOf::I(flex((
                label(format!("Category: {}", category))
                    .text_size(20.0)
                    .color(primary),
                FlexSpacer::Fixed(10.0),
                label(format!("Position: {}", position))
                    .text_size(18.0),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .boxed())
        },
        ProjectionSwitch { item, from_view, to_view } => {
            OneOf::I(flex((
                label(format!("Item: {}", item))
                    .text_size(24.0)
                    .weight(FontWeight::BOLD),
                FlexSpacer::Fixed(15.0),
                flex_row((
                    sized_box(label(from_view.clone()).text_size(18.0))
                        .padding(15.0)
                        .background(Background::Color(surface))
                        .corner_radius(8.0),
                    label("→").text_size(20.0).color(primary),
                    sized_box(label(to_view.clone()).text_size(18.0))
                        .padding(15.0)
                        .background(Background::Color(primary.with_alpha(0.2)))
                        .border(primary, 2.0)
                        .corner_radius(8.0),
                ))
                .gap(15.0)
                .main_axis_alignment(MainAxisAlignment::Center),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .boxed())
        },
        IsomorphicTransfer { source_domain, target_domain, .. } => {
            OneOf::I(flex((
                label("Transfer learning:")
                    .text_size(18.0),
                FlexSpacer::Fixed(15.0),
                flex_row((
                    sized_box(label(source_domain.clone()).text_size(20.0))
                        .padding(20.0)
                        .background(Background::Color(surface))
                        .corner_radius(10.0),
                    label("≈").text_size(24.0).color(primary),
                    sized_box(label(target_domain.clone()).text_size(20.0))
                        .padding(20.0)
                        .background(Background::Color(primary.with_alpha(0.2)))
                        .corner_radius(10.0),
                ))
                .gap(20.0)
                .main_axis_alignment(MainAxisAlignment::Center),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .boxed())
        },
    }
}
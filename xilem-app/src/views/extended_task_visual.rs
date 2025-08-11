// Extended task visual representations using component system

use abcdeez_core::tasks::extended::ExtendedTaskType;
use crate::components::{Components, AppComponents, ComponentOutput};

pub fn extended_task_visual(
    extended_task: &ExtendedTaskType,
) -> ComponentOutput {
    use ExtendedTaskType::*;
    
    match extended_task {
        BetweenQuery { a, b, c } => {
            // Show three items with middle one highlighted
            let items = vec![a.clone(), b.clone(), c.clone()];
            Components::alphabet_sequence(items, Some(1))
        },
        BoundaryBridging { boundaries, .. } => {
            // Show boundaries as a special sequence
            let items = boundaries.iter().take(4)
                .map(|&b| format!("[{}]", b))
                .collect::<Vec<_>>();
            Components::alphabet_sequence(items, None)
        },
        ReverseNTreadmill { start, n, steps } => {
            // Show reversing pattern
            Components::simple_flex_column(vec![
                Components::simple_label(format!("Start: {} | N: {}", start, n)),
                Components::simple_label(format!("Steps: {}", steps)),
            ])
        },
        DirectionalComparison { a, b, backward } => {
            Components::comparison_visual(a, b, !backward)
        },
        InsertionAdaptation { item, after, before } => {
            // Show insertion point
            let path = vec![item.clone()];
            Components::path_visual(after, before, path)
        },
        MacroDiscovery { sequence, .. } => {
            // Show sequence pattern
            let items = sequence.iter().take(5).cloned().collect();
            Components::alphabet_sequence(items, None)
        },
        SemanticFilter { category, position } => {
            // Show category and position
            Components::simple_flex_column(vec![
                Components::simple_label(format!("Category: {}", category)),
                Components::simple_label(format!("Position: {}", position)),
            ])
        },
        ProjectionSwitch { from_view, to_view, item } => {
            // Show view projection
            Components::path_visual(from_view, to_view, vec![item.clone()])
        },
        IsomorphicTransfer { source_domain, target_domain, .. } => {
            // Show isomorphic mapping
            Components::simple_flex_column(vec![
                Components::simple_label(format!("From: {}", source_domain)),
                Components::simple_label(format!("To: {}", target_domain)),
            ])
        },
        LinearExtensionGeneration { partial_order, .. } => {
            // Show partial order visualization
            Components::simple_flex_column(vec![
                Components::simple_label("Linear Extension".to_string()),
                Components::simple_label(format!("Order: {} items", partial_order.len())),
            ])
        },
        NextStepPrediction { current, goal } => {
            // Show current state and goal
            Components::path_visual(current, goal, vec!["→".to_string()])
        },
        LandmarkNavigation { start, end, .. } => {
            // Show navigation path
            Components::path_visual(start, end, vec!["Nav".to_string()])
        },
    }
}
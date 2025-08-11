// Core task visual representations using component system

use abcdeez_core::tasks::core::TaskType as CoreTaskType;
use crate::components::{Components, AppComponents, ComponentOutput};

pub fn core_task_visual(
    core_task: &CoreTaskType,
) -> ComponentOutput {
    use CoreTaskType::*;
    
    match core_task {
        PairwiseOrder { a, b } => {
            Components::comparison_visual(a, b, false)
        },
        Successor { item } | Predecessor { item } | Index { item } => {
            // Show single item prominently
            Components::alphabet_sequence(vec![item.clone()], Some(0))
        },
        KJump { start, k } => {
            // Show start and direction
            let end = "?";
            let path = if *k > 0 {
                vec![format!("+{}", k)]
            } else {
                vec![format!("{}", k)]
            };
            Components::path_visual(start, end, path)
        },
        Segment { start, count, reverse } => {
            // Show a sequence segment
            let items = vec![
                start.clone(),
                "...".to_string(),
                format!("{} items", count),
            ];
            Components::alphabet_sequence(items, if *reverse { Some(2) } else { Some(0) })
        },
        MissingItem { before, after } => {
            Components::missing_item_visual(before, after)
        },
        ShortestDistance { from, to } => {
            // Show path between two items
            Components::path_visual(from, to, vec!["→".to_string()])
        },
        Comparability { a, b } => {
            Components::comparison_visual(a, b, false)
        },
        TopologicalSort { items } => {
            // Show items to be sorted
            Components::alphabet_sequence(items.clone(), None)
        },
        ShortestPath { from, to } => {
            // Show path between two items
            Components::path_visual(from, to, vec!["...".to_string()])
        },
        MinimalElements | MaximalElements => {
            // Show placeholder for minimal/maximal elements
            Components::alphabet_sequence(vec!["Elements".to_string()], Some(0))
        },
    }
}
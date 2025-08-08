use crate::learner::OperationType;
use crate::tasks::{Task, TaskType};
use crate::topology::Topology;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Extended task types to complete paper specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtendedTaskType {
    // Missing from paper section 4.1
    BetweenQuery { a: String, b: String, c: String },
    BoundaryBridging { start: String, count: usize, boundaries: Vec<usize> },
    
    // Missing from paper section 4.2
    DirectionalComparison { a: String, b: String, backward: bool },
    
    // Missing from paper section 4.3
    InsertionAdaptation { item: String, after: String, before: String },
    
    // Missing from paper section 4.4
    NextStepPrediction { current: String, goal: String },
    LandmarkNavigation { start: String, end: String, landmark: String },
    MacroDiscovery { sequence: Vec<String> },
    
    // Missing from paper section 4.5
    SemanticFilter { category: String, position: usize },
    ProjectionSwitch { item: String, from_view: String, to_view: String },
    IsomorphicTransfer { source_domain: String, target_domain: String, task: Box<Task> },
}

pub struct ExtendedTaskGenerator {
    topology: Topology,
    semantic_attributes: HashMap<String, Vec<String>>,
    macros: HashMap<String, Vec<String>>,
}

impl ExtendedTaskGenerator {
    pub fn new(topology: Topology) -> Self {
        let mut semantic_attributes = HashMap::new();
        
        // For alphabet, add vowel/consonant categories
        if topology.nodes.len() == 26 {
            semantic_attributes.insert(
                "vowel".to_string(),
                vec!["A", "E", "I", "O", "U"].iter().map(|s| s.to_string()).collect()
            );
            
            let consonants: Vec<String> = (b'A'..=b'Z')
                .map(|c| (c as char).to_string())
                .filter(|c| !["A", "E", "I", "O", "U"].contains(&c.as_str()))
                .collect();
            semantic_attributes.insert("consonant".to_string(), consonants);
        }
        
        // Common macros for navigation
        let mut macros = HashMap::new();
        macros.insert(
            "consecutive_forward".to_string(),
            vec!["A", "B", "C"].iter().map(|s| s.to_string()).collect()
        );
        macros.insert(
            "skip_pattern".to_string(),
            vec!["A", "C", "E", "G"].iter().map(|s| s.to_string()).collect()
        );
        
        ExtendedTaskGenerator {
            topology,
            semantic_attributes,
            macros,
        }
    }
    
    pub fn generate_between_query(&self, a: String, b: String, c: String) -> Task {
        let prompt = format!("Is '{}' between '{}' and '{}'?", b, a, c);
        
        let a_node = self.topology.get_node_by_label(&a);
        let b_node = self.topology.get_node_by_label(&b);
        let c_node = self.topology.get_node_by_label(&c);
        
        let correct_answer = if let (Some(an), Some(bn), Some(cn)) = (a_node, b_node, c_node) {
            match self.topology.topology_type {
                crate::topology::TopologyType::Linear => {
                    let between = (an.position < bn.position && bn.position < cn.position) ||
                                 (cn.position < bn.position && bn.position < an.position);
                    if between { "Yes" } else { "No" }.to_string()
                }
                crate::topology::TopologyType::Cyclic => {
                    // For cyclic, check both circular directions
                    let forward = self.check_cyclic_between(&an.id, &bn.id, &cn.id);
                    let backward = self.check_cyclic_between(&cn.id, &bn.id, &an.id);
                    if forward || backward { "Yes" } else { "No" }.to_string()
                }
                _ => "Not applicable".to_string()
            }
        } else {
            "Invalid items".to_string()
        };
        
        Task {
            task_type: TaskType::PairwiseOrder { a: b.clone(), b: c.clone() }, // Simplified mapping
            prompt,
            correct_answer: correct_answer.clone(),
            options: vec!["Yes".to_string(), "No".to_string()],
            difficulty: 0.5,
            operation: OperationType::PairwiseOrder,
        }
    }
    
    fn check_cyclic_between(&self, start: &str, middle: &str, end: &str) -> bool {
        let mut current = start.to_string();
        let mut found_middle = false;
        let max_steps = self.topology.nodes.len();
        
        for _ in 0..max_steps {
            if current == middle {
                found_middle = true;
            }
            if found_middle && current == end {
                return true;
            }
            
            if let Some(next) = self.topology.get_successor(&current) {
                current = next;
            } else {
                break;
            }
        }
        false
    }
    
    pub fn generate_boundary_bridging(&self, start: String, count: usize, boundaries: Vec<usize>) -> Task {
        let prompt = format!(
            "List {} items starting from '{}', crossing chunk boundaries at positions {:?}",
            count, start, boundaries
        );
        
        let segment = self.topology.get_segment(&start, count, false);
        let correct_answer = segment.join(", ");
        
        // Calculate difficulty based on boundary crossings
        let start_node = self.topology.get_node_by_label(&start);
        let mut boundary_crossings = 0;
        if let Some(node) = start_node {
            let start_idx = self.topology.node_map[&node.id];
            for boundary in &boundaries {
                if *boundary >= start_idx && *boundary < start_idx + count {
                    boundary_crossings += 1;
                }
            }
        }
        
        let difficulty = 0.3 + (0.2 * boundary_crossings as f64).min(0.7);
        
        Task {
            task_type: TaskType::Segment { start, count, reverse: false },
            prompt,
            correct_answer,
            options: vec![],
            difficulty,
            operation: OperationType::Segment(count, false),
        }
    }
    
    pub fn generate_directional_comparison(&self, a: String, b: String, backward: bool) -> Task {
        let direction = if backward { "backward" } else { "forward" };
        let prompt = format!("Moving {}, does '{}' come before '{}'?", direction, a, b);
        
        let correct_answer = match self.topology.topology_type {
            crate::topology::TopologyType::Cyclic => {
                // In cyclic, direction affects the path
                let path = if backward {
                    self.find_backward_path(&a, &b)
                } else {
                    self.find_forward_path(&a, &b)
                };
                if path { "Yes" } else { "No" }.to_string()
            }
            _ => {
                // Linear doesn't change with direction
                if let Some(before) = self.topology.is_before(&a, &b) {
                    if before { "Yes" } else { "No" }.to_string()
                } else {
                    "Not applicable".to_string()
                }
            }
        };
        
        Task {
            task_type: TaskType::PairwiseOrder { a, b },
            prompt,
            correct_answer: correct_answer.clone(),
            options: vec!["Yes".to_string(), "No".to_string()],
            difficulty: 0.5,
            operation: OperationType::PairwiseOrder,
        }
    }
    
    fn find_forward_path(&self, from: &str, to: &str) -> bool {
        let mut current = from.to_string();
        for _ in 0..self.topology.nodes.len() {
            if current == to {
                return true;
            }
            if let Some(next) = self.topology.get_successor(&current) {
                if let Some(node) = self.topology.get_node_by_id(&next) {
                    current = node.label.clone();
                }
            } else {
                break;
            }
        }
        false
    }
    
    fn find_backward_path(&self, from: &str, to: &str) -> bool {
        let mut current = from.to_string();
        for _ in 0..self.topology.nodes.len() {
            if current == to {
                return true;
            }
            if let Some(prev) = self.topology.get_predecessor(&current) {
                if let Some(node) = self.topology.get_node_by_id(&prev) {
                    current = node.label.clone();
                }
            } else {
                break;
            }
        }
        false
    }
    
    pub fn generate_insertion_adaptation(&self, item: String, after: String, before: String) -> Task {
        let prompt = format!(
            "If '{}' must come after '{}' but before '{}', where in the sequence should it be inserted?",
            item, after, before
        );
        
        let after_node = self.topology.get_node_by_label(&after);
        let before_node = self.topology.get_node_by_label(&before);
        
        let correct_answer = if let (Some(an), Some(bn)) = (after_node, before_node) {
            let after_idx = self.topology.node_map[&an.id];
            let before_idx = self.topology.node_map[&bn.id];
            
            if before_idx > after_idx + 1 {
                // Find intermediate position
                let insert_idx = (after_idx + before_idx) / 2;
                if insert_idx < self.topology.nodes.len() {
                    format!("Between {} and {}", 
                        self.topology.nodes[insert_idx].label,
                        self.topology.nodes[insert_idx + 1].label)
                } else {
                    "At the specified position".to_string()
                }
            } else if before_idx == after_idx + 1 {
                "Directly between them".to_string()
            } else {
                "No valid position (constraints conflict)".to_string()
            }
        } else {
            "Invalid constraints".to_string()
        };
        
        Task {
            task_type: TaskType::MissingItem { before: after, after: before },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.8,
            operation: OperationType::PairwiseOrder,
        }
    }
    
    pub fn generate_next_step_prediction(&self, current: String, goal: String) -> Task {
        let prompt = format!("You are at '{}'. To reach '{}', what should be your next step?", current, goal);
        
        let path = self.topology.shortest_path(&current, &goal);
        let correct_answer = if let Some(p) = path {
            if p.len() > 1 {
                p[1].clone()
            } else {
                "Already at goal".to_string()
            }
        } else {
            "No path available".to_string()
        };
        
        // Generate plausible alternatives
        let mut options = vec![correct_answer.clone()];
        if let Some(node) = self.topology.get_node_by_label(&current) {
            // Add neighbors
            if let Some(succ) = self.topology.get_successor(&node.id) {
                if let Some(n) = self.topology.get_node_by_id(&succ) {
                    if !options.contains(&n.label) {
                        options.push(n.label.clone());
                    }
                }
            }
            if let Some(pred) = self.topology.get_predecessor(&node.id) {
                if let Some(n) = self.topology.get_node_by_id(&pred) {
                    if !options.contains(&n.label) {
                        options.push(n.label.clone());
                    }
                }
            }
        }
        
        // Add some random options
        for node in self.topology.nodes.iter().take(5) {
            if !options.contains(&node.label) && node.label != current {
                options.push(node.label.clone());
                if options.len() >= 4 {
                    break;
                }
            }
        }
        
        options.shuffle(&mut rand::thread_rng());
        
        Task {
            task_type: TaskType::ShortestPath { from: current, to: goal },
            prompt,
            correct_answer,
            options,
            difficulty: 0.6,
            operation: OperationType::PairwiseOrder,
        }
    }
    
    pub fn generate_landmark_navigation(&self, start: String, end: String, landmark: String) -> Task {
        let prompt = format!(
            "To get from '{}' to '{}', is it efficient to go via '{}'?",
            start, end, landmark
        );
        
        let direct_path = self.topology.shortest_path(&start, &end);
        let via_landmark = if let (Some(p1), Some(p2)) = (
            self.topology.shortest_path(&start, &landmark),
            self.topology.shortest_path(&landmark, &end)
        ) {
            Some(p1.len() + p2.len() - 1) // -1 because landmark counted twice
        } else {
            None
        };
        
        let correct_answer = match (direct_path, via_landmark) {
            (Some(direct), Some(via)) => {
                if via <= direct.len() + 1 { // Allow small detour
                    "Yes"
                } else {
                    "No"
                }.to_string()
            }
            _ => "Cannot determine".to_string()
        };
        
        Task {
            task_type: TaskType::ShortestPath { from: start, to: end },
            prompt,
            correct_answer: correct_answer.clone(),
            options: vec!["Yes".to_string(), "No".to_string(), "Cannot determine".to_string()],
            difficulty: 0.7,
            operation: OperationType::PairwiseOrder,
        }
    }
    
    pub fn generate_macro_discovery(&self, sequence: Vec<String>) -> Task {
        let prompt = format!("What pattern or macro does this sequence represent: {:?}?", sequence);
        
        let pattern = self.identify_pattern(&sequence);
        
        Task {
            task_type: TaskType::Segment { 
                start: sequence.first().unwrap_or(&"".to_string()).clone(),
                count: sequence.len(),
                reverse: false
            },
            prompt,
            correct_answer: pattern.clone(),
            options: vec![
                "Consecutive forward".to_string(),
                "Consecutive backward".to_string(),
                "Skip pattern".to_string(),
                "Chunk boundary crossing".to_string(),
                "Random sequence".to_string(),
            ],
            difficulty: 0.6,
            operation: OperationType::Segment(sequence.len(), false),
        }
    }
    
    fn identify_pattern(&self, sequence: &[String]) -> String {
        if sequence.len() < 2 {
            return "Too short to identify".to_string();
        }
        
        // Check consecutive forward
        let mut consecutive_forward = true;
        for i in 0..sequence.len() - 1 {
            if let (Some(curr), Some(next)) = (
                self.topology.get_node_by_label(&sequence[i]),
                self.topology.get_node_by_label(&sequence[i + 1])
            ) {
                if self.topology.get_successor(&curr.id) != Some(next.id.clone()) {
                    consecutive_forward = false;
                    break;
                }
            }
        }
        if consecutive_forward {
            return "Consecutive forward".to_string();
        }
        
        // Check consecutive backward
        let mut consecutive_backward = true;
        for i in 0..sequence.len() - 1 {
            if let (Some(curr), Some(next)) = (
                self.topology.get_node_by_label(&sequence[i]),
                self.topology.get_node_by_label(&sequence[i + 1])
            ) {
                if self.topology.get_predecessor(&curr.id) != Some(next.id.clone()) {
                    consecutive_backward = false;
                    break;
                }
            }
        }
        if consecutive_backward {
            return "Consecutive backward".to_string();
        }
        
        // Check skip pattern
        let mut skip_distances = Vec::new();
        for i in 0..sequence.len() - 1 {
            if let Some(dist) = self.topology.get_distance(
                &self.topology.get_node_by_label(&sequence[i]).unwrap().id,
                &self.topology.get_node_by_label(&sequence[i + 1]).unwrap().id
            ) {
                skip_distances.push(dist);
            }
        }
        
        if !skip_distances.is_empty() && skip_distances.iter().all(|&d| d == skip_distances[0] && d > 1) {
            return "Skip pattern".to_string();
        }
        
        // Check for chunk boundary crossing (simplified)
        if sequence.len() > 5 {
            return "Chunk boundary crossing".to_string();
        }
        
        "Random sequence".to_string()
    }
    
    pub fn generate_semantic_filter(&self, category: String, position: usize) -> Task {
        let prompt = format!("What is the {}th item in the '{}' category?", position, category);
        
        let correct_answer = if let Some(items) = self.semantic_attributes.get(&category) {
            if position > 0 && position <= items.len() {
                items[position - 1].clone()
            } else {
                "Position out of range".to_string()
            }
        } else {
            "Unknown category".to_string()
        };
        
        let mut options = vec![correct_answer.clone()];
        
        // Add some other items from the category
        if let Some(items) = self.semantic_attributes.get(&category) {
            for item in items.iter().take(5) {
                if !options.contains(item) {
                    options.push(item.clone());
                    if options.len() >= 4 {
                        break;
                    }
                }
            }
        }
        
        options.shuffle(&mut rand::thread_rng());
        
        Task {
            task_type: TaskType::Index { item: correct_answer.clone() },
            prompt,
            correct_answer,
            options,
            difficulty: 0.6,
            operation: OperationType::Index,
        }
    }
    
    pub fn generate_projection_switch(&self, item: String, from_view: String, to_view: String) -> Task {
        let prompt = format!(
            "If '{}' is at position X in '{}' view, what position in '{}' view?",
            item, from_view, to_view
        );
        
        let correct_answer = match (from_view.as_str(), to_view.as_str()) {
            ("alphabetical", "reverse") => {
                if let Some(node) = self.topology.get_node_by_label(&item) {
                    let pos = self.topology.node_map[&node.id];
                    let reverse_pos = self.topology.nodes.len() - pos;
                    reverse_pos.to_string()
                } else {
                    "Unknown".to_string()
                }
            }
            ("alphabetical", "vowels_only") => {
                if let Some(vowels) = self.semantic_attributes.get("vowel") {
                    if let Some(idx) = vowels.iter().position(|v| v == &item) {
                        (idx + 1).to_string()
                    } else {
                        "Not in this view".to_string()
                    }
                } else {
                    "Unknown".to_string()
                }
            }
            _ => "Same position".to_string()
        };
        
        Task {
            task_type: TaskType::Index { item },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.5,
            operation: OperationType::Index,
        }
    }
    
    pub fn generate_isomorphic_transfer(&self, source_domain: String, target_domain: String) -> Task {
        let prompt = format!(
            "You learned pattern X in '{}'. Apply the same pattern in '{}'.",
            source_domain, target_domain
        );
        
        // Simplified: just test if they can do successor in a new domain
        let correct_answer = "Transfer successful".to_string();
        
        Task {
            task_type: TaskType::Successor { item: "A".to_string() },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.7,
            operation: OperationType::Successor,
        }
    }
}

/// Dynamic graph adaptation support
pub struct DynamicTopology {
    base_topology: Topology,
    modifications: Vec<GraphModification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphModification {
    AddNode { id: String, label: String, position: f64 },
    RemoveNode { id: String },
    AddEdge { from: String, to: String, weight: f64 },
    RemoveEdge { from: String, to: String },
    UpdateNodePosition { id: String, new_position: f64 },
}

impl DynamicTopology {
    pub fn new(base: Topology) -> Self {
        DynamicTopology {
            base_topology: base,
            modifications: Vec::new(),
        }
    }
    
    pub fn apply_modification(&mut self, modification: GraphModification) {
        self.modifications.push(modification.clone());
        
        match modification {
            GraphModification::AddNode { id, label, position } => {
                self.base_topology.nodes.push(crate::topology::Node {
                    id: id.clone(),
                    label,
                    position,
                });
                self.base_topology.node_map.insert(id, self.base_topology.nodes.len() - 1);
            }
            GraphModification::RemoveNode { id } => {
                self.base_topology.nodes.retain(|n| n.id != id);
                self.base_topology.edges.retain(|e| e.from != id && e.to != id);
                self.rebuild_node_map();
            }
            GraphModification::AddEdge { from, to, weight } => {
                self.base_topology.edges.push(crate::topology::Edge {
                    from,
                    to,
                    weight,
                });
            }
            GraphModification::RemoveEdge { from, to } => {
                self.base_topology.edges.retain(|e| !(e.from == from && e.to == to));
            }
            GraphModification::UpdateNodePosition { id, new_position } => {
                if let Some(node) = self.base_topology.nodes.iter_mut().find(|n| n.id == id) {
                    node.position = new_position;
                }
            }
        }
    }
    
    fn rebuild_node_map(&mut self) {
        self.base_topology.node_map.clear();
        for (i, node) in self.base_topology.nodes.iter().enumerate() {
            self.base_topology.node_map.insert(node.id.clone(), i);
        }
    }
    
    pub fn get_topology(&self) -> &Topology {
        &self.base_topology
    }
    
    pub fn rollback(&mut self, steps: usize) {
        for _ in 0..steps.min(self.modifications.len()) {
            self.modifications.pop();
        }
        // Rebuild topology from scratch with remaining modifications
        self.rebuild_from_modifications();
    }
    
    fn rebuild_from_modifications(&mut self) {
        // This would recreate the topology from base + modifications
        // Simplified for now
    }
}

/// Transfer learning framework
pub struct TransferLearning {
    source_domain: Topology,
    target_domain: Topology,
    mapping: HashMap<String, String>,
}

impl TransferLearning {
    pub fn new(source: Topology, target: Topology) -> Self {
        let mut mapping = HashMap::new();
        
        // Create isomorphic mapping if possible
        if source.nodes.len() == target.nodes.len() {
            for (s, t) in source.nodes.iter().zip(target.nodes.iter()) {
                mapping.insert(s.label.clone(), t.label.clone());
            }
        }
        
        TransferLearning {
            source_domain: source,
            target_domain: target,
            mapping,
        }
    }
    
    pub fn transfer_task(&self, source_task: &Task) -> Option<Task> {
        // Map task from source to target domain
        match &source_task.task_type {
            TaskType::Successor { item } => {
                if let Some(target_item) = self.mapping.get(item) {
                    Some(Task {
                        task_type: TaskType::Successor { item: target_item.clone() },
                        prompt: source_task.prompt.replace(item, target_item),
                        correct_answer: self.mapping.get(&source_task.correct_answer)
                            .unwrap_or(&source_task.correct_answer)
                            .clone(),
                        options: source_task.options.iter()
                            .map(|o| self.mapping.get(o).unwrap_or(o).clone())
                            .collect(),
                        difficulty: source_task.difficulty,
                        operation: source_task.operation.clone(),
                    })
                } else {
                    None
                }
            }
            _ => None, // Simplified for now
        }
    }
    
    pub fn measure_transfer_efficiency(&self, source_performance: f64, target_performance: f64) -> f64 {
        // Calculate transfer efficiency metric
        if source_performance > 0.0 {
            target_performance / source_performance
        } else {
            0.0
        }
    }
}
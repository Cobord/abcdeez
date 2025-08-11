# Directed Acyclic Graphs (DAGs) and Partial Orders

Directed Acyclic Graphs (DAGs) represent hierarchical knowledge structures where elements have prerequisite relationships. In ABCDeez Core, DAGs model learning domains where some concepts must be mastered before others, such as academic curricula, skill trees, or knowledge dependencies.

## Conceptual Foundation

### What Makes a Structure a DAG?

A Directed Acyclic Graph has these essential properties:
1. **Directed edges**: Relationships have direction (A → B means A is prerequisite for B)
2. **No cycles**: Cannot return to a node by following directed edges
3. **Partial ordering**: Some elements are comparable (ordered), others are not
4. **Transitivity**: If A → B and B → C, then A → C (implicitly)
5. **Multiple paths**: There may be several valid learning sequences

### Cognitive Significance

DAGs reflect real-world learning hierarchies:
- **Academic prerequisites**: Algebra before Calculus
- **Skill dependencies**: Basic reading before advanced comprehension
- **Procedural steps**: Some tasks must precede others
- **Conceptual hierarchies**: Abstract concepts built on concrete ones

## Implementation in ABCDeez Core

### Creating DAG Topologies

```rust
use abcdeez_core::core::topology::{Topology, TopologyType};

// Mathematics curriculum DAG
let mut math_curriculum = Topology::new_dag();

// Add concepts
math_curriculum.add_node("Counting", "Basic counting skills");
math_curriculum.add_node("Addition", "Addition operations");
math_curriculum.add_node("Subtraction", "Subtraction operations");
math_curriculum.add_node("Multiplication", "Multiplication tables");
math_curriculum.add_node("Division", "Division operations");
math_curriculum.add_node("Fractions", "Fractional numbers");
math_curriculum.add_node("Algebra", "Algebraic thinking");

// Add prerequisite relationships
math_curriculum.add_prerequisite("Counting", "Addition");
math_curriculum.add_prerequisite("Addition", "Subtraction");
math_curriculum.add_prerequisite("Addition", "Multiplication");
math_curriculum.add_prerequisite("Subtraction", "Division");
math_curriculum.add_prerequisite("Multiplication", "Fractions");
math_curriculum.add_prerequisite("Division", "Fractions");
math_curriculum.add_prerequisite("Fractions", "Algebra");
```

### Internal Representation

```rust
impl Topology {
    pub fn new_dag() -> Self {
        Self {
            topology_type: TopologyType::DAG,
            nodes: Vec::new(),
            edges: Vec::new(),
            node_map: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn add_prerequisite(&mut self, prerequisite: &str, dependent: &str) -> Result<(), String> {
        let prereq_id = self.get_or_create_node(prerequisite)?;
        let dependent_id = self.get_or_create_node(dependent)?;
        
        // Check if adding this edge would create a cycle
        if self.would_create_cycle(&prereq_id, &dependent_id) {
            return Err("Adding this prerequisite would create a cycle".to_string());
        }
        
        self.edges.push(Edge {
            from: prereq_id.clone(),
            to: dependent_id.clone(),
            weight: 1.0,
            edge_type: EdgeType::Prerequisite,
        });
        
        Ok(())
    }
}
```

## Operations on DAGs

### Topological Ordering

```rust
impl Topology {
    pub fn get_topological_sort(&self) -> Result<Vec<String>, String> {
        if self.topology_type != TopologyType::DAG {
            return Err("Topological sort only applies to DAGs".to_string());
        }
        
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        
        // Initialize in-degrees
        for node in &self.nodes {
            in_degree.insert(node.id.clone(), 0);
            adjacency.insert(node.id.clone(), Vec::new());
        }
        
        // Calculate in-degrees and adjacency
        for edge in &self.edges {
            *in_degree.get_mut(&edge.to).unwrap() += 1;
            adjacency.get_mut(&edge.from).unwrap().push(edge.to.clone());
        }
        
        // Kahn's algorithm
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        
        // Find nodes with no prerequisites
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }
        
        while let Some(node_id) = queue.pop_front() {
            result.push(self.get_node_by_id(&node_id)?.label.clone());
            
            // Update neighbors
            if let Some(neighbors) = adjacency.get(&node_id) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        
        if result.len() != self.nodes.len() {
            Err("Graph contains a cycle".to_string())
        } else {
            Ok(result)
        }
    }
}
```

### Finding Prerequisites and Dependencies

```rust
impl Topology {
    // Get all direct prerequisites
    pub fn get_prerequisites(&self, concept: &str) -> Vec<String> {
        if let Some(node_id) = self.get_node_id_by_label(concept) {
            self.edges
                .iter()
                .filter(|e| e.to == node_id)
                .map(|e| self.get_node_by_id(&e.from).unwrap().label.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    // Get all direct dependents
    pub fn get_dependents(&self, concept: &str) -> Vec<String> {
        if let Some(node_id) = self.get_node_id_by_label(concept) {
            self.edges
                .iter()
                .filter(|e| e.from == node_id)
                .map(|e| self.get_node_by_id(&e.to).unwrap().label.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    // Get all transitive prerequisites (recursive)
    pub fn get_all_prerequisites(&self, concept: &str) -> Vec<String> {
        let mut prerequisites = HashSet::new();
        let mut to_visit = VecDeque::new();
        
        if let Some(node_id) = self.get_node_id_by_label(concept) {
            to_visit.push_back(node_id.clone());
            
            while let Some(current_id) = to_visit.pop_front() {
                for edge in &self.edges {
                    if edge.to == current_id {
                        let prereq_label = self.get_node_by_id(&edge.from).unwrap().label.clone();
                        if prerequisites.insert(prereq_label.clone()) {
                            to_visit.push_back(edge.from.clone());
                        }
                    }
                }
            }
        }
        
        prerequisites.into_iter().collect()
    }
}
```

### Learning Path Generation

```rust
pub struct LearningPath {
    pub concepts: Vec<String>,
    pub estimated_difficulty: f64,
    pub total_prerequisites: usize,
}

impl Topology {
    pub fn generate_learning_paths(&self, target: &str) -> Vec<LearningPath> {
        let mut paths = Vec::new();
        let all_prereqs = self.get_all_prerequisites(target);
        
        // Generate different valid orderings
        if let Ok(topo_sort) = self.get_topological_sort() {
            // Filter to only include prerequisites and target
            let mut relevant_concepts: Vec<String> = topo_sort
                .into_iter()
                .filter(|c| all_prereqs.contains(c) || c == target)
                .collect();
            
            // Generate alternative paths by varying non-dependent orderings
            paths.push(LearningPath {
                concepts: relevant_concepts.clone(),
                estimated_difficulty: self.calculate_path_difficulty(&relevant_concepts),
                total_prerequisites: all_prereqs.len(),
            });
            
            // Generate variations for concepts that can be learned in parallel
            let parallel_groups = self.identify_parallel_concepts(&relevant_concepts);
            for variation in self.generate_path_variations(relevant_concepts, parallel_groups) {
                paths.push(LearningPath {
                    concepts: variation.clone(),
                    estimated_difficulty: self.calculate_path_difficulty(&variation),
                    total_prerequisites: all_prereqs.len(),
                });
            }
        }
        
        paths
    }
    
    fn calculate_path_difficulty(&self, path: &[String]) -> f64 {
        path.iter()
            .enumerate()
            .map(|(i, concept)| {
                let base_difficulty = self.get_concept_difficulty(concept);
                let position_penalty = (i as f64) * 0.05; // Later concepts harder
                base_difficulty + position_penalty
            })
            .sum::<f64>() / path.len() as f64
    }
}
```

## Cognitive Phenomena in DAGs

### Prerequisite Mastery Effects

```rust
pub struct PrerequisiteMastery {
    pub concept: String,
    pub mastery_level: f64,        // [0, 1]
    pub prerequisites_satisfied: bool,
}

impl PrerequisiteMastery {
    pub fn can_learn_effectively(&self) -> bool {
        // Need high mastery of prerequisites to learn dependent concepts
        self.prerequisites_satisfied && self.mastery_level > 0.7
    }
    
    pub fn predict_learning_difficulty(&self, target_concept: &str, dag: &Topology) -> f64 {
        let prerequisites = dag.get_all_prerequisites(target_concept);
        
        if prerequisites.is_empty() {
            return 0.3; // Base difficulty for fundamental concepts
        }
        
        // Calculate readiness based on prerequisite mastery
        let prereq_readiness: f64 = prerequisites.iter()
            .map(|prereq| {
                // Simulate getting mastery level for prerequisite
                self.get_mastery_level(prereq)
            })
            .sum::<f64>() / prerequisites.len() as f64;
        
        // Higher prerequisite mastery reduces learning difficulty
        (1.0 - prereq_readiness * 0.7).max(0.1)
    }
    
    fn get_mastery_level(&self, concept: &str) -> f64 {
        // Simplified - in practice this would query the learner model
        if concept == &self.concept {
            self.mastery_level
        } else {
            0.5 // Default mastery assumption
        }
    }
}
```

### Knowledge Transfer Effects

```rust
pub struct KnowledgeTransfer {
    pub source_concept: String,
    pub target_concept: String,
    pub transfer_strength: f64,    // [0, 1] how much knowledge transfers
}

impl Topology {
    pub fn calculate_transfer_effects(&self, mastered_concepts: &[String]) -> HashMap<String, f64> {
        let mut transfer_benefits = HashMap::new();
        
        for concept in &self.get_all_concepts() {
            let mut benefit = 0.0;
            
            // Direct prerequisites provide strong transfer
            for prereq in self.get_prerequisites(concept) {
                if mastered_concepts.contains(&prereq) {
                    benefit += 0.8;
                }
            }
            
            // Related concepts provide weaker transfer
            for mastered in mastered_concepts {
                if let Some(relationship_strength) = self.get_semantic_similarity(mastered, concept) {
                    benefit += relationship_strength * 0.3;
                }
            }
            
            transfer_benefits.insert(concept.clone(), benefit.min(1.0));
        }
        
        transfer_benefits
    }
    
    fn get_semantic_similarity(&self, concept1: &str, concept2: &str) -> Option<f64> {
        // Calculate semantic similarity based on shared prerequisites,
        // concept embeddings, or domain knowledge
        let prereqs1: HashSet<_> = self.get_all_prerequisites(concept1).into_iter().collect();
        let prereqs2: HashSet<_> = self.get_all_prerequisites(concept2).into_iter().collect();
        
        let intersection = prereqs1.intersection(&prereqs2).count();
        let union = prereqs1.union(&prereqs2).count();
        
        if union > 0 {
            Some(intersection as f64 / union as f64)
        } else {
            None
        }
    }
}
```

## Learning Strategies for DAGs

### Bottom-Up vs Top-Down Learning

```rust
pub enum LearningStrategy {
    BottomUp,      // Start with fundamentals
    TopDown,       // Start with goals, work backward
    MiddleOut,     // Start with moderate concepts
    Adaptive,      // Switch based on performance
}

impl Topology {
    pub fn generate_strategy_sequence(&self, target: &str, strategy: LearningStrategy) -> Vec<String> {
        match strategy {
            LearningStrategy::BottomUp => {
                self.bottom_up_sequence(target)
            }
            LearningStrategy::TopDown => {
                self.top_down_sequence(target)
            }
            LearningStrategy::MiddleOut => {
                self.middle_out_sequence(target)
            }
            LearningStrategy::Adaptive => {
                // Start bottom-up but allow strategy switching
                self.adaptive_sequence(target)
            }
        }
    }
    
    fn bottom_up_sequence(&self, target: &str) -> Vec<String> {
        // Standard topological ordering (prerequisites first)
        if let Ok(topo_sort) = self.get_topological_sort() {
            let all_prereqs: HashSet<_> = self.get_all_prerequisites(target).into_iter().collect();
            topo_sort.into_iter()
                .filter(|c| all_prereqs.contains(c) || c == target)
                .collect()
        } else {
            vec![target.to_string()]
        }
    }
    
    fn top_down_sequence(&self, target: &str) -> Vec<String> {
        // Start with target, then work backward through prerequisites
        let mut sequence = vec![target.to_string()];
        let mut remaining: VecDeque<String> = self.get_prerequisites(target).into();
        
        while let Some(concept) = remaining.pop_front() {
            if !sequence.contains(&concept) {
                sequence.insert(sequence.len() - 1, concept.clone());
                for prereq in self.get_prerequisites(&concept) {
                    if !sequence.contains(&prereq) {
                        remaining.push_back(prereq);
                    }
                }
            }
        }
        
        sequence
    }
    
    fn middle_out_sequence(&self, target: &str) -> Vec<String> {
        // Start with concepts of moderate complexity
        let all_concepts = self.get_learning_path_concepts(target);
        let complexity_scores: HashMap<String, usize> = all_concepts
            .iter()
            .map(|c| (c.clone(), self.get_all_prerequisites(c).len()))
            .collect();
        
        let mut sorted_by_complexity: Vec<_> = all_concepts.iter()
            .map(|c| (c, complexity_scores[c]))
            .collect();
        sorted_by_complexity.sort_by_key(|(_, complexity)| *complexity);
        
        // Start from middle complexity
        let mid_index = sorted_by_complexity.len() / 2;
        let start_concept = sorted_by_complexity[mid_index].0.clone();
        
        // Build sequence ensuring prerequisites are met
        self.build_valid_sequence_from(start_concept, all_concepts)
    }
}
```

### Parallel Learning Opportunities

```rust
pub struct ParallelLearningGroup {
    pub concepts: Vec<String>,
    pub can_learn_simultaneously: bool,
    pub estimated_time_savings: f64,
}

impl Topology {
    pub fn identify_parallel_learning_opportunities(&self) -> Vec<ParallelLearningGroup> {
        let mut groups = Vec::new();
        let concepts = self.get_all_concepts();
        
        for window in concepts.windows(2) {
            let concept1 = &window[0];
            let concept2 = &window[1];
            
            // Check if concepts are independent (no prerequisite relationship)
            if self.are_independent_concepts(concept1, concept2) {
                let can_parallel = self.can_learn_in_parallel(concept1, concept2);
                let time_savings = if can_parallel { 0.3 } else { 0.0 };
                
                groups.push(ParallelLearningGroup {
                    concepts: vec![concept1.clone(), concept2.clone()],
                    can_learn_simultaneously: can_parallel,
                    estimated_time_savings: time_savings,
                });
            }
        }
        
        groups
    }
    
    fn are_independent_concepts(&self, concept1: &str, concept2: &str) -> bool {
        let prereqs1: HashSet<_> = self.get_all_prerequisites(concept1).into_iter().collect();
        let prereqs2: HashSet<_> = self.get_all_prerequisites(concept2).into_iter().collect();
        
        // Independent if neither is a prerequisite of the other
        !prereqs1.contains(concept2) && !prereqs2.contains(concept1)
    }
    
    fn can_learn_in_parallel(&self, concept1: &str, concept2: &str) -> bool {
        // Check cognitive load and similarity factors
        let complexity1 = self.get_concept_complexity(concept1);
        let complexity2 = self.get_concept_complexity(concept2);
        let similarity = self.get_semantic_similarity(concept1, concept2).unwrap_or(0.0);
        
        // Can learn in parallel if:
        // 1. Combined complexity not too high
        // 2. Concepts not too similar (to avoid confusion)
        (complexity1 + complexity2) < 1.5 && similarity < 0.7
    }
}
```

## Common DAG Patterns

### Academic Curriculum

```rust
impl Topology {
    pub fn create_math_curriculum() -> Self {
        let mut curriculum = Self::new_dag();
        
        // Elementary concepts
        curriculum.add_concept("Number Recognition", 0.2);
        curriculum.add_concept("Counting", 0.3);
        curriculum.add_concept("Addition", 0.4);
        curriculum.add_concept("Subtraction", 0.4);
        
        // Intermediate concepts
        curriculum.add_concept("Multiplication", 0.6);
        curriculum.add_concept("Division", 0.6);
        curriculum.add_concept("Fractions", 0.7);
        curriculum.add_concept("Decimals", 0.7);
        
        // Advanced concepts
        curriculum.add_concept("Algebra", 0.8);
        curriculum.add_concept("Geometry", 0.8);
        curriculum.add_concept("Calculus", 0.9);
        
        // Add prerequisite relationships
        curriculum.add_prerequisite("Number Recognition", "Counting");
        curriculum.add_prerequisite("Counting", "Addition");
        curriculum.add_prerequisite("Addition", "Subtraction");
        curriculum.add_prerequisite("Addition", "Multiplication");
        curriculum.add_prerequisite("Subtraction", "Division");
        curriculum.add_prerequisite("Multiplication", "Fractions");
        curriculum.add_prerequisite("Division", "Fractions");
        curriculum.add_prerequisite("Fractions", "Decimals");
        curriculum.add_prerequisite("Decimals", "Algebra");
        curriculum.add_prerequisite("Algebra", "Calculus");
        
        // Geometry has its own path
        curriculum.add_prerequisite("Fractions", "Geometry");
        
        curriculum
    }
}
```

### Skill Trees

```rust
pub struct SkillTree {
    topology: Topology,
    skill_levels: HashMap<String, u32>,
    unlock_requirements: HashMap<String, Vec<UnlockRequirement>>,
}

#[derive(Clone)]
pub struct UnlockRequirement {
    pub prerequisite_skill: String,
    pub minimum_level: u32,
    pub proficiency_threshold: f64,
}

impl SkillTree {
    pub fn create_programming_skills() -> Self {
        let mut tree = Self {
            topology: Topology::new_dag(),
            skill_levels: HashMap::new(),
            unlock_requirements: HashMap::new(),
        };
        
        // Basic programming skills
        tree.add_skill("Syntax Understanding", 1, 0.3);
        tree.add_skill("Variables", 2, 0.4);
        tree.add_skill("Control Structures", 3, 0.5);
        tree.add_skill("Functions", 4, 0.6);
        tree.add_skill("Data Structures", 5, 0.7);
        tree.add_skill("Algorithms", 6, 0.8);
        tree.add_skill("Object-Oriented Programming", 7, 0.8);
        tree.add_skill("Design Patterns", 8, 0.9);
        
        // Set up unlock requirements
        tree.require_skill("Variables", "Syntax Understanding", 1, 0.7);
        tree.require_skill("Control Structures", "Variables", 2, 0.7);
        tree.require_skill("Functions", "Control Structures", 3, 0.7);
        tree.require_skill("Data Structures", "Functions", 4, 0.8);
        tree.require_skill("Algorithms", "Data Structures", 5, 0.8);
        tree.require_skill("Object-Oriented Programming", "Functions", 4, 0.8);
        tree.require_skill("Design Patterns", "Object-Oriented Programming", 7, 0.9);
        
        tree
    }
    
    fn add_skill(&mut self, name: &str, level: u32, difficulty: f64) {
        self.topology.add_concept(name, difficulty);
        self.skill_levels.insert(name.to_string(), level);
    }
    
    fn require_skill(&mut self, skill: &str, prerequisite: &str, min_level: u32, threshold: f64) {
        self.topology.add_prerequisite(prerequisite, skill).unwrap();
        self.unlock_requirements
            .entry(skill.to_string())
            .or_default()
            .push(UnlockRequirement {
                prerequisite_skill: prerequisite.to_string(),
                minimum_level: min_level,
                proficiency_threshold: threshold,
            });
    }
    
    pub fn get_unlockable_skills(&self, learner: &LearnerModel) -> Vec<String> {
        let mut unlockable = Vec::new();
        
        for (skill, requirements) in &self.unlock_requirements {
            let can_unlock = requirements.iter().all(|req| {
                learner.get_skill_level(&req.prerequisite_skill) >= req.minimum_level &&
                learner.get_proficiency(&req.prerequisite_skill) >= req.proficiency_threshold
            });
            
            if can_unlock {
                unlockable.push(skill.clone());
            }
        }
        
        unlockable
    }
}
```

## Optimization and Algorithms

### Efficient Reachability Queries

```rust
pub struct ReachabilityMatrix {
    matrix: Vec<Vec<bool>>,
    node_indices: HashMap<String, usize>,
}

impl ReachabilityMatrix {
    pub fn from_dag(dag: &Topology) -> Self {
        let nodes: Vec<_> = dag.nodes.iter().map(|n| n.id.clone()).collect();
        let n = nodes.len();
        
        let mut node_indices = HashMap::new();
        for (i, node_id) in nodes.iter().enumerate() {
            node_indices.insert(node_id.clone(), i);
        }
        
        let mut matrix = vec![vec![false; n]; n];
        
        // Initialize direct edges
        for edge in &dag.edges {
            let from_idx = node_indices[&edge.from];
            let to_idx = node_indices[&edge.to];
            matrix[from_idx][to_idx] = true;
        }
        
        // Floyd-Warshall for transitive closure
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    matrix[i][j] = matrix[i][j] || (matrix[i][k] && matrix[k][j]);
                }
            }
        }
        
        Self { matrix, node_indices }
    }
    
    pub fn is_reachable(&self, from: &str, to: &str) -> bool {
        if let (Some(&from_idx), Some(&to_idx)) = 
            (self.node_indices.get(from), self.node_indices.get(to)) {
            self.matrix[from_idx][to_idx]
        } else {
            false
        }
    }
}
```

### Minimum Spanning Prerequisites

```rust
impl Topology {
    pub fn find_minimum_prerequisites(&self, target_concepts: &[String]) -> Vec<String> {
        let mut required = HashSet::new();
        
        // Add all direct and indirect prerequisites
        for concept in target_concepts {
            for prereq in self.get_all_prerequisites(concept) {
                required.insert(prereq);
            }
        }
        
        // Remove redundant prerequisites (those implied by others)
        let mut minimal = required.clone();
        for prereq1 in &required {
            for prereq2 in &required {
                if prereq1 != prereq2 && self.is_prerequisite_of(prereq1, prereq2) {
                    minimal.remove(prereq1);
                }
            }
        }
        
        minimal.into_iter().collect()
    }
    
    fn is_prerequisite_of(&self, potential_prereq: &str, concept: &str) -> bool {
        self.get_all_prerequisites(concept).contains(&potential_prereq.to_string())
    }
}
```

## Testing DAG Structures

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dag_creation_and_validation() {
        let mut dag = Topology::new_dag();
        
        // Add valid prerequisite chain
        dag.add_prerequisite("A", "B").unwrap();
        dag.add_prerequisite("B", "C").unwrap();
        dag.add_prerequisite("A", "C").unwrap(); // Redundant but valid
        
        // Verify topological sort
        let sorted = dag.get_topological_sort().unwrap();
        assert!(sorted.iter().position(|x| x == "A") < sorted.iter().position(|x| x == "B"));
        assert!(sorted.iter().position(|x| x == "B") < sorted.iter().position(|x| x == "C"));
    }
    
    #[test]
    fn test_cycle_detection() {
        let mut dag = Topology::new_dag();
        
        dag.add_prerequisite("A", "B").unwrap();
        dag.add_prerequisite("B", "C").unwrap();
        
        // This should fail (creates cycle)
        assert!(dag.add_prerequisite("C", "A").is_err());
    }
    
    #[test]
    fn test_learning_path_generation() {
        let dag = Topology::create_math_curriculum();
        let paths = dag.generate_learning_paths("Calculus");
        
        assert!(!paths.is_empty());
        
        // Verify all paths end with the target
        for path in &paths {
            assert_eq!(path.concepts.last().unwrap(), "Calculus");
        }
        
        // Verify prerequisites come before dependents
        for path in &paths {
            for (i, concept) in path.concepts.iter().enumerate() {
                let prereqs = dag.get_prerequisites(concept);
                for prereq in prereqs {
                    let prereq_pos = path.concepts.iter().position(|c| c == &prereq);
                    assert!(prereq_pos.map_or(true, |pos| pos < i));
                }
            }
        }
    }
    
    #[test] 
    fn test_parallel_learning_identification() {
        let dag = Topology::create_math_curriculum();
        let parallel_groups = dag.identify_parallel_learning_opportunities();
        
        // Should find that some concepts can be learned in parallel
        let parallel_possible = parallel_groups.iter()
            .any(|group| group.can_learn_simultaneously);
        assert!(parallel_possible);
    }
}
```

## Best Practices

1. **Validate Acyclicity**: Always check for cycles when adding prerequisites
2. **Minimize Dependencies**: Don't add redundant prerequisite relationships
3. **Consider Parallel Paths**: Identify concepts that can be learned simultaneously
4. **Plan Learning Sequences**: Use topological sorting for curriculum planning
5. **Account for Transfer**: Consider how mastered concepts support new learning
6. **Adaptive Pathways**: Allow multiple valid learning sequences

## Applications

- **Educational Software**: Curriculum sequencing and prerequisite tracking
- **Training Programs**: Skill development and competency management
- **Knowledge Management**: Organizing domain expertise and dependencies
- **Game Design**: Skill trees and progression systems
- **Project Management**: Task dependencies and workflow optimization

## Next Steps

- Explore [General Graphs](./graphs.md) for complex relationship networks
- Learn about [Bayesian Models](../bayesian/intro.md) for uncertainty in prerequisite mastery
- Understand [Task Generation](../tasks/generation.md) for DAG-based assessment
- See [Learning Strategies](../psychology/strategies.md) for pedagogical applications
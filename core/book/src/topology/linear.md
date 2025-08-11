# Linear Structures

Linear structures are the foundation of sequential learning in ABCDeez Core. They represent ordered sequences where each element has a clear position and relationship to its neighbors. This chapter explores how linear topologies model everything from alphabets to numbered lists to procedural steps.

## Conceptual Foundation

### What Makes a Structure Linear?

A linear structure has these properties:
1. **Total ordering**: Every element has a unique position
2. **Single successor**: Each element (except the last) has exactly one next element
3. **Single predecessor**: Each element (except the first) has exactly one previous element
4. **Transitive ordering**: If A < B and B < C, then A < C
5. **No cycles**: You cannot return to an element by moving forward

### Cognitive Importance

Linear structures are fundamental to human cognition:
- **Sequential memory**: How we remember ordered lists
- **Procedural knowledge**: Step-by-step processes
- **Temporal reasoning**: Understanding before/after relationships
- **Counting and enumeration**: Foundation of mathematical thinking

## Implementation in ABCDeez Core

### Creating Linear Topologies

```rust
use abcdeez_core::core::topology::Topology;

// Predefined linear structures
let alphabet = Topology::alphabet();  // A-Z
let numbers = Topology::number_line(1, 10);  // 1-10

// Custom linear structure
let days = Topology::new_linear(vec![
    "Monday".to_string(),
    "Tuesday".to_string(),
    "Wednesday".to_string(),
    "Thursday".to_string(),
    "Friday".to_string(),
]);

// From any sequence
let procedures = Topology::new_linear(vec![
    "Turn on computer",
    "Open browser",
    "Navigate to site",
    "Log in",
    "Complete task",
]
.iter().map(|s| s.to_string()).collect());
```

### Internal Representation

```rust
impl Topology {
    pub fn new_linear(items: Vec<String>) -> Self {
        let n = items.len();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        for (i, label) in items.iter().enumerate() {
            let id = format!("node_{}", i);
            
            // Create node with linear position
            nodes.push(Node {
                id: id.clone(),
                label: label.clone(),
                position: i as f64,  // Simple index-based positioning
            });
            
            node_map.insert(id.clone(), i);

            // Create edge to next node
            if i < n - 1 {
                edges.push(Edge {
                    from: id.clone(),
                    to: format!("node_{}", i + 1),
                    weight: 1.0,  // Unit distance
                });
            }
        }

        Topology {
            topology_type: TopologyType::Linear,
            nodes,
            edges,
            node_map,
        }
    }
}
```

## Operations on Linear Structures

### Navigation

```rust
impl Topology {
    // Move forward one step
    pub fn get_successor(&self, node_id: &str) -> Option<String> {
        self.edges
            .iter()
            .find(|e| e.from == node_id)
            .map(|e| e.to.clone())
    }
    
    // Move backward one step
    pub fn get_predecessor(&self, node_id: &str) -> Option<String> {
        self.edges
            .iter()
            .find(|e| e.to == node_id)
            .map(|e| e.from.clone())
    }
    
    // Jump k positions
    pub fn get_k_jump(&self, start: &str, k: i32) -> Option<String> {
        let start_idx = self.node_map.get(start)?;
        let target_idx = (*start_idx as i32 + k) as usize;
        
        if target_idx < self.nodes.len() {
            Some(format!("node_{}", target_idx))
        } else {
            None  // Out of bounds
        }
    }
}
```

### Distance Calculations

```rust
// Distance between two nodes
pub fn get_distance(&self, from: &str, to: &str) -> Option<usize> {
    let from_idx = self.node_map.get(from)?;
    let to_idx = self.node_map.get(to)?;
    
    Some((*to_idx as i32 - *from_idx as i32).abs() as usize)
}

// Check relative ordering
pub fn is_before(&self, a: &str, b: &str) -> Option<bool> {
    let a_idx = self.node_map.get(a)?;
    let b_idx = self.node_map.get(b)?;
    
    Some(a_idx < b_idx)
}
```

### Segment Extraction

```rust
pub fn get_segment(&self, start: &str, count: usize, reverse: bool) -> Vec<String> {
    let mut result = Vec::new();
    let start_idx = match self.node_map.get(start) {
        Some(idx) => *idx,
        None => return result,
    };
    
    if reverse {
        // Extract backward from start
        for i in (0..count).rev() {
            let idx = start_idx.saturating_sub(i);
            if idx < self.nodes.len() {
                result.push(self.nodes[idx].label.clone());
            }
        }
    } else {
        // Extract forward from start
        for i in 0..count {
            let idx = start_idx + i;
            if idx < self.nodes.len() {
                result.push(self.nodes[idx].label.clone());
            }
        }
    }
    
    result
}
```

## Cognitive Phenomena in Linear Structures

### Serial Position Effects

Linear structures exhibit strong serial position effects:

```rust
pub fn calculate_serial_position_strength(position: usize, total: usize) -> f64 {
    let relative_pos = position as f64 / total as f64;
    
    // Primacy effect (beginning advantage)
    let primacy = (-3.0 * relative_pos).exp();
    
    // Recency effect (end advantage)
    let recency = (-3.0 * (1.0 - relative_pos)).exp();
    
    // Combined U-shaped curve
    0.5 + 0.3 * primacy + 0.2 * recency
}
```

### Chunk Boundaries

People naturally segment linear sequences into chunks:

```rust
pub struct ChunkDetector {
    chunk_size: usize,
    overlap: f64,
}

impl ChunkDetector {
    pub fn detect_chunks(&self, topology: &Topology) -> Vec<ChunkBoundary> {
        let mut boundaries = Vec::new();
        let n = topology.nodes.len();
        
        // Common chunk sizes: 3-4 for beginners, 5-7 for experts
        for i in (self.chunk_size..n).step_by(self.chunk_size) {
            boundaries.push(ChunkBoundary {
                position: i,
                strength: 0.8,  // Initial boundary strength
            });
        }
        
        // Alphabetic special cases (after G, M, S)
        if topology.nodes[0].label == "A" {
            boundaries.push(ChunkBoundary { position: 7, strength: 0.6 });
            boundaries.push(ChunkBoundary { position: 13, strength: 0.6 });
            boundaries.push(ChunkBoundary { position: 19, strength: 0.6 });
        }
        
        boundaries
    }
}
```

### Symbolic Distance Effect

Response time increases with distance between items:

```rust
pub fn predict_comparison_time(distance: usize) -> f64 {
    // Base response time
    let base_rt = 500.0;
    
    // Linear increase with distance
    let distance_penalty = 50.0 * distance as f64;
    
    // Logarithmic component for large distances
    let log_component = 100.0 * (distance as f64 + 1.0).ln();
    
    base_rt + distance_penalty + log_component
}
```

## Learning Patterns

### Forward vs. Backward Asymmetry

People learn forward sequences faster than backward:

```rust
pub struct DirectionalLearning {
    forward_strength: f64,
    backward_strength: f64,
}

impl DirectionalLearning {
    pub fn update(&mut self, direction: Direction, success: bool) {
        match direction {
            Direction::Forward => {
                if success {
                    self.forward_strength += 0.15;  // Faster learning
                } else {
                    self.forward_strength -= 0.05;
                }
            }
            Direction::Backward => {
                if success {
                    self.backward_strength += 0.10;  // Slower learning
                } else {
                    self.backward_strength -= 0.08;  // More fragile
                }
            }
        }
    }
}
```

### Anchoring Effects

Certain positions serve as cognitive anchors:

```rust
pub fn get_anchor_points(topology: &Topology) -> Vec<usize> {
    let mut anchors = Vec::new();
    let n = topology.nodes.len();
    
    // First and last are always anchors
    anchors.push(0);
    anchors.push(n - 1);
    
    // Middle point for odd-length sequences
    if n % 2 == 1 {
        anchors.push(n / 2);
    }
    
    // Quarter points for longer sequences
    if n >= 8 {
        anchors.push(n / 4);
        anchors.push(3 * n / 4);
    }
    
    anchors
}
```

## Common Linear Structures

### The Alphabet

```rust
impl Topology {
    pub fn alphabet() -> Self {
        let letters: Vec<String> = (b'A'..=b'Z')
            .map(|c| (c as char).to_string())
            .collect();
        
        let mut topo = Self::new_linear(letters);
        
        // Add alphabet-specific metadata
        topo.metadata.insert("structure_type", "alphabet");
        topo.metadata.insert("cultural_significance", "high");
        topo.metadata.insert("learned_age", "3-5");
        
        topo
    }
}
```

### Number Lines

```rust
impl Topology {
    pub fn number_line(start: i32, end: i32) -> Self {
        let numbers: Vec<String> = (start..=end)
            .map(|n| n.to_string())
            .collect();
        
        let mut topo = Self::new_linear(numbers);
        
        // Numbers have additional properties
        topo.metadata.insert("supports_arithmetic", "true");
        topo.metadata.insert("infinite_extension", "true");
        
        topo
    }
}
```

### Temporal Sequences

```rust
pub fn months_of_year() -> Topology {
    Topology::new_linear(vec![
        "January", "February", "March", "April",
        "May", "June", "July", "August",
        "September", "October", "November", "December"
    ].iter().map(|s| s.to_string()).collect())
}

pub fn hours_of_day() -> Topology {
    let hours: Vec<String> = (0..24)
        .map(|h| format!("{:02}:00", h))
        .collect();
    Topology::new_linear(hours)
}
```

## Optimization Techniques

### Caching Distance Matrices

```rust
pub struct OptimizedLinearTopology {
    topology: Topology,
    distance_cache: HashMap<(String, String), usize>,
}

impl OptimizedLinearTopology {
    pub fn new(items: Vec<String>) -> Self {
        let topology = Topology::new_linear(items);
        let mut distance_cache = HashMap::new();
        
        // Precompute all distances
        for i in 0..topology.nodes.len() {
            for j in 0..topology.nodes.len() {
                let from = format!("node_{}", i);
                let to = format!("node_{}", j);
                let distance = (j as i32 - i as i32).abs() as usize;
                distance_cache.insert((from, to), distance);
            }
        }
        
        Self { topology, distance_cache }
    }
    
    pub fn get_distance(&self, from: &str, to: &str) -> Option<usize> {
        self.distance_cache.get(&(from.to_string(), to.to_string())).copied()
    }
}
```

### Efficient Segment Operations

```rust
pub struct SegmentIterator<'a> {
    topology: &'a Topology,
    current_idx: usize,
    remaining: usize,
    reverse: bool,
}

impl<'a> Iterator for SegmentIterator<'a> {
    type Item = String;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        
        let idx = if self.reverse {
            self.current_idx.saturating_sub(self.remaining - 1)
        } else {
            self.current_idx
        };
        
        if idx < self.topology.nodes.len() {
            self.remaining -= 1;
            if !self.reverse {
                self.current_idx += 1;
            }
            Some(self.topology.nodes[idx].label.clone())
        } else {
            None
        }
    }
}
```

## Testing Linear Structures

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_linear_properties() {
        let topo = Topology::alphabet();
        
        // Test transitivity
        assert!(topo.is_before("A", "B").unwrap());
        assert!(topo.is_before("B", "C").unwrap());
        assert!(topo.is_before("A", "C").unwrap());
        
        // Test uniqueness
        assert_eq!(topo.get_distance("A", "Z"), Some(25));
        
        // Test boundaries
        assert!(topo.get_successor("Z").is_none());
        assert!(topo.get_predecessor("A").is_none());
    }
    
    #[test]
    fn test_segment_extraction() {
        let topo = Topology::alphabet();
        
        let forward = topo.get_segment("D", 3, false);
        assert_eq!(forward, vec!["D", "E", "F"]);
        
        let backward = topo.get_segment("D", 3, true);
        assert_eq!(backward, vec!["B", "C", "D"]);
    }
}
```

## Best Practices

1. **Use appropriate chunk sizes**: 3-5 items for beginners, 5-7 for experts
2. **Consider direction**: Forward learning is typically easier
3. **Leverage anchor points**: Use beginning, middle, and end as reference points
4. **Account for serial position**: Expect U-shaped memory curves
5. **Optimize for common operations**: Cache distances if frequently accessed

## Applications

- **Educational sequences**: Teaching ordered content
- **Procedural training**: Step-by-step instructions
- **Memory tasks**: Serial recall experiments
- **Navigation interfaces**: Linear menus and lists
- **Time-based learning**: Historical timelines

## Next Steps

- Explore [Cyclic Structures](./cyclic.md) for repeating patterns
- Learn about [DAGs](./dag.md) for prerequisite relationships
- Understand [General Graphs](./graphs.md) for complex networks
- See [Topology API](../api/core.md#topology) for implementation details
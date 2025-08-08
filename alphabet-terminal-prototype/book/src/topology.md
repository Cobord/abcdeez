# Topology Models

Topologies define the structure that learners are trying to master. The system supports multiple topology types, each presenting unique learning challenges.

## Core Topology Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub topology_type: TopologyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,       // Unique identifier (e.g., "node_0")
    pub label: String,    // Human-readable label (e.g., "A")
    pub position: usize,  // Position in sequence
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,     // Source node ID
    pub to: String,       // Target node ID
    pub weight: f64,      // Edge strength/distance
}
```

## Topology Types

### Linear Topology

The simplest structure - a unidirectional chain:

```rust
impl Topology {
    pub fn linear(labels: Vec<String>) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        for (i, label) in labels.iter().enumerate() {
            nodes.push(Node {
                id: format!("node_{}", i),
                label: label.clone(),
                position: i,
            });
            
            // Add edge to next node (except for last)
            if i < labels.len() - 1 {
                edges.push(Edge {
                    from: format!("node_{}", i),
                    to: format!("node_{}", i + 1),
                    weight: 1.0,
                });
            }
        }
        
        Topology {
            nodes,
            edges,
            topology_type: TopologyType::Linear,
        }
    }
}
```

**Properties:**
- Clear start and end points
- Distance metric is absolute difference in position
- No ambiguity in ordering

### Cyclic Topology

A ring structure where the last element connects to the first:

```rust
impl Topology {
    pub fn cyclic(labels: Vec<String>) -> Self {
        let mut topology = Self::linear(labels.clone());
        
        // Add edge from last to first
        let n = topology.nodes.len();
        topology.edges.push(Edge {
            from: format!("node_{}", n - 1),
            to: "node_0".to_string(),
            weight: 1.0,
        });
        
        topology.topology_type = TopologyType::Cyclic;
        topology
    }
}
```

**Properties:**
- No inherent start/end
- Distance metric must consider wrap-around
- Multiple valid paths between nodes

### Hierarchical Topology

Includes higher-level structure (chunks):

```rust
impl Topology {
    pub fn alphabet() -> Self {
        let mut topology = Self::linear(
            ('A'..='Z').map(|c| c.to_string()).collect()
        );
        
        // Add chunk boundaries (e.g., ABCD, EFGH, ...)
        // These affect processing speed and error patterns
        topology
    }
}
```

## Navigation Methods

The topology provides methods for traversal:

```rust
impl Topology {
    pub fn get_successor(&self, label: &str) -> Option<String> {
        let node = self.get_node_by_label(label)?;
        
        // Find outgoing edge
        for edge in &self.edges {
            if edge.from == node.id {
                let target = self.get_node_by_id(&edge.to)?;
                return Some(target.label.clone());
            }
        }
        
        None
    }
    
    pub fn get_predecessor(&self, label: &str) -> Option<String> {
        let node = self.get_node_by_label(label)?;
        
        // Find incoming edge
        for edge in &self.edges {
            if edge.to == node.id {
                let source = self.get_node_by_id(&edge.from)?;
                return Some(source.label.clone());
            }
        }
        
        None
    }
}
```

## Distance Metrics

Different topologies require different distance calculations:

```rust
impl Topology {
    pub fn calculate_distance(&self, from: &str, to: &str) -> usize {
        match self.topology_type {
            TopologyType::Linear => {
                self.linear_distance(from, to)
            }
            TopologyType::Cyclic => {
                self.cyclic_distance(from, to)
            }
            TopologyType::Graph => {
                self.shortest_path_distance(from, to)
            }
        }
    }
    
    fn linear_distance(&self, from: &str, to: &str) -> usize {
        let from_pos = self.get_node_by_label(from).unwrap().position;
        let to_pos = self.get_node_by_label(to).unwrap().position;
        
        from_pos.abs_diff(to_pos)
    }
    
    fn cyclic_distance(&self, from: &str, to: &str) -> usize {
        let from_pos = self.get_node_by_label(from).unwrap().position;
        let to_pos = self.get_node_by_label(to).unwrap().position;
        let n = self.nodes.len();
        
        // Consider both directions around the cycle
        let forward = (to_pos + n - from_pos) % n;
        let backward = (from_pos + n - to_pos) % n;
        
        forward.min(backward)
    }
}
```

## Domain-Specific Topologies

### Alphabet

The classic example with special properties:

```rust
pub fn alphabet() -> Self {
    let labels: Vec<String> = ('A'..='Z').map(|c| c.to_string()).collect();
    let mut topology = Self::linear(labels);
    
    // Alphabet-specific features:
    // 1. Chunk boundaries (ABCD, EFGH, etc.)
    // 2. Vowel/consonant categories
    // 3. Common sequences (ABC, XYZ)
    
    topology
}
```

### Musical Scales

Music provides rich hierarchical structure:

```rust
pub fn chromatic_scale() -> Self {
    let notes = vec![
        "C", "C#", "D", "D#", "E", "F", 
        "F#", "G", "G#", "A", "A#", "B"
    ];
    
    let mut topology = Self::cyclic(
        notes.iter().map(|s| s.to_string()).collect()
    );
    
    // Add octave relationships
    // Add major/minor scale subsets
    // Add chord relationships
    
    topology
}
```

### Chess Board

2D grid with specific movement rules:

```rust
pub fn chess_board() -> Self {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    
    // Create 8x8 grid
    for rank in 1..=8 {
        for file in 'a'..='h' {
            let label = format!("{}{}", file, rank);
            let id = format!("node_{}_{}", file as usize - 'a' as usize, rank - 1);
            
            nodes.push(Node {
                id: id.clone(),
                label,
                position: (rank - 1) * 8 + (file as usize - 'a' as usize),
            });
        }
    }
    
    // Add edges for adjacent squares
    // (More complex for actual chess moves)
    
    Topology {
        nodes,
        edges,
        topology_type: TopologyType::Grid,
    }
}
```

## Topology Analysis

The system analyzes topology properties:

```rust
impl Topology {
    pub fn analyze(&self) -> TopologyAnalysis {
        TopologyAnalysis {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            is_connected: self.is_connected(),
            diameter: self.compute_diameter(),
            average_degree: self.compute_average_degree(),
            clustering_coefficient: self.compute_clustering(),
        }
    }
    
    fn compute_diameter(&self) -> usize {
        // Maximum shortest path between any two nodes
        let mut max_distance = 0;
        
        for from in &self.nodes {
            for to in &self.nodes {
                let dist = self.calculate_distance(&from.label, &to.label);
                max_distance = max_distance.max(dist);
            }
        }
        
        max_distance
    }
}
```

## Learning Implications

Different topologies affect learning in predictable ways:

### Linear Topology Learning Patterns
- **Serial position effects**: Better memory for beginning/end
- **Distance effects**: Harder to judge relative position of distant items
- **Asymmetry**: Forward navigation often easier than backward

### Cyclic Topology Learning Patterns
- **No anchor points**: Harder to establish reference frame
- **Modular arithmetic**: Must learn wrap-around
- **Strategy diversity**: Multiple valid mental models

### Hierarchical Topology Learning Patterns
- **Chunk effects**: Faster within-chunk than between-chunk
- **Categorical knowledge**: Learn group membership before exact position
- **Transfer effects**: Knowledge of one chunk helps with others

## Implementation Example

Let's trace through task generation for different topologies:

```rust
fn generate_task_for_topology(topology: &Topology) -> Task {
    match topology.topology_type {
        TopologyType::Linear => {
            // Favor middle items (hardest to learn)
            let middle = topology.nodes.len() / 2;
            let item = &topology.nodes[middle].label;
            
            Task {
                task_type: TaskType::Successor { 
                    item: item.clone() 
                },
                // ...
            }
        }
        
        TopologyType::Cyclic => {
            // Test wrap-around understanding
            let last = &topology.nodes.last().unwrap().label;
            
            Task {
                task_type: TaskType::Successor { 
                    item: last.clone() 
                },
                prompt: format!("What comes after {}?", last),
                correct_answer: topology.nodes[0].label.clone(),
                // ...
            }
        }
        
        TopologyType::Hierarchical => {
            // Test chunk boundary crossing
            // Find a chunk boundary and test items on either side
            // ...
        }
    }
}
```

## Summary

Topologies provide:
- **Structure definition**: Nodes, edges, and relationships
- **Navigation methods**: Successor, predecessor, distance
- **Domain modeling**: Alphabet, music, chess, etc.
- **Learning scaffolding**: Different structures teach different skills

The topology forms the foundation that learners build their mental models upon. Next, we'll explore how the learner model represents and updates knowledge about these structures.
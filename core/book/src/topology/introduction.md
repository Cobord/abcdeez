# Topology System Introduction

The topology system in ABCDeez Core provides a flexible framework for representing structured knowledge domains. Whether modeling simple sequences like the alphabet or complex prerequisite graphs in educational curricula, the topology system captures the relationships between elements that learners must master.

## Core Concepts

### What is a Topology?

In ABCDeez Core, a topology represents the structure of a knowledge domain:
- **Nodes**: Individual elements to be learned (letters, concepts, skills)
- **Edges**: Relationships between elements (succession, prerequisites, associations)
- **Positions**: Spatial or conceptual locations of nodes
- **Distances**: Metric relationships between nodes

### Why Topology Matters

The structure of knowledge profoundly affects how it's learned:

1. **Sequential Dependencies**: Some knowledge must be learned in order
2. **Conceptual Distance**: Related concepts are easier to learn together
3. **Transfer Effects**: Learning one element affects learning others
4. **Chunking Patterns**: Natural groupings emerge from structure

## Topology Types

ABCDeez Core supports four fundamental topology types:

### 1. Linear Topology
Perfect for ordered sequences:
```rust
let alphabet = Topology::alphabet();
let numbers = Topology::new_linear(
    (1..=10).map(|n| n.to_string()).collect()
);
```

### 2. Cyclic Topology
For repeating patterns:
```rust
let days = Topology::days_of_week();
let months = Topology::months_of_year();
```

### 3. Partial Order (DAG)
For prerequisite structures:
```rust
let curriculum = Topology::new_dag(
    vec!["Algebra", "Geometry", "Calculus"],
    vec![
        ("Algebra", "Calculus"),
        ("Geometry", "Calculus"),
    ]
);
```

### 4. General Graph
For arbitrary relationships:
```rust
let semantic_network = Topology::new_graph(
    vec!["Dog", "Cat", "Pet", "Animal"],
    vec![
        ("Dog", "Pet", 0.9),
        ("Cat", "Pet", 0.9),
        ("Pet", "Animal", 0.8),
    ]
);
```

## Core Data Structures

### Node Representation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,       // Unique identifier
    pub label: String,    // Human-readable label
    pub position: f64,    // Position in topology space
}
```

### Edge Representation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,     // Source node ID
    pub to: String,       // Target node ID
    pub weight: f64,      // Connection strength
}
```

### Topology Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub topology_type: TopologyType,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub node_map: HashMap<String, usize>,
}
```

## Key Operations

### Navigation Operations

```rust
impl Topology {
    // Get the next node in sequence
    pub fn get_successor(&self, node_id: &str) -> Option<String>;
    
    // Get the previous node
    pub fn get_predecessor(&self, node_id: &str) -> Option<String>;
    
    // Jump k steps forward/backward
    pub fn get_k_jump(&self, start: &str, k: i32) -> Option<String>;
    
    // Get a segment of nodes
    pub fn get_segment(&self, start: &str, count: usize, reverse: bool) -> Vec<String>;
}
```

### Distance and Path Operations

```rust
impl Topology {
    // Calculate distance between nodes
    pub fn get_distance(&self, from: &str, to: &str) -> Option<usize>;
    
    // Find shortest path
    pub fn shortest_path(&self, from: &str, to: &str) -> Option<Vec<String>>;
    
    // Check if path exists
    pub fn has_path(&self, from: &str, to: &str) -> Option<bool>;
    
    // Check ordering relationship
    pub fn is_before(&self, a: &str, b: &str) -> Option<bool>;
}
```

## Position Encoding

Node positions encode structural information:

### Linear Topology
Positions are simple indices:
```rust
// A=0.0, B=1.0, C=2.0, ..., Z=25.0
position = index as f64
```

### Cyclic Topology
Positions are angles on a circle:
```rust
// Distribute evenly around circle
position = (2.0 * PI * index) / n_nodes
```

### DAG Topology
Positions reflect topological depth:
```rust
// Level-based positioning
position = topological_level as f64
```

## Distance Metrics

Different topologies use appropriate distance metrics:

### Linear Distance
Simple absolute difference:
```rust
distance = |position_a - position_b|
```

### Cyclic Distance
Minimum arc length:
```rust
forward = (b - a + n) % n
backward = (a - b + n) % n
distance = min(forward, backward)
```

### Graph Distance
Shortest path length using Dijkstra's algorithm:
```rust
distance = dijkstra(from, to).path_length
```

## Practical Examples

### Example 1: Alphabet Learning

```rust
let topology = Topology::alphabet();

// Navigate through alphabet
let after_m = topology.get_successor("node_12"); // "node_13" (N)
let before_m = topology.get_predecessor("node_12"); // "node_11" (L)

// Calculate distances
let dist_a_to_z = topology.get_distance("node_0", "node_25"); // 25

// Get segments for chunking
let chunk = topology.get_segment("D", 3, false); // ["D", "E", "F"]
```

### Example 2: Curriculum Planning

```rust
let curriculum = Topology::new_dag(
    vec![
        "Basic Math",
        "Algebra I",
        "Geometry",
        "Algebra II", 
        "Pre-Calculus",
        "Calculus"
    ],
    vec![
        ("Basic Math", "Algebra I"),
        ("Basic Math", "Geometry"),
        ("Algebra I", "Algebra II"),
        ("Geometry", "Pre-Calculus"),
        ("Algebra II", "Pre-Calculus"),
        ("Pre-Calculus", "Calculus"),
    ]
);

// Get learning order
let order = curriculum.get_topological_sort();
// ["Basic Math", "Algebra I", "Geometry", "Algebra II", "Pre-Calculus", "Calculus"]

// Check prerequisites
let can_take_calculus = curriculum.has_path("Algebra II", "Calculus"); // true
```

### Example 3: Semantic Networks

```rust
let concepts = Topology::new_graph(
    vec!["Dog", "Cat", "Bird", "Mammal", "Pet", "Animal"],
    vec![
        ("Dog", "Mammal", 1.0),
        ("Cat", "Mammal", 1.0),
        ("Bird", "Animal", 1.0),
        ("Mammal", "Animal", 1.0),
        ("Dog", "Pet", 0.9),
        ("Cat", "Pet", 0.9),
        ("Bird", "Pet", 0.3),
    ]
);

// Find conceptual paths
let path = concepts.shortest_path("Dog", "Animal");
// ["Dog", "Mammal", "Animal"]
```

## Integration with Learning Models

Topologies integrate seamlessly with learner models:

```rust
let topology = Topology::alphabet();
let mut learner = LearnerModel::new("student_001", &topology);

// Learner's mental model initialized from topology
for node in &topology.nodes {
    learner.initialize_node_embedding(&node);
}

// Task generation uses topology structure
let task = Task::from_topology(&topology, TaskType::Successor);
```

## Performance Characteristics

| Operation | Linear | Cyclic | DAG | Graph |
|-----------|--------|--------|-----|-------|
| get_successor | O(1) | O(1) | O(E) | O(E) |
| get_distance | O(1) | O(1) | O(V+E) | O(V²) |
| shortest_path | O(1) | O(1) | O(V+E) | O(V²) |
| topological_sort | N/A | N/A | O(V+E) | N/A |

Where V = number of vertices, E = number of edges

## Extending the Topology System

### Custom Topology Types

```rust
pub enum CustomTopology {
    Tree(TreeParams),
    Lattice(LatticeParams),
    SmallWorld(SmallWorldParams),
}

impl CustomTopology {
    pub fn to_topology(self) -> Topology {
        match self {
            CustomTopology::Tree(params) => build_tree(params),
            CustomTopology::Lattice(params) => build_lattice(params),
            CustomTopology::SmallWorld(params) => build_small_world(params),
        }
    }
}
```

### Dynamic Topologies

Support for topologies that change during learning:

```rust
pub struct DynamicTopology {
    base: Topology,
    modifications: Vec<TopologyChange>,
}

impl DynamicTopology {
    pub fn add_edge(&mut self, from: &str, to: &str, weight: f64) {
        self.modifications.push(TopologyChange::AddEdge {
            from: from.to_string(),
            to: to.to_string(),
            weight,
        });
    }
    
    pub fn current(&self) -> Topology {
        self.apply_modifications()
    }
}
```

## Best Practices

1. **Choose Appropriate Structure**: Match topology to domain characteristics
2. **Validate Connectivity**: Ensure all nodes are reachable
3. **Consider Cognitive Load**: Complex topologies may overwhelm learners
4. **Use Meaningful Labels**: Node labels should be intuitive
5. **Optimize for Common Operations**: Structure data for efficient access

## Next Steps

- Explore [Linear Structures](./linear.md) in detail
- Learn about [Cyclic Structures](./cyclic.md)
- Understand [DAGs and Partial Orders](./dag.md)
- Study [General Graphs](./graphs.md)
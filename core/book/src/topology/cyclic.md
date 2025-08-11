# Cyclic Structures

Cyclic structures represent sequences that wrap around on themselves, creating endless loops without clear beginning or end points. In ABCDeez Core, these topologies model repeating patterns like days of the week, months, seasons, and other cyclical phenomena that are fundamental to human experience.

## Conceptual Foundation

### What Makes a Structure Cyclic?

A cyclic structure has these essential properties:
1. **Circular ordering**: Every element has exactly one successor and predecessor
2. **No endpoints**: Unlike linear structures, there's no first or last element
3. **Wrap-around behavior**: Moving past the "end" returns to the "beginning"
4. **Bidirectional distance**: Distance can be measured in either direction
5. **Rotational symmetry**: Any element can serve as a starting point

### Cognitive Significance

Cyclic structures reflect natural and cultural rhythms:
- **Temporal cycles**: Days, weeks, months, seasons
- **Spatial cycles**: Compass directions, clock positions
- **Biological rhythms**: Circadian cycles, life stages
- **Cultural patterns**: Holidays, ceremonies, traditions

## Implementation in ABCDeez Core

### Creating Cyclic Topologies

```rust
use abcdeez_core::core::topology::Topology;

// Predefined cyclic structures
let weekdays = Topology::days_of_week();
let months = Topology::months_of_year();

// Custom cyclic structure  
let seasons = Topology::new_cyclic(vec![
    "Spring".to_string(),
    "Summer".to_string(),
    "Fall".to_string(),
    "Winter".to_string(),
]);

// Clock positions
let hours = Topology::new_cyclic(
    (1..=12).map(|h| h.to_string()).collect()
);
```

### Internal Representation

```rust
impl Topology {
    pub fn new_cyclic(items: Vec<String>) -> Self {
        let n = items.len();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        for (i, label) in items.iter().enumerate() {
            let id = format!("node_{}", i);
            
            // Position as angle around circle
            let angle = (2.0 * PI * i as f64) / n as f64;
            
            nodes.push(Node {
                id: id.clone(),
                label: label.clone(),
                position: angle,  // Angular position in radians
            });
            
            node_map.insert(id.clone(), i);

            // Create edge to next node (wrapping around)
            let next_idx = (i + 1) % n;
            edges.push(Edge {
                from: id.clone(),
                to: format!("node_{}", next_idx),
                weight: 1.0,
            });
        }

        Topology {
            topology_type: TopologyType::Cyclic,
            nodes,
            edges,
            node_map,
        }
    }
}
```

## Operations on Cyclic Structures

### Navigation with Wrap-Around

```rust
impl Topology {
    pub fn get_successor(&self, node_id: &str) -> Option<String> {
        match self.topology_type {
            TopologyType::Cyclic => {
                let idx = *self.node_map.get(node_id)?;
                let next_idx = (idx + 1) % self.nodes.len();
                Some(format!("node_{}", next_idx))
            }
            _ => self.edges.iter()
                .find(|e| e.from == node_id)
                .map(|e| e.to.clone())
        }
    }

    pub fn get_predecessor(&self, node_id: &str) -> Option<String> {
        match self.topology_type {
            TopologyType::Cyclic => {
                let idx = *self.node_map.get(node_id)?;
                let n = self.nodes.len();
                let prev_idx = (idx + n - 1) % n;
                Some(format!("node_{}", prev_idx))
            }
            _ => self.edges.iter()
                .find(|e| e.to == node_id)
                .map(|e| e.from.clone())
        }
    }
}
```

### Bidirectional Distance Calculation

```rust
pub fn get_distance(&self, from: &str, to: &str) -> Option<usize> {
    match self.topology_type {
        TopologyType::Cyclic => {
            let from_idx = *self.node_map.get(from)?;
            let to_idx = *self.node_map.get(to)?;
            let n = self.nodes.len();
            
            // Calculate distance in both directions
            let forward = (to_idx + n - from_idx) % n;
            let backward = (from_idx + n - to_idx) % n;
            
            // Return minimum distance
            Some(forward.min(backward))
        }
        _ => {
            // Linear distance calculation
            let from_idx = *self.node_map.get(from)?;
            let to_idx = *self.node_map.get(to)?;
            Some((to_idx as i32 - from_idx as i32).abs() as usize)
        }
    }
}
```

### K-Jump Operations

```rust
pub fn get_k_jump(&self, start: &str, k: i32) -> Option<String> {
    match self.topology_type {
        TopologyType::Cyclic => {
            let start_idx = *self.node_map.get(start)?;
            let n = self.nodes.len() as i32;
            
            // Handle both positive and negative jumps
            let target_idx = (start_idx as i32 + k).rem_euclid(n) as usize;
            
            Some(format!("node_{}", target_idx))
        }
        _ => {
            // Linear k-jump with bounds checking
            let start_idx = *self.node_map.get(start)? as i32;
            let target_idx = start_idx + k;
            
            if target_idx >= 0 && target_idx < self.nodes.len() as i32 {
                Some(format!("node_{}", target_idx))
            } else {
                None
            }
        }
    }
}
```

## Cognitive Phenomena in Cyclic Structures

### Lack of Serial Position Effects

Unlike linear structures, cyclic structures don't show traditional primacy/recency effects:

```rust
pub fn calculate_cyclic_memory_strength(position: usize, total: usize) -> f64 {
    // Cyclic structures have more uniform memory strength
    let base_strength = 0.7;
    
    // Small random variation to account for individual differences
    let variation = 0.1 * (position as f64 * 2.0 * PI / total as f64).sin();
    
    base_strength + variation
}
```

### Reference Point Effects

People often anchor cyclic knowledge at culturally significant points:

```rust
pub struct CyclicReferencePoints {
    pub natural_anchors: Vec<usize>,    // e.g., January, Monday
    pub cultural_anchors: Vec<usize>,   // e.g., New Year, Weekend
    pub personal_anchors: Vec<usize>,   // e.g., Birthday, Anniversary
}

impl CyclicReferencePoints {
    pub fn for_weekdays() -> Self {
        Self {
            natural_anchors: vec![0],      // Monday (work week start)
            cultural_anchors: vec![5, 6],  // Weekend (Saturday, Sunday)
            personal_anchors: vec![],      // Individual-specific
        }
    }
    
    pub fn for_months() -> Self {
        Self {
            natural_anchors: vec![0, 5, 8, 11], // Seasonal transitions
            cultural_anchors: vec![0, 11],       // New Year, December holidays
            personal_anchors: vec![],            // Individual birthdays, etc.
        }
    }
}
```

### Directional Asymmetries

Some cyclic structures have preferred directions:

```rust
pub struct DirectionalPreference {
    pub forward_strength: f64,
    pub backward_strength: f64,
    pub preferred_direction: CyclicDirection,
}

#[derive(Debug, Clone)]
pub enum CyclicDirection {
    Clockwise,
    Counterclockwise,
    NoPreference,
}

impl DirectionalPreference {
    pub fn for_time_cycles() -> Self {
        // Time moves forward
        Self {
            forward_strength: 0.8,
            backward_strength: 0.4,
            preferred_direction: CyclicDirection::Clockwise,
        }
    }
    
    pub fn for_spatial_cycles() -> Self {
        // Spatial cycles may have no inherent direction
        Self {
            forward_strength: 0.6,
            backward_strength: 0.6,
            preferred_direction: CyclicDirection::NoPreference,
        }
    }
}
```

## Specialized Distance Metrics

### Angular Distance

For cycles representing angular positions:

```rust
pub fn angular_distance(from_angle: f64, to_angle: f64) -> f64 {
    let diff = (to_angle - from_angle).abs();
    diff.min(2.0 * PI - diff)
}

pub fn angular_interpolation(from: f64, to: f64, t: f64) -> f64 {
    let diff = to - from;
    let shortest_diff = if diff.abs() > PI {
        if diff > 0.0 { diff - 2.0 * PI } else { diff + 2.0 * PI }
    } else {
        diff
    };
    
    (from + t * shortest_diff).rem_euclid(2.0 * PI)
}
```

### Temporal Distance

For time-based cycles with unequal intervals:

```rust
pub struct TemporalCycle {
    items: Vec<String>,
    durations: Vec<f64>,  // Days, hours, etc.
}

impl TemporalCycle {
    pub fn months_of_year() -> Self {
        Self {
            items: vec![
                "January", "February", "March", "April",
                "May", "June", "July", "August", 
                "September", "October", "November", "December"
            ].iter().map(|s| s.to_string()).collect(),
            durations: vec![31.0, 28.25, 31.0, 30.0, 31.0, 30.0,
                           31.0, 31.0, 30.0, 31.0, 30.0, 31.0],
        }
    }
    
    pub fn temporal_distance(&self, from_idx: usize, to_idx: usize) -> f64 {
        let n = self.items.len();
        let mut distance = 0.0;
        let mut current = from_idx;
        
        while current != to_idx {
            distance += self.durations[current];
            current = (current + 1) % n;
        }
        
        // Check if going backward is shorter
        let mut backward_distance = 0.0;
        let mut current = from_idx;
        
        while current != to_idx {
            current = (current + n - 1) % n;
            backward_distance += self.durations[current];
        }
        
        distance.min(backward_distance)
    }
}
```

## Common Cyclic Structures

### Days of the Week

```rust
impl Topology {
    pub fn days_of_week() -> Self {
        let days = vec![
            "Monday", "Tuesday", "Wednesday", "Thursday",
            "Friday", "Saturday", "Sunday"
        ];
        
        let mut topology = Self::new_cyclic(
            days.iter().map(|s| s.to_string()).collect()
        );
        
        // Add cultural metadata
        topology.set_cultural_anchor(0);  // Monday (work week)
        topology.set_cultural_anchor(5);  // Weekend start
        
        topology
    }
}
```

### Seasonal Cycles

```rust
pub fn seasons() -> Topology {
    let mut topology = Topology::new_cyclic(vec![
        "Spring".to_string(),
        "Summer".to_string(), 
        "Fall".to_string(),
        "Winter".to_string(),
    ]);
    
    // Add seasonal transitions as natural anchors
    for i in 0..4 {
        topology.set_natural_anchor(i);
    }
    
    topology
}
```

### Clock Positions

```rust
pub fn clock_12hour() -> Topology {
    let hours: Vec<String> = (1..=12).map(|h| {
        format!("{}:00", h)
    }).collect();
    
    let mut topology = Topology::new_cyclic(hours);
    
    // 12, 3, 6, 9 are natural reference points
    topology.set_natural_anchor(11); // 12 o'clock
    topology.set_natural_anchor(2);  // 3 o'clock  
    topology.set_natural_anchor(5);  // 6 o'clock
    topology.set_natural_anchor(8);  // 9 o'clock
    
    topology
}
```

## Learning Dynamics

### Phase Transitions

Learning cyclic structures often involves recognizing phase transitions:

```rust
pub struct PhaseTransition {
    pub from_phase: String,
    pub to_phase: String,
    pub transition_strength: f64,
    pub difficulty: f64,
}

impl PhaseTransition {
    pub fn detect_transitions(topology: &Topology) -> Vec<PhaseTransition> {
        let mut transitions = Vec::new();
        
        for i in 0..topology.nodes.len() {
            let current = &topology.nodes[i];
            let next_idx = (i + 1) % topology.nodes.len();
            let next = &topology.nodes[next_idx];
            
            // Check if this represents a significant transition
            let difficulty = if Self::is_major_transition(current, next) {
                0.8  // Hard transitions (e.g., Sunday -> Monday)
            } else {
                0.3  // Easy transitions (e.g., Tuesday -> Wednesday)
            };
            
            transitions.push(PhaseTransition {
                from_phase: current.label.clone(),
                to_phase: next.label.clone(),
                transition_strength: 0.5,
                difficulty,
            });
        }
        
        transitions
    }
    
    fn is_major_transition(from: &Node, to: &Node) -> bool {
        // Week boundaries, season changes, etc.
        match (&from.label[..], &to.label[..]) {
            ("Sunday", "Monday") => true,      // Weekend to workweek
            ("Friday", "Saturday") => true,    // Workweek to weekend
            ("Winter", "Spring") => true,      // Season change
            ("December", "January") => true,   // Year boundary
            _ => false,
        }
    }
}
```

### Wrap-Around Difficulties

The wrap-around property can be challenging:

```rust
pub fn assess_wraparound_difficulty(
    topology: &Topology,
    from: &str,
    to: &str,
) -> f64 {
    let from_idx = topology.node_map[from];
    let to_idx = topology.node_map[to];
    let n = topology.nodes.len();
    
    let forward_distance = (to_idx + n - from_idx) % n;
    let backward_distance = (from_idx + n - to_idx) % n;
    
    // If shortest path crosses the boundary, it's harder
    if forward_distance != backward_distance {
        let crosses_boundary = forward_distance > n / 2 || backward_distance > n / 2;
        if crosses_boundary {
            0.7  // Increased difficulty for boundary crossing
        } else {
            0.4  // Normal difficulty
        }
    } else {
        0.4  // Equal distances in both directions
    }
}
```

## Testing Cyclic Structures

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cyclic_navigation() {
        let days = Topology::days_of_week();
        
        // Test forward navigation
        let monday_succ = days.get_successor("node_0").unwrap();
        assert_eq!(days.get_node_by_id(&monday_succ).unwrap().label, "Tuesday");
        
        // Test wrap-around
        let sunday_succ = days.get_successor("node_6").unwrap();
        assert_eq!(days.get_node_by_id(&sunday_succ).unwrap().label, "Monday");
    }
    
    #[test]
    fn test_bidirectional_distance() {
        let days = Topology::days_of_week();
        
        // Monday to Wednesday: 2 steps forward
        assert_eq!(days.get_distance("node_0", "node_2"), Some(2));
        
        // Monday to Saturday: 2 steps backward (shorter than 5 forward)
        assert_eq!(days.get_distance("node_0", "node_5"), Some(2));
    }
    
    #[test]
    fn test_k_jump_wrap() {
        let seasons = Topology::new_cyclic(vec![
            "Spring", "Summer", "Fall", "Winter"
        ].iter().map(|s| s.to_string()).collect());
        
        // Jump 3 seasons forward from Spring -> Winter
        let result = seasons.get_k_jump("node_0", 3);
        assert!(result.is_some());
        let target_label = seasons.get_node_by_id(&result.unwrap()).unwrap().label;
        assert_eq!(target_label, "Winter");
        
        // Jump backward should work too
        let result = seasons.get_k_jump("node_0", -1);
        assert!(result.is_some());
        let target_label = seasons.get_node_by_id(&result.unwrap()).unwrap().label;
        assert_eq!(target_label, "Winter");
    }
}
```

## Applications and Use Cases

### Temporal Learning

- **Calendar systems**: Days, months, years
- **Scheduling**: Recurring events and cycles
- **Biological rhythms**: Circadian patterns
- **Historical cycles**: Economic, political patterns

### Spatial Reasoning

- **Navigation**: Compass directions
- **Rotation**: Clock faces, gears
- **Periodic structures**: Crystal lattices
- **Circular arrangements**: Seating charts

### Cultural Knowledge

- **Holidays**: Annual celebrations
- **Ceremonies**: Life cycle events
- **Traditions**: Seasonal customs
- **Rituals**: Religious observances

## Performance Considerations

### Efficient Modular Arithmetic

```rust
// Fast modular operations for cyclic navigation
pub fn fast_modulo(value: i32, modulus: usize) -> usize {
    let m = modulus as i32;
    ((value % m + m) % m) as usize
}

// Optimized distance calculation
pub fn cyclic_distance_optimized(from: usize, to: usize, n: usize) -> usize {
    let forward = (to + n - from) % n;
    let backward = (from + n - to) % n;
    forward.min(backward)
}
```

### Lookup Tables for Common Cycles

```rust
pub struct CyclicLookupTable {
    distances: Vec<Vec<usize>>,
    size: usize,
}

impl CyclicLookupTable {
    pub fn new(size: usize) -> Self {
        let mut distances = vec![vec![0; size]; size];
        
        for i in 0..size {
            for j in 0..size {
                distances[i][j] = cyclic_distance_optimized(i, j, size);
            }
        }
        
        Self { distances, size }
    }
    
    pub fn get_distance(&self, from: usize, to: usize) -> usize {
        self.distances[from][to]
    }
}
```

## Best Practices

1. **Clarify reference points**: Establish clear anchor points for orientation
2. **Handle wrap-around explicitly**: Make boundary crossings clear to learners
3. **Consider cultural context**: Different cultures may have different starting points
4. **Use appropriate visualizations**: Circular layouts work better than linear
5. **Account for directional preferences**: Some cycles have natural directions

## Next Steps

- Learn about [DAGs and Partial Orders](./dag.md) for hierarchical structures
- Explore [General Graphs](./graphs.md) for complex relationships
- See [Topology API Reference](../api/core.md#topology) for full implementation
- Review [Temporal Reasoning](../psychology/temporal.md) for time-based cognition
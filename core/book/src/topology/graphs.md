# General Graphs and Complex Networks

General graphs represent the most flexible topology in ABCDeez Core, capable of modeling complex relationships where connections can be bidirectional, weighted, and form arbitrary network structures. These are ideal for modeling social networks, concept maps, neural architectures, and other complex relational systems.

## Conceptual Foundation

### What Makes a Structure a General Graph?

A general graph has these properties:
1. **Arbitrary connections**: Any node can connect to any other node
2. **Bidirectional edges**: Edges can be undirected or have bidirectional relationships
3. **Weighted relationships**: Connections can have varying strengths
4. **Cycles allowed**: Nodes can form feedback loops and complex cycles
5. **Multiple edge types**: Different kinds of relationships between nodes
6. **Dynamic structure**: The graph can evolve and change over time

### Cognitive Significance

General graphs model complex cognitive and social phenomena:
- **Semantic networks**: How concepts relate to each other
- **Social learning**: Peer influences and knowledge sharing
- **Neural networks**: Brain-like connection patterns
- **Associative memory**: How memories link and trigger each other
- **Collaboration networks**: Team-based learning and problem solving

## Implementation in ABCDeez Core

### Creating General Graph Topologies

```rust
use abcdeez_core::core::topology::{Topology, TopologyType, EdgeType};

// Create concept map for science topics
let mut concept_map = Topology::new_graph();

// Add scientific concepts
concept_map.add_node("Physics", "Physical sciences");
concept_map.add_node("Chemistry", "Chemical sciences");
concept_map.add_node("Biology", "Life sciences");
concept_map.add_node("Mathematics", "Mathematical sciences");
concept_map.add_node("Statistics", "Statistical analysis");
concept_map.add_node("Research Methods", "Scientific methodology");

// Add weighted relationships
concept_map.add_weighted_edge("Mathematics", "Physics", 0.9, EdgeType::StrongPrerequisite);
concept_map.add_weighted_edge("Mathematics", "Chemistry", 0.7, EdgeType::Prerequisite);
concept_map.add_weighted_edge("Mathematics", "Statistics", 0.8, EdgeType::Prerequisite);
concept_map.add_weighted_edge("Physics", "Chemistry", 0.6, EdgeType::Related);
concept_map.add_weighted_edge("Chemistry", "Biology", 0.8, EdgeType::Related);
concept_map.add_weighted_edge("Statistics", "Research Methods", 0.9, EdgeType::Prerequisite);

// Add bidirectional relationships (mutual reinforcement)
concept_map.add_bidirectional_edge("Physics", "Chemistry", 0.4, EdgeType::MutualReinforcement);
concept_map.add_bidirectional_edge("Biology", "Chemistry", 0.7, EdgeType::MutualReinforcement);
```

### Internal Representation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub weight: f64,
    pub edge_type: EdgeType,
    pub bidirectional: bool,
    pub creation_time: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    Prerequisite,
    StrongPrerequisite,
    Related,
    Similar,
    Opposite,
    MutualReinforcement,
    Inhibitory,
    Causal,
    Temporal,
    Spatial,
    Custom(String),
}

impl Topology {
    pub fn new_graph() -> Self {
        Self {
            topology_type: TopologyType::Graph,
            nodes: Vec::new(),
            edges: Vec::new(),
            node_map: HashMap::new(),
            adjacency_matrix: None,
            edge_metadata: HashMap::new(),
        }
    }
    
    pub fn add_weighted_edge(&mut self, from: &str, to: &str, weight: f64, edge_type: EdgeType) -> Result<(), String> {
        let from_id = self.get_or_create_node(from)?;
        let to_id = self.get_or_create_node(to)?;
        
        self.edges.push(GraphEdge {
            from: from_id.clone(),
            to: to_id.clone(),
            weight,
            edge_type,
            bidirectional: false,
            creation_time: Utc::now(),
            last_updated: Utc::now(),
            metadata: HashMap::new(),
        });
        
        Ok(())
    }
    
    pub fn add_bidirectional_edge(&mut self, node1: &str, node2: &str, weight: f64, edge_type: EdgeType) -> Result<(), String> {
        let node1_id = self.get_or_create_node(node1)?;
        let node2_id = self.get_or_create_node(node2)?;
        
        // Add both directions
        self.edges.push(GraphEdge {
            from: node1_id.clone(),
            to: node2_id.clone(),
            weight,
            edge_type: edge_type.clone(),
            bidirectional: true,
            creation_time: Utc::now(),
            last_updated: Utc::now(),
            metadata: HashMap::new(),
        });
        
        self.edges.push(GraphEdge {
            from: node2_id,
            to: node1_id,
            weight,
            edge_type,
            bidirectional: true,
            creation_time: Utc::now(),
            last_updated: Utc::now(),
            metadata: HashMap::new(),
        });
        
        Ok(())
    }
}
```

## Operations on General Graphs

### Network Analysis Metrics

```rust
pub struct NetworkMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
    pub average_degree: f64,
    pub clustering_coefficient: f64,
    pub diameter: Option<usize>,
    pub average_path_length: f64,
}

impl Topology {
    pub fn calculate_network_metrics(&self) -> NetworkMetrics {
        let n = self.nodes.len();
        let m = self.edges.len();
        
        // Graph density
        let max_edges = n * (n - 1);
        let density = if max_edges > 0 { (2 * m) as f64 / max_edges as f64 } else { 0.0 };
        
        // Average degree
        let average_degree = if n > 0 { (2 * m) as f64 / n as f64 } else { 0.0 };
        
        // Clustering coefficient
        let clustering_coefficient = self.calculate_clustering_coefficient();
        
        // Path metrics
        let (diameter, avg_path_length) = self.calculate_path_metrics();
        
        NetworkMetrics {
            node_count: n,
            edge_count: m,
            density,
            average_degree,
            clustering_coefficient,
            diameter,
            average_path_length: avg_path_length,
        }
    }
    
    fn calculate_clustering_coefficient(&self) -> f64 {
        let mut total_clustering = 0.0;
        let mut valid_nodes = 0;
        
        for node in &self.nodes {
            let neighbors = self.get_neighbors(&node.id);
            let degree = neighbors.len();
            
            if degree < 2 {
                continue; // Need at least 2 neighbors for clustering
            }
            
            // Count triangles
            let mut triangles = 0;
            for i in 0..neighbors.len() {
                for j in i+1..neighbors.len() {
                    if self.has_edge(&neighbors[i], &neighbors[j]) {
                        triangles += 1;
                    }
                }
            }
            
            let possible_triangles = degree * (degree - 1) / 2;
            if possible_triangles > 0 {
                total_clustering += triangles as f64 / possible_triangles as f64;
                valid_nodes += 1;
            }
        }
        
        if valid_nodes > 0 {
            total_clustering / valid_nodes as f64
        } else {
            0.0
        }
    }
}
```

### Shortest Path Algorithms

```rust
pub struct PathResult {
    pub path: Vec<String>,
    pub total_weight: f64,
    pub edge_types: Vec<EdgeType>,
}

impl Topology {
    // Dijkstra's algorithm for weighted shortest paths
    pub fn find_shortest_path(&self, start: &str, end: &str) -> Option<PathResult> {
        let mut distances: HashMap<String, f64> = HashMap::new();
        let mut previous: HashMap<String, (String, EdgeType)> = HashMap::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: BinaryHeap<(OrderedFloat<f64>, String)> = BinaryHeap::new();
        
        // Initialize distances
        for node in &self.nodes {
            distances.insert(node.id.clone(), f64::INFINITY);
        }
        
        let start_id = self.get_node_id_by_label(start)?;
        let end_id = self.get_node_id_by_label(end)?;
        
        distances.insert(start_id.clone(), 0.0);
        queue.push((OrderedFloat(0.0), start_id.clone()));
        
        while let Some((OrderedFloat(current_dist), current_node)) = queue.pop() {
            if visited.contains(&current_node) {
                continue;
            }
            
            visited.insert(current_node.clone());
            
            if current_node == end_id {
                // Reconstruct path
                return Some(self.reconstruct_path(&previous, &start_id, &end_id));
            }
            
            // Check neighbors
            for edge in &self.edges {
                if edge.from == current_node && !visited.contains(&edge.to) {
                    let new_dist = current_dist + edge.weight;
                    
                    if new_dist < distances[&edge.to] {
                        distances.insert(edge.to.clone(), new_dist);
                        previous.insert(edge.to.clone(), (current_node.clone(), edge.edge_type.clone()));
                        queue.push((OrderedFloat(new_dist), edge.to.clone()));
                    }
                }
            }
        }
        
        None // No path found
    }
    
    fn reconstruct_path(&self, previous: &HashMap<String, (String, EdgeType)>, start: &str, end: &str) -> PathResult {
        let mut path = Vec::new();
        let mut edge_types = Vec::new();
        let mut current = end;
        let mut total_weight = 0.0;
        
        while let Some((prev_node, edge_type)) = previous.get(current) {
            let current_label = self.get_node_by_id(current).unwrap().label.clone();
            path.push(current_label);
            edge_types.push(edge_type.clone());
            
            // Add edge weight
            if let Some(edge) = self.edges.iter().find(|e| e.from == *prev_node && e.to == current) {
                total_weight += edge.weight;
            }
            
            current = prev_node;
        }
        
        path.push(self.get_node_by_id(start).unwrap().label.clone());
        path.reverse();
        edge_types.reverse();
        
        PathResult {
            path,
            total_weight,
            edge_types,
        }
    }
}
```

### Community Detection

```rust
pub struct Community {
    pub nodes: Vec<String>,
    pub internal_edges: usize,
    pub external_edges: usize,
    pub modularity_contribution: f64,
}

impl Topology {
    // Simple community detection using modularity optimization
    pub fn detect_communities(&self) -> Vec<Community> {
        let mut communities = Vec::new();
        let mut node_to_community: HashMap<String, usize> = HashMap::new();
        
        // Initialize each node as its own community
        for (i, node) in self.nodes.iter().enumerate() {
            node_to_community.insert(node.id.clone(), i);
            communities.push(Community {
                nodes: vec![node.label.clone()],
                internal_edges: 0,
                external_edges: 0,
                modularity_contribution: 0.0,
            });
        }
        
        // Iteratively merge communities to maximize modularity
        let mut improved = true;
        while improved {
            improved = false;
            let current_modularity = self.calculate_modularity(&communities);
            
            // Try merging each pair of communities
            for i in 0..communities.len() {
                for j in i+1..communities.len() {
                    // Temporarily merge communities i and j
                    let merged_community = self.merge_communities(&communities[i], &communities[j]);
                    
                    // Calculate new modularity
                    let mut test_communities = communities.clone();
                    test_communities[i] = merged_community;
                    test_communities.remove(j);
                    
                    let new_modularity = self.calculate_modularity(&test_communities);
                    
                    if new_modularity > current_modularity {
                        communities = test_communities;
                        improved = true;
                        break;
                    }
                }
                if improved { break; }
            }
        }
        
        // Update community metrics
        for community in &mut communities {
            self.update_community_metrics(community);
        }
        
        communities
    }
    
    fn calculate_modularity(&self, communities: &[Community]) -> f64 {
        let m = self.edges.len() as f64;
        let mut modularity = 0.0;
        
        for community in communities {
            let internal = community.internal_edges as f64;
            let total_degree = (community.internal_edges + community.external_edges) as f64;
            
            modularity += (internal / m) - (total_degree / (2.0 * m)).powi(2);
        }
        
        modularity
    }
    
    fn merge_communities(&self, comm1: &Community, comm2: &Community) -> Community {
        let mut nodes = comm1.nodes.clone();
        nodes.extend(comm2.nodes.clone());
        
        Community {
            nodes,
            internal_edges: 0, // Will be recalculated
            external_edges: 0, // Will be recalculated
            modularity_contribution: 0.0,
        }
    }
}
```

## Cognitive Phenomena in General Graphs

### Spreading Activation

```rust
pub struct ActivationSpread {
    pub activation_levels: HashMap<String, f64>,
    pub decay_rate: f64,
    pub spread_threshold: f64,
    pub max_iterations: usize,
}

impl ActivationSpread {
    pub fn new(decay_rate: f64, spread_threshold: f64) -> Self {
        Self {
            activation_levels: HashMap::new(),
            decay_rate,
            spread_threshold,
            max_iterations: 10,
        }
    }
    
    pub fn activate_concept(&mut self, concept: &str, initial_activation: f64) {
        self.activation_levels.insert(concept.to_string(), initial_activation);
    }
    
    pub fn spread_activation(&mut self, graph: &Topology) -> HashMap<String, f64> {
        for iteration in 0..self.max_iterations {
            let mut new_activations = self.activation_levels.clone();
            
            // Spread activation through the network
            for (source, &activation) in &self.activation_levels {
                if activation < self.spread_threshold {
                    continue;
                }
                
                // Find connected concepts
                for edge in &graph.edges {
                    let source_id = graph.get_node_id_by_label(source);
                    if let Some(src_id) = source_id {
                        if edge.from == src_id {
                            let target_label = graph.get_node_by_id(&edge.to).unwrap().label.clone();
                            
                            // Calculate spreading activation
                            let spread_amount = activation * edge.weight * (1.0 - self.decay_rate);
                            let current_target = new_activations.get(&target_label).unwrap_or(&0.0);
                            
                            new_activations.insert(target_label, current_target + spread_amount);
                        }
                    }
                }
            }
            
            // Apply decay to all activations
            for (_, activation) in new_activations.iter_mut() {
                *activation *= (1.0 - self.decay_rate);
            }
            
            self.activation_levels = new_activations;
            
            // Check for convergence
            let total_change: f64 = self.activation_levels.values().sum::<f64>();
            if total_change < 0.001 {
                break;
            }
        }
        
        self.activation_levels.clone()
    }
}
```

### Network-Based Learning

```rust
pub struct NetworkLearning {
    pub influence_strengths: HashMap<(String, String), f64>,
    pub learning_rates: HashMap<String, f64>,
    pub social_contagion_rate: f64,
}

impl NetworkLearning {
    pub fn simulate_peer_learning(
        &self,
        graph: &Topology,
        initial_knowledge: &HashMap<String, f64>,
        learning_steps: usize,
    ) -> HashMap<String, f64> {
        let mut knowledge = initial_knowledge.clone();
        
        for _step in 0..learning_steps {
            let mut knowledge_updates = HashMap::new();
            
            for node in &graph.nodes {
                let current_knowledge = knowledge.get(&node.label).unwrap_or(&0.0);
                let mut influence_sum = 0.0;
                let mut influence_count = 0.0;
                
                // Gather influence from connected peers
                for edge in &graph.edges {
                    if edge.to == node.id {
                        let peer_label = graph.get_node_by_id(&edge.from).unwrap().label.clone();
                        let peer_knowledge = knowledge.get(&peer_label).unwrap_or(&0.0);
                        
                        let influence_strength = edge.weight * self.social_contagion_rate;
                        influence_sum += peer_knowledge * influence_strength;
                        influence_count += influence_strength;
                    }
                }
                
                // Calculate knowledge update
                let average_peer_knowledge = if influence_count > 0.0 {
                    influence_sum / influence_count
                } else {
                    *current_knowledge
                };
                
                let learning_rate = self.learning_rates.get(&node.label).unwrap_or(&0.1);
                let knowledge_update = learning_rate * (average_peer_knowledge - current_knowledge);
                
                knowledge_updates.insert(node.label.clone(), current_knowledge + knowledge_update);
            }
            
            knowledge = knowledge_updates;
        }
        
        knowledge
    }
}
```

## Specialized Graph Types

### Concept Maps

```rust
pub struct ConceptMap {
    topology: Topology,
    concept_types: HashMap<String, ConceptType>,
    relationship_strengths: HashMap<(String, String), f64>,
}

#[derive(Debug, Clone)]
pub enum ConceptType {
    Fundamental,    // Core concepts
    Applied,        // Applied knowledge
    Abstract,       // Abstract principles
    Procedural,     // How-to knowledge
    Factual,        // Specific facts
}

impl ConceptMap {
    pub fn create_science_map() -> Self {
        let mut map = Self {
            topology: Topology::new_graph(),
            concept_types: HashMap::new(),
            relationship_strengths: HashMap::new(),
        };
        
        // Add fundamental concepts
        map.add_concept("Energy", ConceptType::Fundamental);
        map.add_concept("Matter", ConceptType::Fundamental);
        map.add_concept("Force", ConceptType::Fundamental);
        map.add_concept("Motion", ConceptType::Fundamental);
        
        // Add applied concepts
        map.add_concept("Heat Transfer", ConceptType::Applied);
        map.add_concept("Chemical Reactions", ConceptType::Applied);
        map.add_concept("Electricity", ConceptType::Applied);
        
        // Add relationships with strengths
        map.add_relationship("Energy", "Heat Transfer", 0.9, EdgeType::StrongPrerequisite);
        map.add_relationship("Matter", "Chemical Reactions", 0.8, EdgeType::Prerequisite);
        map.add_relationship("Force", "Motion", 0.9, EdgeType::Causal);
        map.add_relationship("Energy", "Electricity", 0.7, EdgeType::Related);
        
        map
    }
    
    fn add_concept(&mut self, name: &str, concept_type: ConceptType) {
        self.topology.add_node(name, &format!("{:?} concept", concept_type));
        self.concept_types.insert(name.to_string(), concept_type);
    }
    
    fn add_relationship(&mut self, from: &str, to: &str, strength: f64, edge_type: EdgeType) {
        self.topology.add_weighted_edge(from, to, strength, edge_type).unwrap();
        self.relationship_strengths.insert((from.to_string(), to.to_string()), strength);
    }
    
    pub fn identify_key_concepts(&self, min_connections: usize) -> Vec<String> {
        let mut key_concepts = Vec::new();
        
        for node in &self.topology.nodes {
            let connection_count = self.topology.get_degree(&node.id);
            if connection_count >= min_connections {
                key_concepts.push(node.label.clone());
            }
        }
        
        // Sort by connection count (descending)
        key_concepts.sort_by(|a, b| {
            let a_connections = self.topology.get_degree_by_label(a);
            let b_connections = self.topology.get_degree_by_label(b);
            b_connections.cmp(&a_connections)
        });
        
        key_concepts
    }
}
```

### Social Learning Networks

```rust
pub struct SocialLearningNetwork {
    topology: Topology,
    peer_influences: HashMap<(String, String), PeerInfluence>,
    learning_outcomes: HashMap<String, LearningOutcome>,
}

#[derive(Debug, Clone)]
pub struct PeerInfluence {
    pub influence_strength: f64,
    pub influence_type: InfluenceType,
    pub interaction_frequency: f64,
    pub shared_interests: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum InfluenceType {
    Positive,      // Helpful influence
    Negative,      // Hindering influence
    Neutral,       // No significant impact
    Competitive,   // Motivating through competition
    Collaborative, // Learning together
}

impl SocialLearningNetwork {
    pub fn simulate_collaborative_learning(
        &mut self,
        topic: &str,
        learning_duration: usize,
    ) -> HashMap<String, f64> {
        let mut learning_progress = HashMap::new();
        
        // Initialize all learners with base progress
        for node in &self.topology.nodes {
            learning_progress.insert(node.label.clone(), 0.1);
        }
        
        for round in 0..learning_duration {
            let mut progress_updates = HashMap::new();
            
            for learner_node in &self.topology.nodes {
                let current_progress = learning_progress[&learner_node.label];
                let mut total_influence = 0.0;
                let mut influence_count = 0;
                
                // Gather influences from connected peers
                for edge in &self.topology.edges {
                    if edge.to == learner_node.id {
                        let peer_label = self.topology.get_node_by_id(&edge.from).unwrap().label.clone();
                        let peer_progress = learning_progress[&peer_label];
                        
                        if let Some(influence) = self.peer_influences.get(&(peer_label.clone(), learner_node.label.clone())) {
                            match influence.influence_type {
                                InfluenceType::Positive | InfluenceType::Collaborative => {
                                    total_influence += peer_progress * influence.influence_strength * edge.weight;
                                    influence_count += 1;
                                }
                                InfluenceType::Competitive => {
                                    // Competition motivates if peer is ahead
                                    if peer_progress > current_progress {
                                        total_influence += 0.1 * influence.influence_strength;
                                    }
                                    influence_count += 1;
                                }
                                _ => {} // Neutral or negative influences
                            }
                        }
                    }
                }
                
                // Calculate progress update
                let base_learning = 0.02; // Individual learning rate
                let social_learning = if influence_count > 0 {
                    total_influence / influence_count as f64 * 0.05
                } else {
                    0.0
                };
                
                let new_progress = (current_progress + base_learning + social_learning).min(1.0);
                progress_updates.insert(learner_node.label.clone(), new_progress);
            }
            
            learning_progress = progress_updates;
        }
        
        learning_progress
    }
    
    pub fn identify_learning_leaders(&self) -> Vec<String> {
        let centrality_scores = self.calculate_betweenness_centrality();
        let mut leaders: Vec<_> = centrality_scores.into_iter().collect();
        leaders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        leaders.into_iter().take(3).map(|(node, _)| node).collect()
    }
    
    fn calculate_betweenness_centrality(&self) -> HashMap<String, f64> {
        let mut centrality = HashMap::new();
        
        // Initialize centrality scores
        for node in &self.topology.nodes {
            centrality.insert(node.label.clone(), 0.0);
        }
        
        // Calculate shortest paths between all pairs
        for source in &self.topology.nodes {
            for target in &self.topology.nodes {
                if source.id != target.id {
                    if let Some(path_result) = self.topology.find_shortest_path(&source.label, &target.label) {
                        // For each intermediate node in the path, increment its centrality
                        for (i, node_label) in path_result.path.iter().enumerate() {
                            if i > 0 && i < path_result.path.len() - 1 {
                                *centrality.get_mut(node_label).unwrap() += 1.0;
                            }
                        }
                    }
                }
            }
        }
        
        centrality
    }
}
```

## Advanced Graph Algorithms

### Graph Neural Network Integration

```rust
pub struct GraphNeuralNetwork {
    pub node_embeddings: HashMap<String, Vec<f64>>,
    pub edge_features: HashMap<(String, String), Vec<f64>>,
    pub embedding_dim: usize,
    pub learning_rate: f64,
}

impl GraphNeuralNetwork {
    pub fn new(embedding_dim: usize, learning_rate: f64) -> Self {
        Self {
            node_embeddings: HashMap::new(),
            edge_features: HashMap::new(),
            embedding_dim,
            learning_rate,
        }
    }
    
    pub fn initialize_embeddings(&mut self, graph: &Topology) {
        let mut rng = rand::thread_rng();
        
        for node in &graph.nodes {
            let embedding: Vec<f64> = (0..self.embedding_dim)
                .map(|_| rng.gen_range(-0.1..0.1))
                .collect();
            self.node_embeddings.insert(node.label.clone(), embedding);
        }
    }
    
    pub fn message_passing_update(&mut self, graph: &Topology) {
        let mut new_embeddings = HashMap::new();
        
        for node in &graph.nodes {
            let current_embedding = &self.node_embeddings[&node.label];
            let mut aggregated_messages = vec![0.0; self.embedding_dim];
            let mut neighbor_count = 0;
            
            // Aggregate messages from neighbors
            for edge in &graph.edges {
                if edge.to == node.id {
                    let neighbor_label = graph.get_node_by_id(&edge.from).unwrap().label.clone();
                    let neighbor_embedding = &self.node_embeddings[&neighbor_label];
                    
                    // Weighted aggregation
                    for i in 0..self.embedding_dim {
                        aggregated_messages[i] += neighbor_embedding[i] * edge.weight;
                    }
                    neighbor_count += 1;
                }
            }
            
            // Normalize by number of neighbors
            if neighbor_count > 0 {
                for i in 0..self.embedding_dim {
                    aggregated_messages[i] /= neighbor_count as f64;
                }
            }
            
            // Update embedding (simple linear combination)
            let mut updated_embedding = Vec::new();
            for i in 0..self.embedding_dim {
                let new_value = 0.5 * current_embedding[i] + 0.5 * aggregated_messages[i];
                updated_embedding.push(new_value);
            }
            
            new_embeddings.insert(node.label.clone(), updated_embedding);
        }
        
        self.node_embeddings = new_embeddings;
    }
    
    pub fn predict_link_probability(&self, node1: &str, node2: &str) -> f64 {
        if let (Some(emb1), Some(emb2)) = 
            (self.node_embeddings.get(node1), self.node_embeddings.get(node2)) {
            
            // Compute dot product similarity
            let dot_product: f64 = emb1.iter().zip(emb2.iter())
                .map(|(a, b)| a * b)
                .sum();
                
            // Apply sigmoid to get probability
            1.0 / (1.0 + (-dot_product).exp())
        } else {
            0.0
        }
    }
}
```

## Performance Optimization

### Efficient Graph Storage

```rust
pub struct CompressedGraph {
    pub adjacency_lists: Vec<Vec<(usize, f64)>>,  // (neighbor_index, weight)
    pub node_labels: Vec<String>,
    pub label_to_index: HashMap<String, usize>,
    pub edge_types: Vec<Vec<EdgeType>>,
}

impl CompressedGraph {
    pub fn from_topology(topology: &Topology) -> Self {
        let mut node_labels = Vec::new();
        let mut label_to_index = HashMap::new();
        
        // Build node mapping
        for (i, node) in topology.nodes.iter().enumerate() {
            node_labels.push(node.label.clone());
            label_to_index.insert(node.label.clone(), i);
        }
        
        let n = node_labels.len();
        let mut adjacency_lists = vec![Vec::new(); n];
        let mut edge_types = vec![Vec::new(); n];
        
        // Build adjacency lists
        for edge in &topology.edges {
            let from_label = topology.get_node_by_id(&edge.from).unwrap().label.clone();
            let to_label = topology.get_node_by_id(&edge.to).unwrap().label.clone();
            
            if let (Some(&from_idx), Some(&to_idx)) = 
                (label_to_index.get(&from_label), label_to_index.get(&to_label)) {
                adjacency_lists[from_idx].push((to_idx, edge.weight));
                edge_types[from_idx].push(edge.edge_type.clone());
            }
        }
        
        Self {
            adjacency_lists,
            node_labels,
            label_to_index,
            edge_types,
        }
    }
    
    pub fn get_neighbors(&self, node_label: &str) -> Option<&Vec<(usize, f64)>> {
        self.label_to_index.get(node_label)
            .and_then(|&idx| self.adjacency_lists.get(idx))
    }
    
    pub fn fast_shortest_path(&self, start: &str, end: &str) -> Option<(Vec<String>, f64)> {
        let start_idx = *self.label_to_index.get(start)?;
        let end_idx = *self.label_to_index.get(end)?;
        
        let mut distances = vec![f64::INFINITY; self.node_labels.len()];
        let mut previous = vec![None; self.node_labels.len()];
        let mut visited = vec![false; self.node_labels.len()];
        
        distances[start_idx] = 0.0;
        
        for _ in 0..self.node_labels.len() {
            // Find unvisited node with minimum distance
            let mut min_dist = f64::INFINITY;
            let mut min_idx = None;
            
            for i in 0..self.node_labels.len() {
                if !visited[i] && distances[i] < min_dist {
                    min_dist = distances[i];
                    min_idx = Some(i);
                }
            }
            
            let current_idx = min_idx?;
            if current_idx == end_idx {
                break;
            }
            
            visited[current_idx] = true;
            
            // Update distances to neighbors
            for &(neighbor_idx, weight) in &self.adjacency_lists[current_idx] {
                if !visited[neighbor_idx] {
                    let new_dist = distances[current_idx] + weight;
                    if new_dist < distances[neighbor_idx] {
                        distances[neighbor_idx] = new_dist;
                        previous[neighbor_idx] = Some(current_idx);
                    }
                }
            }
        }
        
        // Reconstruct path
        let mut path = Vec::new();
        let mut current = Some(end_idx);
        
        while let Some(idx) = current {
            path.push(self.node_labels[idx].clone());
            current = previous[idx];
        }
        
        path.reverse();
        
        if path.len() > 1 && path[0] == start {
            Some((path, distances[end_idx]))
        } else {
            None
        }
    }
}
```

## Testing General Graphs

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graph_creation_and_metrics() {
        let mut graph = Topology::new_graph();
        
        graph.add_node("A", "Node A");
        graph.add_node("B", "Node B");
        graph.add_node("C", "Node C");
        graph.add_node("D", "Node D");
        
        graph.add_weighted_edge("A", "B", 0.8, EdgeType::Related).unwrap();
        graph.add_weighted_edge("B", "C", 0.6, EdgeType::Related).unwrap();
        graph.add_weighted_edge("C", "D", 0.7, EdgeType::Related).unwrap();
        graph.add_weighted_edge("A", "D", 0.9, EdgeType::StrongPrerequisite).unwrap();
        
        let metrics = graph.calculate_network_metrics();
        assert_eq!(metrics.node_count, 4);
        assert_eq!(metrics.edge_count, 4);
        assert!(metrics.density > 0.0);
        assert!(metrics.average_degree > 0.0);
    }
    
    #[test]
    fn test_shortest_path() {
        let mut graph = Topology::new_graph();
        
        graph.add_node("Start", "Starting point");
        graph.add_node("Middle", "Middle point");
        graph.add_node("End", "End point");
        
        graph.add_weighted_edge("Start", "Middle", 2.0, EdgeType::Related).unwrap();
        graph.add_weighted_edge("Middle", "End", 3.0, EdgeType::Related).unwrap();
        graph.add_weighted_edge("Start", "End", 10.0, EdgeType::Related).unwrap();
        
        let path_result = graph.find_shortest_path("Start", "End").unwrap();
        assert_eq!(path_result.path, vec!["Start", "Middle", "End"]);
        assert_eq!(path_result.total_weight, 5.0);
    }
    
    #[test]
    fn test_spreading_activation() {
        let mut graph = Topology::new_graph();
        
        graph.add_node("Central", "Central concept");
        graph.add_node("Related1", "Related concept 1");
        graph.add_node("Related2", "Related concept 2");
        
        graph.add_weighted_edge("Central", "Related1", 0.8, EdgeType::Related).unwrap();
        graph.add_weighted_edge("Central", "Related2", 0.6, EdgeType::Related).unwrap();
        
        let mut activation = ActivationSpread::new(0.1, 0.05);
        activation.activate_concept("Central", 1.0);
        
        let final_activations = activation.spread_activation(&graph);
        assert!(final_activations["Related1"] > 0.0);
        assert!(final_activations["Related2"] > 0.0);
        assert!(final_activations["Related1"] > final_activations["Related2"]); // Stronger connection
    }
    
    #[test]
    fn test_community_detection() {
        let mut graph = Topology::new_graph();
        
        // Create two clusters
        graph.add_node("A1", "Cluster A node 1");
        graph.add_node("A2", "Cluster A node 2");
        graph.add_node("B1", "Cluster B node 1");
        graph.add_node("B2", "Cluster B node 2");
        
        // Strong intra-cluster connections
        graph.add_weighted_edge("A1", "A2", 0.9, EdgeType::Related).unwrap();
        graph.add_weighted_edge("B1", "B2", 0.9, EdgeType::Related).unwrap();
        
        // Weak inter-cluster connection
        graph.add_weighted_edge("A1", "B1", 0.2, EdgeType::Related).unwrap();
        
        let communities = graph.detect_communities();
        assert!(communities.len() >= 2);
    }
}
```

## Best Practices

1. **Choose appropriate representations**: Use adjacency lists for sparse graphs, matrices for dense ones
2. **Consider edge weights**: Model relationship strengths accurately
3. **Handle cycles carefully**: Implement cycle detection for algorithms that require acyclic graphs
4. **Optimize for your use case**: Different algorithms excel in different scenarios
5. **Monitor network evolution**: Track how graph structure changes over time
6. **Validate connectivity**: Ensure important nodes remain connected

## Applications

- **Knowledge Graphs**: Semantic relationships between concepts
- **Social Networks**: Peer learning and influence modeling
- **Neural Architecture**: Brain-inspired learning systems
- **Collaborative Learning**: Team-based educational platforms
- **Recommendation Systems**: Content and learning path suggestions
- **Research Networks**: Citation and collaboration analysis

## Next Steps

- Learn about [Bayesian Inference](../bayesian/intro.md) for uncertainty in network relationships
- Explore [Task Generation](../tasks/generation.md) for graph-based assessment
- Understand [Machine Learning Integration](../implementation/ml.md) for advanced graph analytics
- See [Performance Optimization](../implementation/performance.md) for large-scale graph processing
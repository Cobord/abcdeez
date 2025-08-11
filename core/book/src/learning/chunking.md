# Chunk Boundaries and Structural Knowledge

Chunk boundaries represent how learners organize sequential information into meaningful groups or "chunks." This fundamental cognitive process affects memory capacity, retrieval speed, and learning efficiency. ABCDeez Core's chunking module detects and models these organizational patterns to enhance personalized learning.

## Conceptual Foundation

### What are Chunks?

A chunk is a collection of elements that are stored and retrieved as a single unit in memory. Chunks serve several cognitive functions:

1. **Memory compression**: Reduce cognitive load by grouping related items
2. **Retrieval efficiency**: Access multiple items through a single retrieval cue
3. **Pattern recognition**: Identify familiar sequences and structures
4. **Learning acceleration**: Build on existing chunks to learn new material
5. **Expertise development**: Experts develop larger, more sophisticated chunks

### Theoretical Background

Chunking theory is grounded in cognitive science research:

- **Miller's 7±2 Rule**: Working memory can hold 7±2 chunks (not individual items)
- **Chase & Simon**: Chess experts chunk board positions into meaningful patterns
- **Ericsson & Kintsch**: Long-term working memory allows experts to use larger chunks
- **Gobet & Simon**: Template theory explains how chunks become organized templates

## Core Data Structures

### Chunk Boundary Representation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkBoundary {
    pub position: usize,           // Position in sequence (0-indexed)
    pub strength: f64,             // [0, 1] how strong the boundary is
    pub boundary_type: BoundaryType,
    pub detection_confidence: f64,  // [0, 1] confidence in this boundary
    pub creation_time: DateTime<Utc>,
    pub evidence_count: usize,      // Number of observations supporting this boundary
    pub last_reinforced: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundaryType {
    Natural,        // Inherent in the structure (e.g., word boundaries)
    Learned,        // Acquired through practice
    Semantic,       // Based on meaning or category
    Phonetic,       // Based on sound patterns
    Rhythmic,       // Based on temporal patterns
    Individual,     // Idiosyncratic to this learner
}

impl ChunkBoundary {
    pub fn new(position: usize, boundary_type: BoundaryType) -> Self {
        Self {
            position,
            strength: 0.1,
            boundary_type,
            detection_confidence: 0.5,
            creation_time: Utc::now(),
            evidence_count: 1,
            last_reinforced: Utc::now(),
        }
    }
    
    pub fn reinforce(&mut self, evidence_strength: f64) {
        self.evidence_count += 1;
        self.last_reinforced = Utc::now();
        
        // Strengthen boundary with evidence
        let learning_rate = 0.1 / (1.0 + (self.evidence_count as f64).sqrt());
        self.strength = (self.strength + learning_rate * evidence_strength).min(1.0);
        
        // Increase confidence
        self.detection_confidence = (self.detection_confidence + 0.05).min(1.0);
    }
    
    pub fn decay(&mut self, decay_rate: f64) {
        // Boundaries weaken without reinforcement
        self.strength *= 1.0 - decay_rate;
        if self.strength < 0.05 {
            self.strength = 0.0; // Remove very weak boundaries
        }
    }
    
    pub fn is_significant(&self) -> bool {
        self.strength > 0.3 && self.detection_confidence > 0.6
    }
}
```

### Chunk Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub chunk_id: String,
    pub elements: Vec<String>,          // Items in this chunk
    pub start_position: usize,          // Starting position in sequence
    pub end_position: usize,            // Ending position in sequence
    pub access_strength: f64,           // How easily this chunk is retrieved
    pub internal_coherence: f64,        // How well elements hang together
    pub chunk_type: ChunkType,
    pub formation_method: FormationMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Sequential,     // Elements in order (ABC)
    Categorical,    // Elements sharing category (colors, animals)
    Associative,    // Elements linked by association
    Hierarchical,   // Elements in tree structure
    Pattern,        // Elements forming a pattern (ABAB)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormationMethod {
    Automatic,      // Formed automatically during learning
    Deliberate,     // Consciously created by learner
    Instructed,     // Taught explicitly
    Discovered,     // Found through exploration
}

impl Chunk {
    pub fn size(&self) -> usize {
        self.elements.len()
    }
    
    pub fn contains_position(&self, position: usize) -> bool {
        position >= self.start_position && position <= self.end_position
    }
    
    pub fn overlap_with(&self, other: &Chunk) -> usize {
        let start = self.start_position.max(other.start_position);
        let end = self.end_position.min(other.end_position);
        if start <= end { end - start + 1 } else { 0 }
    }
    
    pub fn calculate_retrieval_time(&self) -> f64 {
        // Larger chunks take longer to retrieve but provide more information
        let base_time = 200.0; // Base retrieval time in ms
        let size_penalty = self.size() as f64 * 50.0;
        let strength_bonus = self.access_strength * 100.0;
        
        (base_time + size_penalty - strength_bonus).max(100.0)
    }
}
```

## Chunk Detection Algorithms

### Response Time-Based Detection

```rust
pub struct ResponseTimeChunkDetector {
    pub slowdown_threshold: f64,       // RT increase indicating boundary
    pub consistency_requirement: usize, // How many times to observe
    pub window_size: usize,            // Context window for comparison
}

impl ResponseTimeChunkDetector {
    pub fn detect_boundaries(
        &self,
        response_times: &[f64],
        positions: &[usize],
    ) -> Vec<ChunkBoundary> {
        let mut boundaries = Vec::new();
        
        if response_times.len() < self.window_size * 2 {
            return boundaries;
        }
        
        // Calculate local response time increases
        for i in self.window_size..(response_times.len() - self.window_size) {
            let before_window = &response_times[(i - self.window_size)..i];
            let after_window = &response_times[(i + 1)..(i + 1 + self.window_size)];
            
            let before_mean = before_window.iter().sum::<f64>() / before_window.len() as f64;
            let after_mean = after_window.iter().sum::<f64>() / after_window.len() as f64;
            let current_rt = response_times[i];
            
            // Look for response time spikes
            let spike_ratio = current_rt / before_mean;
            let context_ratio = current_rt / after_mean;
            
            if spike_ratio > self.slowdown_threshold && context_ratio > 1.2 {
                let strength = ((spike_ratio - 1.0) / 2.0).min(1.0);
                let boundary = ChunkBoundary {
                    position: positions[i],
                    strength,
                    boundary_type: BoundaryType::Learned,
                    detection_confidence: 0.7,
                    creation_time: Utc::now(),
                    evidence_count: 1,
                    last_reinforced: Utc::now(),
                };
                boundaries.push(boundary);
            }
        }
        
        // Merge nearby boundaries
        self.merge_nearby_boundaries(&mut boundaries, 2);
        boundaries
    }
    
    fn merge_nearby_boundaries(&self, boundaries: &mut Vec<ChunkBoundary>, max_distance: usize) {
        if boundaries.len() < 2 {
            return;
        }
        
        boundaries.sort_by_key(|b| b.position);
        let mut merged = Vec::new();
        let mut current = boundaries[0].clone();
        
        for boundary in boundaries.iter().skip(1) {
            if boundary.position - current.position <= max_distance {
                // Merge boundaries
                let total_strength = current.strength + boundary.strength;
                let weighted_position = (current.position as f64 * current.strength + 
                                       boundary.position as f64 * boundary.strength) / total_strength;
                
                current.position = weighted_position.round() as usize;
                current.strength = total_strength.min(1.0);
                current.evidence_count += boundary.evidence_count;
            } else {
                merged.push(current);
                current = boundary.clone();
            }
        }
        merged.push(current);
        
        *boundaries = merged;
    }
}
```

### Error Pattern-Based Detection

```rust
pub struct ErrorPatternDetector {
    pub error_spike_threshold: f64,
    pub context_window: usize,
    pub min_evidence: usize,
}

impl ErrorPatternDetector {
    pub fn detect_from_errors(
        &self,
        error_positions: &[usize],
        sequence_length: usize,
    ) -> Vec<ChunkBoundary> {
        // Count errors at each position
        let mut error_counts = vec![0; sequence_length];
        for &pos in error_positions {
            if pos < sequence_length {
                error_counts[pos] += 1;
            }
        }
        
        // Smooth error counts
        let smoothed = self.smooth_counts(&error_counts);
        
        // Find peaks in error rates
        let mut boundaries = Vec::new();
        for i in 1..(smoothed.len() - 1) {
            let local_max = smoothed[i] > smoothed[i - 1] && smoothed[i] > smoothed[i + 1];
            let above_threshold = smoothed[i] as f64 > self.error_spike_threshold;
            
            if local_max && above_threshold && smoothed[i] >= self.min_evidence {
                let strength = (smoothed[i] as f64 / 10.0).min(1.0); // Normalize
                boundaries.push(ChunkBoundary {
                    position: i,
                    strength,
                    boundary_type: BoundaryType::Learned,
                    detection_confidence: 0.6,
                    creation_time: Utc::now(),
                    evidence_count: smoothed[i],
                    last_reinforced: Utc::now(),
                });
            }
        }
        
        boundaries
    }
    
    fn smooth_counts(&self, counts: &[usize]) -> Vec<usize> {
        let mut smoothed = Vec::new();
        let half_window = self.context_window / 2;
        
        for i in 0..counts.len() {
            let start = i.saturating_sub(half_window);
            let end = (i + half_window).min(counts.len());
            let sum: usize = counts[start..end].iter().sum();
            let avg = sum / (end - start);
            smoothed.push(avg);
        }
        
        smoothed
    }
}
```

### Semantic Clustering Detection

```rust
pub struct SemanticChunkDetector {
    pub similarity_threshold: f64,
    pub min_cluster_size: usize,
    pub max_cluster_size: usize,
}

impl SemanticChunkDetector {
    pub fn detect_semantic_chunks(
        &self,
        items: &[String],
        similarity_matrix: &[Vec<f64>],
    ) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let n = items.len();
        
        // Use hierarchical clustering to find semantic groups
        let mut clusters = self.hierarchical_clustering(similarity_matrix);
        
        // Convert clusters to chunks
        for cluster in clusters {
            if cluster.len() >= self.min_cluster_size && cluster.len() <= self.max_cluster_size {
                let start_pos = cluster.iter().min().unwrap();
                let end_pos = cluster.iter().max().unwrap();
                
                let chunk_elements: Vec<String> = cluster
                    .iter()
                    .map(|&idx| items[idx].clone())
                    .collect();
                
                let coherence = self.calculate_cluster_coherence(&cluster, similarity_matrix);
                
                chunks.push(Chunk {
                    chunk_id: format!("semantic_{}_{}", start_pos, end_pos),
                    elements: chunk_elements,
                    start_position: *start_pos,
                    end_position: *end_pos,
                    access_strength: 0.5,
                    internal_coherence: coherence,
                    chunk_type: ChunkType::Categorical,
                    formation_method: FormationMethod::Discovered,
                });
            }
        }
        
        chunks
    }
    
    fn hierarchical_clustering(&self, similarity_matrix: &[Vec<f64>]) -> Vec<Vec<usize>> {
        let n = similarity_matrix.len();
        let mut clusters: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
        
        while clusters.len() > 1 {
            let mut best_merge = (0, 1);
            let mut best_similarity = 0.0;
            
            // Find most similar clusters
            for i in 0..clusters.len() {
                for j in (i + 1)..clusters.len() {
                    let sim = self.cluster_similarity(&clusters[i], &clusters[j], similarity_matrix);
                    if sim > best_similarity {
                        best_similarity = sim;
                        best_merge = (i, j);
                    }
                }
            }
            
            // If similarity too low, stop merging
            if best_similarity < self.similarity_threshold {
                break;
            }
            
            // Merge clusters
            let (i, j) = best_merge;
            let mut merged = clusters[i].clone();
            merged.extend(&clusters[j]);
            
            // Remove old clusters (higher index first)
            if i < j {
                clusters.remove(j);
                clusters.remove(i);
            } else {
                clusters.remove(i);
                clusters.remove(j);
            }
            
            clusters.push(merged);
        }
        
        clusters
    }
    
    fn cluster_similarity(
        &self,
        cluster1: &[usize],
        cluster2: &[usize],
        similarity_matrix: &[Vec<f64>],
    ) -> f64 {
        let mut total_similarity = 0.0;
        let mut count = 0;
        
        for &i in cluster1 {
            for &j in cluster2 {
                if i < similarity_matrix.len() && j < similarity_matrix[i].len() {
                    total_similarity += similarity_matrix[i][j];
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            total_similarity / count as f64
        } else {
            0.0
        }
    }
    
    fn calculate_cluster_coherence(
        &self,
        cluster: &[usize],
        similarity_matrix: &[Vec<f64>],
    ) -> f64 {
        if cluster.len() < 2 {
            return 1.0;
        }
        
        let mut total_similarity = 0.0;
        let mut count = 0;
        
        for i in 0..cluster.len() {
            for j in (i + 1)..cluster.len() {
                let idx1 = cluster[i];
                let idx2 = cluster[j];
                if idx1 < similarity_matrix.len() && idx2 < similarity_matrix[idx1].len() {
                    total_similarity += similarity_matrix[idx1][idx2];
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            total_similarity / count as f64
        } else {
            0.0
        }
    }
}
```

## Chunk Learning and Evolution

### Dynamic Chunk Formation

```rust
pub struct ChunkLearner {
    pub formation_threshold: f64,      // Minimum co-occurrence for chunk formation
    pub strengthening_rate: f64,       // How fast chunks strengthen
    pub decay_rate: f64,              // How fast unused chunks decay
    pub max_chunk_size: usize,         // Maximum elements per chunk
    pub competition_factor: f64,       // How chunks compete for elements
}

impl ChunkLearner {
    pub fn process_sequence(
        &mut self,
        sequence: &[String],
        existing_chunks: &mut Vec<Chunk>,
        response_times: Option<&[f64]>,
    ) {
        // Strengthen existing chunks that were activated
        self.strengthen_activated_chunks(sequence, existing_chunks);
        
        // Look for new chunk formation opportunities
        let new_chunks = self.discover_new_chunks(sequence, existing_chunks, response_times);
        
        // Add viable new chunks
        for chunk in new_chunks {
            if self.is_chunk_viable(&chunk, existing_chunks) {
                existing_chunks.push(chunk);
            }
        }
        
        // Apply decay to unused chunks
        self.apply_decay(existing_chunks);
        
        // Remove chunks that have become too weak
        existing_chunks.retain(|chunk| chunk.access_strength > 0.1);
        
        // Resolve chunk competition
        self.resolve_chunk_competition(existing_chunks);
    }
    
    fn strengthen_activated_chunks(&self, sequence: &[String], chunks: &mut [Chunk]) {
        for chunk in chunks.iter_mut() {
            if self.chunk_matches_sequence(chunk, sequence) {
                chunk.access_strength = (chunk.access_strength + self.strengthening_rate).min(1.0);
            }
        }
    }
    
    fn discover_new_chunks(
        &self,
        sequence: &[String],
        existing_chunks: &[Chunk],
        response_times: Option<&[f64]>,
    ) -> Vec<Chunk> {
        let mut new_chunks = Vec::new();
        
        // Try different subsequence lengths
        for length in 2..=self.max_chunk_size.min(sequence.len()) {
            for start in 0..=(sequence.len() - length) {
                let subseq = &sequence[start..start + length];
                
                // Check if this subsequence appears frequently together
                if self.is_frequent_pattern(subseq, sequence) {
                    // Check if there's evidence from response times
                    let rt_evidence = response_times
                        .map(|rts| self.has_response_time_evidence(start, start + length - 1, rts))
                        .unwrap_or(false);
                    
                    if rt_evidence || self.has_semantic_coherence(subseq) {
                        let chunk = Chunk {
                            chunk_id: format!("discovered_{}_{}", start, start + length - 1),
                            elements: subseq.to_vec(),
                            start_position: start,
                            end_position: start + length - 1,
                            access_strength: 0.3,
                            internal_coherence: self.calculate_coherence(subseq),
                            chunk_type: ChunkType::Sequential,
                            formation_method: FormationMethod::Discovered,
                        };
                        
                        new_chunks.push(chunk);
                    }
                }
            }
        }
        
        new_chunks
    }
    
    fn is_frequent_pattern(&self, pattern: &[String], sequence: &[String]) -> bool {
        // Count pattern occurrences
        let mut count = 0;
        for i in 0..=(sequence.len() - pattern.len()) {
            if sequence[i..i + pattern.len()] == *pattern {
                count += 1;
            }
        }
        
        count as f64 / sequence.len() as f64 > self.formation_threshold
    }
    
    fn has_response_time_evidence(
        &self,
        start: usize,
        end: usize,
        response_times: &[f64],
    ) -> bool {
        if end >= response_times.len() {
            return false;
        }
        
        // Look for faster response times within the potential chunk
        let chunk_mean = response_times[start..=end].iter().sum::<f64>() / (end - start + 1) as f64;
        let context_start = start.saturating_sub(2);
        let context_end = (end + 2).min(response_times.len() - 1);
        let context_mean = response_times[context_start..=context_end].iter().sum::<f64>() 
            / (context_end - context_start + 1) as f64;
        
        chunk_mean < context_mean * 0.9 // 10% faster indicates chunking
    }
    
    fn has_semantic_coherence(&self, elements: &[String]) -> bool {
        // Simplified semantic coherence check
        // In practice, would use semantic embeddings or categories
        
        // Check for alphabetical sequences
        if elements.len() >= 2 {
            let mut is_alphabetical = true;
            for i in 1..elements.len() {
                if elements[i-1].len() == 1 && elements[i].len() == 1 {
                    let prev_char = elements[i-1].chars().next().unwrap() as u8;
                    let curr_char = elements[i].chars().next().unwrap() as u8;
                    if curr_char != prev_char + 1 {
                        is_alphabetical = false;
                        break;
                    }
                }
            }
            if is_alphabetical {
                return true;
            }
        }
        
        // Check for repeated elements (patterns)
        let unique_elements: std::collections::HashSet<_> = elements.iter().collect();
        if unique_elements.len() < elements.len() {
            return true; // Repetition indicates structure
        }
        
        false
    }
    
    fn calculate_coherence(&self, elements: &[String]) -> f64 {
        // Simplified coherence calculation
        if self.has_semantic_coherence(elements) {
            0.8
        } else {
            0.4
        }
    }
    
    fn resolve_chunk_competition(&self, chunks: &mut Vec<Chunk>) {
        // When chunks overlap, the stronger chunk wins contested elements
        chunks.sort_by(|a, b| b.access_strength.partial_cmp(&a.access_strength).unwrap());
        
        let mut occupied_positions = std::collections::HashSet::new();
        let mut resolved_chunks = Vec::new();
        
        for chunk in chunks {
            let chunk_positions: Vec<_> = (chunk.start_position..=chunk.end_position).collect();
            let conflicts: usize = chunk_positions.iter()
                .filter(|&&pos| occupied_positions.contains(&pos))
                .count();
            
            // If less than half the chunk is contested, keep it
            if conflicts < chunk_positions.len() / 2 {
                for pos in chunk_positions {
                    occupied_positions.insert(pos);
                }
                resolved_chunks.push(chunk.clone());
            }
        }
        
        *chunks = resolved_chunks;
    }
    
    fn is_chunk_viable(&self, chunk: &Chunk, existing_chunks: &[Chunk]) -> bool {
        // Check minimum strength
        if chunk.access_strength < 0.2 {
            return false;
        }
        
        // Check for excessive overlap with existing chunks
        for existing in existing_chunks {
            let overlap = chunk.overlap_with(existing);
            if overlap > chunk.size() / 2 && existing.access_strength > chunk.access_strength {
                return false; // Too much overlap with stronger chunk
            }
        }
        
        true
    }
    
    fn chunk_matches_sequence(&self, chunk: &Chunk, sequence: &[String]) -> bool {
        if chunk.end_position >= sequence.len() {
            return false;
        }
        
        let seq_slice = &sequence[chunk.start_position..=chunk.end_position];
        seq_slice == chunk.elements
    }
    
    fn apply_decay(&self, chunks: &mut [Chunk]) {
        for chunk in chunks.iter_mut() {
            chunk.access_strength *= 1.0 - self.decay_rate;
        }
    }
}
```

## Chunk-Based Performance Prediction

### Response Time Modeling

```rust
pub struct ChunkPerformanceModel {
    pub base_retrieval_time: f64,      // Base time per element
    pub chunk_access_time: f64,        // Time to access a chunk
    pub chunk_advantage: f64,          // RT reduction from chunking
    pub boundary_penalty: f64,         // RT increase at boundaries
}

impl ChunkPerformanceModel {
    pub fn predict_sequence_time(
        &self,
        sequence: &[String],
        chunks: &[Chunk],
        chunk_boundaries: &[ChunkBoundary],
    ) -> Vec<f64> {
        let mut predicted_times = Vec::new();
        
        for (i, _item) in sequence.iter().enumerate() {
            let mut base_time = self.base_retrieval_time;
            
            // Check if position is in a chunk
            let in_chunk = chunks.iter().find(|chunk| chunk.contains_position(i));
            
            if let Some(chunk) = in_chunk {
                // Chunked elements are faster to retrieve
                base_time *= 1.0 - self.chunk_advantage * chunk.access_strength;
                
                // First element in chunk pays chunk access cost
                if i == chunk.start_position {
                    base_time += self.chunk_access_time;
                }
            }
            
            // Add boundary penalty
            for boundary in chunk_boundaries {
                if boundary.position == i && boundary.is_significant() {
                    base_time += self.boundary_penalty * boundary.strength;
                }
            }
            
            predicted_times.push(base_time.max(100.0)); // Minimum 100ms
        }
        
        predicted_times
    }
    
    pub fn predict_accuracy_effects(
        &self,
        position: usize,
        chunks: &[Chunk],
        boundaries: &[ChunkBoundary],
    ) -> f64 {
        let mut accuracy_modifier = 1.0;
        
        // Check if position benefits from chunking
        if let Some(chunk) = chunks.iter().find(|c| c.contains_position(position)) {
            // Stronger chunks provide more accuracy benefit
            accuracy_modifier *= 1.0 + 0.3 * chunk.access_strength * chunk.internal_coherence;
        }
        
        // Chunk boundaries may increase error rates
        for boundary in boundaries {
            if boundary.position == position && boundary.is_significant() {
                accuracy_modifier *= 1.0 - 0.2 * boundary.strength;
            }
        }
        
        accuracy_modifier.max(0.3).min(1.5) // Bound the effects
    }
}
```

### Individual Differences in Chunking

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingProfile {
    pub learner_id: String,
    pub preferred_chunk_size: usize,    // Typical chunk size for this learner
    pub chunking_sensitivity: f64,      // How readily chunks are formed
    pub boundary_detection_skill: f64,  // Ability to detect natural boundaries
    pub semantic_chunking_preference: f64, // Preference for meaning-based chunks
    pub individual_chunks: Vec<Chunk>,  // Learner-specific chunks
}

impl ChunkingProfile {
    pub fn new(learner_id: String) -> Self {
        Self {
            learner_id,
            preferred_chunk_size: 3, // Default to 3-4 items
            chunking_sensitivity: 0.5,
            boundary_detection_skill: 0.5,
            semantic_chunking_preference: 0.6,
            individual_chunks: Vec::new(),
        }
    }
    
    pub fn update_from_performance(
        &mut self,
        sequence: &[String],
        response_times: &[f64],
        accuracies: &[bool],
    ) {
        // Infer preferred chunk size from response time patterns
        self.infer_chunk_size_preference(response_times);
        
        // Update chunking sensitivity based on error patterns
        self.update_chunking_sensitivity(sequence, accuracies);
        
        // Assess boundary detection ability
        self.assess_boundary_detection(response_times);
    }
    
    fn infer_chunk_size_preference(&mut self, response_times: &[f64]) {
        // Look for rhythmic patterns in response times
        let mut best_period = 3;
        let mut best_regularity = 0.0;
        
        for period in 2..=7 {
            let regularity = self.calculate_periodicity(response_times, period);
            if regularity > best_regularity {
                best_regularity = regularity;
                best_period = period;
            }
        }
        
        if best_regularity > 0.3 {
            // Update preferred chunk size with some inertia
            self.preferred_chunk_size = 
                (self.preferred_chunk_size + best_period) / 2;
        }
    }
    
    fn calculate_periodicity(&self, response_times: &[f64], period: usize) -> f64 {
        if response_times.len() < period * 3 {
            return 0.0;
        }
        
        let mut correlations = Vec::new();
        
        for phase in 0..period {
            let phase_times: Vec<f64> = response_times.iter()
                .enumerate()
                .filter(|(i, _)| i % period == phase)
                .map(|(_, &rt)| rt)
                .collect();
                
            if phase_times.len() >= 3 {
                let mean = phase_times.iter().sum::<f64>() / phase_times.len() as f64;
                let variance = phase_times.iter()
                    .map(|&rt| (rt - mean).powi(2))
                    .sum::<f64>() / phase_times.len() as f64;
                
                // Low variance within phase indicates regularity
                correlations.push(1.0 / (1.0 + variance.sqrt()));
            }
        }
        
        if !correlations.is_empty() {
            correlations.iter().sum::<f64>() / correlations.len() as f64
        } else {
            0.0
        }
    }
    
    fn update_chunking_sensitivity(&mut self, sequence: &[String], accuracies: &[bool]) {
        // Higher sensitivity if performance improves in chunks
        let mut chunk_performance = Vec::new();
        
        for chunk_size in 2..=self.preferred_chunk_size + 2 {
            for start in 0..=(sequence.len() - chunk_size) {
                let chunk_accuracy = accuracies[start..start + chunk_size]
                    .iter()
                    .map(|&acc| if acc { 1.0 } else { 0.0 })
                    .sum::<f64>() / chunk_size as f64;
                
                chunk_performance.push(chunk_accuracy);
            }
        }
        
        let overall_accuracy = accuracies.iter()
            .map(|&acc| if acc { 1.0 } else { 0.0 })
            .sum::<f64>() / accuracies.len() as f64;
            
        let chunk_benefit = if !chunk_performance.is_empty() {
            chunk_performance.iter().sum::<f64>() / chunk_performance.len() as f64 - overall_accuracy
        } else {
            0.0
        };
        
        // Update sensitivity based on benefit
        self.chunking_sensitivity = (self.chunking_sensitivity + 0.1 * chunk_benefit).max(0.0).min(1.0);
    }
    
    fn assess_boundary_detection(&mut self, response_times: &[f64]) {
        // Look for response time spikes that indicate boundary detection
        let mut boundary_evidence = 0.0;
        let mean_rt = response_times.iter().sum::<f64>() / response_times.len() as f64;
        
        for (i, &rt) in response_times.iter().enumerate() {
            if i > 0 && i < response_times.len() - 1 {
                let prev_rt = response_times[i - 1];
                let next_rt = response_times[i + 1];
                
                // Look for spikes relative to context
                if rt > mean_rt * 1.5 && rt > prev_rt * 1.3 && rt > next_rt * 1.3 {
                    boundary_evidence += 0.1;
                }
            }
        }
        
        // Update boundary detection skill
        self.boundary_detection_skill = 
            (self.boundary_detection_skill + 0.1 * boundary_evidence).max(0.0).min(1.0);
    }
    
    pub fn generate_personalized_chunks(
        &self,
        sequence: &[String],
        universal_chunks: &[Chunk],
    ) -> Vec<Chunk> {
        let mut personalized = universal_chunks.to_vec();
        
        // Generate additional chunks based on individual preferences
        for size in (2..=self.preferred_chunk_size) {
            for start in 0..=(sequence.len() - size) {
                let subseq = &sequence[start..start + size];
                
                // Create chunk if it matches preferences
                if self.should_create_personal_chunk(subseq) {
                    let chunk = Chunk {
                        chunk_id: format!("personal_{}_{}_{}", self.learner_id, start, start + size - 1),
                        elements: subseq.to_vec(),
                        start_position: start,
                        end_position: start + size - 1,
                        access_strength: self.chunking_sensitivity * 0.8,
                        internal_coherence: self.calculate_personal_coherence(subseq),
                        chunk_type: ChunkType::Sequential,
                        formation_method: FormationMethod::Individual,
                    };
                    
                    personalized.push(chunk);
                }
            }
        }
        
        personalized
    }
    
    fn should_create_personal_chunk(&self, elements: &[String]) -> bool {
        // Simple heuristics for personal chunk creation
        elements.len() == self.preferred_chunk_size && 
        self.chunking_sensitivity > 0.6
    }
    
    fn calculate_personal_coherence(&self, elements: &[String]) -> f64 {
        // Base coherence adjusted by personal preferences
        let base_coherence = 0.5;
        let semantic_bonus = if self.has_personal_semantic_connection(elements) {
            self.semantic_chunking_preference * 0.3
        } else {
            0.0
        };
        
        (base_coherence + semantic_bonus).min(1.0)
    }
    
    fn has_personal_semantic_connection(&self, elements: &[String]) -> bool {
        // Check against individual chunks for semantic patterns
        for chunk in &self.individual_chunks {
            if chunk.elements.iter().any(|e| elements.contains(e)) {
                return true;
            }
        }
        false
    }
}
```

## Testing and Validation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chunk_boundary_detection() {
        let detector = ResponseTimeChunkDetector {
            slowdown_threshold: 1.5,
            consistency_requirement: 3,
            window_size: 3,
        };
        
        let response_times = vec![500.0, 520.0, 510.0, 750.0, 530.0, 540.0, 520.0, 800.0, 510.0];
        let positions = (0..response_times.len()).collect::<Vec<_>>();
        
        let boundaries = detector.detect_boundaries(&response_times, &positions);
        
        // Should detect boundaries at positions with RT spikes
        assert!(!boundaries.is_empty());
        assert!(boundaries.iter().any(|b| b.position == 3)); // RT spike at 750ms
    }
    
    #[test]
    fn test_chunk_formation() {
        let mut learner = ChunkLearner {
            formation_threshold: 0.3,
            strengthening_rate: 0.1,
            decay_rate: 0.05,
            max_chunk_size: 4,
            competition_factor: 0.5,
        };
        
        let sequence = vec!["A", "B", "C", "D", "E", "F"]
            .into_iter().map(String::from).collect::<Vec<_>>();
        let mut chunks = Vec::new();
        
        // Process sequence multiple times to allow chunk formation
        for _ in 0..5 {
            learner.process_sequence(&sequence, &mut chunks, None);
        }
        
        // Should form some chunks
        assert!(!chunks.is_empty());
        
        // Chunks should have reasonable properties
        for chunk in &chunks {
            assert!(chunk.size() >= 2);
            assert!(chunk.size() <= learner.max_chunk_size);
            assert!(chunk.access_strength > 0.0);
        }
    }
    
    #[test]
    fn test_chunk_performance_prediction() {
        let model = ChunkPerformanceModel {
            base_retrieval_time: 500.0,
            chunk_access_time: 100.0,
            chunk_advantage: 0.3,
            boundary_penalty: 200.0,
        };
        
        let sequence = vec!["A", "B", "C", "D"];
        let chunk = Chunk {
            chunk_id: "test".to_string(),
            elements: vec!["B".to_string(), "C".to_string()],
            start_position: 1,
            end_position: 2,
            access_strength: 0.8,
            internal_coherence: 0.9,
            chunk_type: ChunkType::Sequential,
            formation_method: FormationMethod::Discovered,
        };
        
        let boundary = ChunkBoundary {
            position: 0,
            strength: 0.7,
            boundary_type: BoundaryType::Natural,
            detection_confidence: 0.8,
            creation_time: Utc::now(),
            evidence_count: 5,
            last_reinforced: Utc::now(),
        };
        
        let predicted_times = model.predict_sequence_time(
            &sequence,
            &[chunk],
            &[boundary],
        );
        
        // Should predict different times based on chunking and boundaries
        assert_eq!(predicted_times.len(), 4);
        assert!(predicted_times[0] > predicted_times[1]); // Boundary penalty at position 0
        assert!(predicted_times[2] < model.base_retrieval_time); // Chunked element faster
    }
    
    #[test]
    fn test_chunking_profile_update() {
        let mut profile = ChunkingProfile::new("test_learner".to_string());
        
        let sequence = vec!["A", "B", "C", "D", "E", "F"]
            .into_iter().map(String::from).collect::<Vec<_>>();
        let response_times = vec![500.0, 520.0, 510.0, 500.0, 530.0, 520.0];
        let accuracies = vec![true, true, false, true, true, true];
        
        let initial_sensitivity = profile.chunking_sensitivity;
        
        profile.update_from_performance(&sequence, &response_times, &accuracies);
        
        // Profile should adapt based on performance
        assert!(profile.preferred_chunk_size >= 2);
        assert!(profile.preferred_chunk_size <= 7);
        assert!(profile.chunking_sensitivity >= 0.0);
        assert!(profile.chunking_sensitivity <= 1.0);
    }
    
    #[test]
    fn test_semantic_chunk_detection() {
        let detector = SemanticChunkDetector {
            similarity_threshold: 0.6,
            min_cluster_size: 2,
            max_cluster_size: 4,
        };
        
        let items = vec!["cat", "dog", "bird", "red", "blue", "green"];
        
        // Mock similarity matrix (animals similar, colors similar)
        let similarity_matrix = vec![
            vec![1.0, 0.8, 0.7, 0.1, 0.1, 0.1], // cat
            vec![0.8, 1.0, 0.6, 0.1, 0.1, 0.1], // dog  
            vec![0.7, 0.6, 1.0, 0.1, 0.1, 0.1], // bird
            vec![0.1, 0.1, 0.1, 1.0, 0.9, 0.8], // red
            vec![0.1, 0.1, 0.1, 0.9, 1.0, 0.7], // blue
            vec![0.1, 0.1, 0.1, 0.8, 0.7, 1.0], // green
        ];
        
        let chunks = detector.detect_semantic_chunks(&items, &similarity_matrix);
        
        // Should detect animal and color chunks
        assert!(chunks.len() >= 1);
        
        for chunk in &chunks {
            assert!(chunk.chunk_type == ChunkType::Categorical);
            assert!(chunk.internal_coherence > 0.5);
        }
    }
}
```

## Best Practices

1. **Multi-modal detection**: Use response times, errors, and semantic similarity together
2. **Individual adaptation**: Account for personal chunking preferences and abilities
3. **Dynamic updates**: Allow chunks to strengthen, weaken, and evolve with experience
4. **Boundary significance**: Only act on boundaries with sufficient evidence
5. **Chunk competition**: Resolve conflicts between overlapping chunks appropriately
6. **Validation**: Test chunk-based predictions against actual performance data

## Common Pitfalls

- **Over-chunking**: Creating too many small chunks that don't provide benefit
- **Static boundaries**: Failing to adapt chunk boundaries as learning progresses
- **Ignoring individual differences**: Using one-size-fits-all chunking strategies
- **Weak evidence**: Acting on chunk boundaries with insufficient supporting data
- **Competition neglect**: Allowing overlapping chunks to create confusion

## Applications

- **Sequence learning**: Optimize presentation of ordered material
- **Memory training**: Help learners develop effective chunking strategies
- **Difficulty prediction**: Use chunk boundaries to predict where errors will occur
- **Personalized pacing**: Adjust timing based on individual chunking patterns
- **Educational design**: Structure content to align with natural chunk boundaries

## Next Steps

- Learn about [Bayesian Models](../bayesian/intro.md) for uncertainty in chunk detection
- Explore [Memory Dynamics](./memory.md) for chunk-based memory modeling
- Understand [Response Time Analysis](../statistics/response_times.md) for chunking evidence
- See [Individual Differences](../psychology/individual_differences.md) for chunking profiles
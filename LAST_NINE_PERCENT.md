# Last 9%: Feasible Enhancements for Graph-Coded Learning Prototype

## Overview
This document outlines the plan for implementing the remaining feasible features to achieve near-complete coverage of the paper specifications. Focus areas include domain-specific implementations (especially music), explicit boundary training, data export for collaborative analysis, and advanced navigation tasks.

## Priority 1: Domain-Specific Implementations

### 1.1 Music Theory Domain
Music has rich graph structure perfect for this system:

#### Structures to Implement:
- **Chromatic Scale** (12-node cycle)
  - C → C# → D → D# → E → F → F# → G → G# → A → A# → B → C
  - Cyclic with enharmonic equivalents
  
- **Circle of Fifths** (12-node cycle with special properties)
  - Different traversal pattern than chromatic
  - Major/minor key relationships
  - Chord progression patterns
  
- **Scale Degrees** (7-node cycles within 12-node space)
  - Major: W-W-H-W-W-W-H pattern
  - Minor variants (natural, harmonic, melodic)
  - Modes as rotations
  
- **Chord Progressions** (DAG structure)
  - I → IV/V → I (basic)
  - ii-V-I progressions
  - Secondary dominants as shortcuts

#### Tasks:
- Interval identification (distance between notes)
- Scale navigation (next note in scale)
- Chord building (stacking thirds)
- Progression prediction (what chord follows)
- Modulation paths (key changes)
- Relative/parallel key relationships

### 1.2 Programming Syntax Trees
- **Control Flow Graphs** (if/else, loops)
- **Dependency Graphs** (import/module structure)
- **Inheritance Hierarchies** (OOP class trees)
- **State Machines** (program states)

### 1.3 Geographic Navigation
- **Metro/Subway Systems** (real-world graphs)
- **Road Networks** (weighted by distance/time)
- **Building Layouts** (floor plans)
- **Trail Systems** (hiking paths with elevation)

### 1.4 Menu Hierarchies
- **Application Menus** (File → Edit → View)
- **Settings Trees** (nested preferences)
- **Command Palettes** (VSCode-style)
- **Website Navigation** (sitemap structure)

## Priority 2: Explicit Boundary Training

### 2.1 Boundary Bridging Tasks
```rust
pub struct BoundaryBridgingTask {
    pub boundary_type: BoundaryType,
    pub span_size: usize,
    pub boundary_positions: Vec<usize>,
}

pub enum BoundaryType {
    ChunkBoundary,      // e.g., between F-G in alphabet
    OctaveBoundary,     // e.g., B→C in music  
    ModuleBoundary,     // e.g., between packages
    CategoryBoundary,   // e.g., vowel→consonant
}
```

### 2.2 Hierarchical Chunking
- **Multi-level boundaries** (letters → syllables → words → sentences)
- **Recursive structures** (nested parentheses, nested loops)
- **Fractal patterns** (self-similar at different scales)

## Priority 3: Data Export & Collaborative Analysis

### 3.1 Data Export Formats
```rust
pub struct LearnerDataExport {
    pub learner_id: String,
    pub sessions: Vec<SessionData>,
    pub performance_trajectories: Vec<PerformancePoint>,
    pub error_patterns: ErrorAnalysis,
    pub model_parameters: ModelSnapshot,
    pub metadata: ExportMetadata,
}

// Export formats
impl LearnerDataExport {
    pub fn to_json(&self) -> String;
    pub fn to_csv(&self) -> String;
    pub fn to_parquet(&self) -> Result<Vec<u8>>;
    pub fn to_sqlite(&self) -> Result<SqliteDb>;
}
```

### 3.2 Collaborative Analysis Tools
```rust
pub struct PopulationAnalyzer {
    pub learners: Vec<LearnerDataExport>,
}

impl PopulationAnalyzer {
    // Identify common difficult transitions
    pub fn find_population_bottlenecks(&self) -> Vec<(String, String, f64)>;
    
    // Cluster learners by strategy
    pub fn cluster_by_strategy(&self) -> HashMap<StrategyType, Vec<String>>;
    
    // Generate difficulty norms
    pub fn compute_item_difficulties(&self) -> HashMap<TaskType, DifficultyNorm>;
    
    // Find optimal task sequences
    pub fn mine_successful_paths(&self) -> Vec<TaskSequence>;
    
    // Compare learning curves
    pub fn align_trajectories(&self) -> AlignedTrajectories;
}
```

### 3.3 Privacy-Preserving Aggregation
- Differential privacy for sensitive data
- Federated learning approaches
- Anonymous performance benchmarks

## Priority 4: Advanced Navigation Tasks

### 4.1 Conditional Navigation
```rust
pub enum NavigationConstraint {
    AvoidNodes(Vec<String>),           // Don't pass through these
    RequireNodes(Vec<String>),         // Must pass through these
    MaxDistance(usize),                // Path length limit
    MinDistance(usize),                // Minimum path length
    RequireProperty(PropertyFilter),   // Only nodes with property
}

pub struct ConditionalNavigationTask {
    pub start: String,
    pub goal: String,
    pub constraints: Vec<NavigationConstraint>,
}
```

### 4.2 Multi-hop Reasoning
- "What's 3 steps from A that's also 2 steps from Z?"
- "Find all nodes equidistant from X and Y"
- "Navigate to any vowel in exactly 5 steps"

### 4.3 Backtracking Scenarios
- Dead-end detection
- Undo/redo navigation
- Alternative path finding

## Priority 5: Performance Prediction

### 5.1 Learning Curve Extrapolation
```rust
pub struct PerformancePredictor {
    pub model_type: PredictorType,
    pub parameters: PredictorParams,
}

pub enum PredictorType {
    PowerLaw,           // P(t) = a * t^b + c
    Exponential,        // P(t) = a * (1 - e^(-b*t))
    Logistic,          // P(t) = L / (1 + e^(-k*(t-t0)))
    BiExponential,     // P(t) = a*e^(-b*t) + c*e^(-d*t)
}

impl PerformancePredictor {
    pub fn fit(&mut self, data: &[PerformancePoint]);
    pub fn predict(&self, future_trials: usize) -> Vec<f64>;
    pub fn confidence_interval(&self, alpha: f64) -> (Vec<f64>, Vec<f64>);
}
```

### 5.2 Optimal Schedule Generation
- When to review each item
- Optimal task type sequences
- Personalized difficulty progression

## Priority 6: Real-time Adaptation

### 6.1 Struggle Detection
```rust
pub struct StruggleDetector {
    pub rt_threshold: f64,
    pub error_streak: usize,
    pub hint_threshold: f64,
}

impl StruggleDetector {
    pub fn should_provide_hint(&self, current_rt: f64) -> bool;
    pub fn should_reduce_difficulty(&self, recent_errors: usize) -> bool;
    pub fn should_switch_task_type(&self, performance: &[bool]) -> bool;
}
```

### 6.2 Progressive Hints
- Level 1: Confirm/deny current approach
- Level 2: Provide partial information
- Level 3: Show worked example
- Level 4: Direct instruction

## Implementation Plan

### Phase 1: Music Domain (Week 1)
- [ ] Implement chromatic scale topology
- [ ] Implement circle of fifths topology
- [ ] Create interval/scale/chord tasks
- [ ] Add music-specific semantic attributes
- [ ] Test transfer between scales/keys

### Phase 2: Data Export (Week 1)
- [ ] Design export schema
- [ ] Implement JSON/CSV exporters
- [ ] Create population analyzer
- [ ] Add privacy controls
- [ ] Build comparison visualizations

### Phase 3: Advanced Navigation (Week 2)
- [ ] Implement conditional navigation
- [ ] Add multi-hop reasoning
- [ ] Create backtracking tasks
- [ ] Test with multiple domains

### Phase 4: Boundary Training (Week 2)
- [ ] Explicit boundary tasks
- [ ] Hierarchical chunking
- [ ] Cross-domain boundaries
- [ ] Measure integration success

### Phase 5: Performance Prediction (Week 3)
- [ ] Implement curve fitting
- [ ] Add prediction models
- [ ] Generate optimal schedules
- [ ] Validate predictions

### Phase 6: Real-time Adaptation (Week 3)
- [ ] Struggle detection
- [ ] Progressive hint system
- [ ] Dynamic difficulty adjustment
- [ ] User experience testing

## Success Metrics

### Coverage Goals
- Achieve 99%+ implementation of paper specifications
- Support 5+ distinct domains with full task sets
- Enable cross-domain transfer experiments

### Performance Goals
- Export data for 100+ sessions efficiently
- Predict performance with R² > 0.8
- Detect struggle within 3 seconds
- Generate hints that improve success by 20%

### Research Goals
- Identify universal difficulty patterns
- Validate transfer learning hypotheses
- Discover optimal training sequences
- Quantify boundary crossing improvements

## Technical Considerations

### Database Schema
```sql
CREATE TABLE learners (
    id TEXT PRIMARY KEY,
    created_at TIMESTAMP,
    metadata JSON
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    learner_id TEXT REFERENCES learners(id),
    topology_type TEXT,
    start_time TIMESTAMP,
    end_time TIMESTAMP
);

CREATE TABLE responses (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id),
    task_type TEXT,
    correct BOOLEAN,
    response_time_ms INTEGER,
    task_data JSON,
    timestamp TIMESTAMP
);

CREATE TABLE model_snapshots (
    learner_id TEXT REFERENCES learners(id),
    timestamp TIMESTAMP,
    parameters JSON,
    PRIMARY KEY (learner_id, timestamp)
);
```

### API Endpoints (Future Web Version)
```
POST   /api/learners                 # Create learner
GET    /api/learners/:id/export      # Export data
POST   /api/sessions                 # Start session
POST   /api/sessions/:id/responses   # Submit response
GET    /api/analytics/population     # Population stats
GET    /api/analytics/compare        # Compare learners
POST   /api/predictions/performance  # Predict trajectory
```

## Conclusion

This plan addresses the remaining 9% of features in a systematic way, prioritizing:
1. Rich domain implementations (especially music)
2. Data export and collaborative analysis
3. Advanced navigation and boundary training
4. Performance prediction and real-time adaptation

The implementation maintains the prototype's focus on being a functional research tool while adding the sophistication needed for real-world experiments and collaborative research.
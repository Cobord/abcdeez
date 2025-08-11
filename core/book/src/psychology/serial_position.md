# Serial Position Effects

Serial position effects are among the most robust findings in memory research, describing how the position of an item in a sequence affects its memorability. ABCDeez Core accurately models these effects to create realistic learning simulations.

## The Classic U-Shaped Curve

When people learn ordered sequences, they show characteristic patterns:

```
Memory Strength
    ^
1.0 |●                           ●
    |  ●                       ●
0.8 |    ●                   ●
    |      ●               ●
0.6 |        ●           ●
    |          ●       ●
0.4 |            ● ● ●
    |
    +----------------------------->
      First    Middle        Last
           Serial Position
```

## Components of Serial Position Effects

### Primacy Effect

Items at the beginning of a sequence are remembered better due to:

1. **Increased Rehearsal**: More opportunities for rehearsal
2. **Reduced Proactive Interference**: No prior items to interfere
3. **Distinctive Encoding**: First items receive more attention

```rust
pub fn calculate_primacy_boost(position: usize, list_length: usize) -> f64 {
    let relative_pos = position as f64 / list_length as f64;
    
    // Exponential decay from start
    let primacy_strength = (-3.0 * relative_pos).exp();
    
    // Scale by primacy factor
    const PRIMACY_FACTOR: f64 = 0.3;
    primacy_strength * PRIMACY_FACTOR
}
```

### Recency Effect

Items at the end show enhanced recall due to:

1. **Working Memory**: Still in short-term/working memory
2. **Reduced Retroactive Interference**: No subsequent items
3. **Temporal Distinctiveness**: Recent items are temporally distinct

```rust
pub fn calculate_recency_boost(position: usize, list_length: usize) -> f64 {
    let from_end = list_length - position - 1;
    let relative_from_end = from_end as f64 / list_length as f64;
    
    // Exponential increase toward end
    let recency_strength = (-4.0 * relative_from_end).exp();
    
    // Scale by recency factor
    const RECENCY_FACTOR: f64 = 0.25;
    recency_strength * RECENCY_FACTOR
}
```

## Implementation in ABCDeez Core

### Memory Strength Calculation

```rust
impl LearnerModel {
    pub fn get_retention_probability(&self, node_id: &str) -> f64 {
        // Base memory strength from practice
        let base_strength = self.memory_strengths
            .get(node_id)
            .map(|m| m.strength)
            .unwrap_or(0.0);
        
        // Apply forgetting curve
        let retained = self.apply_forgetting_curve(base_strength, hours_elapsed);
        
        // Add serial position effects
        if let Some(embed) = self.node_embeddings.get(node_id) {
            let n_nodes = self.node_embeddings.len() as f64;
            let pos_norm = embed.position / (n_nodes - 1.0);
            
            // U-shaped boost using cosine
            let edge_emphasis = (PI * pos_norm).cos().abs();
            let boost = 1.0 + self.config.edge_emphasis * edge_emphasis;
            
            (retained * boost).min(1.0)
        } else {
            retained
        }
    }
}
```

### Empirical Validation

Our test suite validates serial position effects:

```rust
#[test]
fn test_serial_position_effect() {
    let items: Vec<String> = (0..20)
        .map(|i| format!("item_{}", i))
        .collect();
    
    let mut learner = LearnerModel::new("test".to_string(), 
                                       &Topology::new_linear(items));
    
    // Simulate learning
    for _ in 0..5 {
        for (i, item) in items.iter().enumerate() {
            learner.update_memory_strength(item, true);
        }
    }
    
    // Measure retention
    let retention: Vec<f64> = items.iter()
        .map(|item| learner.get_retention_probability(item))
        .collect();
    
    // Verify U-shaped curve
    let first_third = mean(&retention[0..7]);
    let middle_third = mean(&retention[7..14]);
    let last_third = mean(&retention[14..20]);
    
    assert!(first_third > middle_third, "Primacy effect");
    assert!(last_third > middle_third, "Recency effect");
    assert!(min(&retention[7..14]) < max(&retention[0..7]), "U-shape");
}
```

## Factors Affecting Serial Position

### 1. Presentation Rate

Faster presentation reduces primacy, maintains recency:

```rust
pub struct PresentationParameters {
    pub rate_ms: f64,
    pub primacy_weight: f64,
    pub recency_weight: f64,
}

impl PresentationParameters {
    pub fn from_rate(rate_ms: f64) -> Self {
        // Fast presentation (< 1000ms): Reduced primacy
        let primacy_weight = if rate_ms < 1000.0 {
            0.5 + 0.5 * (rate_ms / 1000.0)
        } else {
            1.0
        };
        
        // Recency relatively unaffected
        let recency_weight = 1.0;
        
        Self { rate_ms, primacy_weight, recency_weight }
    }
}
```

### 2. List Length

Longer lists show different patterns:

```rust
pub fn adjust_for_list_length(base_effect: f64, length: usize) -> f64 {
    match length {
        1..=5 => base_effect * 0.5,   // Minimal effect for short lists
        6..=10 => base_effect * 0.8,  // Moderate effect
        11..=20 => base_effect,        // Full effect
        _ => base_effect * 1.2,        // Enhanced for very long lists
    }
}
```

### 3. Delay and Interference

Delay eliminates recency, preserves primacy:

```rust
pub fn apply_delay_effects(
    primacy: f64, 
    recency: f64, 
    delay_seconds: f64
) -> (f64, f64) {
    // Recency decays rapidly (seconds to minutes)
    let recency_retained = recency * (-delay_seconds / 30.0).exp();
    
    // Primacy decays slowly (hours to days)
    let primacy_retained = primacy * (-delay_seconds / 3600.0).exp();
    
    (primacy_retained, recency_retained)
}
```

## Individual Differences

### Working Memory Capacity

Individuals with higher working memory show:
- Stronger primacy effects (better rehearsal)
- Extended recency effects (larger buffer)

```rust
pub struct IndividualDifferences {
    pub working_memory_span: usize,
    pub rehearsal_rate: f64,
    pub attention_consistency: f64,
}

impl IndividualDifferences {
    pub fn adjust_serial_position(&self, base_curve: &[f64]) -> Vec<f64> {
        base_curve.iter().enumerate().map(|(i, &base)| {
            let primacy_mult = 1.0 + (self.rehearsal_rate - 1.0) * 
                              (1.0 - i as f64 / base_curve.len() as f64);
            let recency_mult = if i >= base_curve.len() - self.working_memory_span {
                1.0 + 0.2 * self.attention_consistency
            } else {
                1.0
            };
            base * primacy_mult * recency_mult
        }).collect()
    }
}
```

## Applications in Learning

### 1. Optimal Item Ordering

Place difficult items in positions with natural advantages:

```rust
pub fn optimize_item_ordering(items: Vec<Item>) -> Vec<Item> {
    let mut sorted = items;
    sorted.sort_by_key(|item| item.difficulty);
    
    let n = sorted.len();
    let mut optimized = vec![None; n];
    
    // Place hardest items at edges
    let mut hard_idx = n - 1;
    let mut easy_idx = 0;
    
    for i in 0..n {
        let position_strength = calculate_position_strength(i, n);
        if position_strength > 0.7 {
            // Strong position: place hard item
            optimized[i] = Some(sorted[hard_idx].clone());
            hard_idx = hard_idx.saturating_sub(1);
        } else {
            // Weak position: place easy item
            optimized[i] = Some(sorted[easy_idx].clone());
            easy_idx += 1;
        }
    }
    
    optimized.into_iter().filter_map(|x| x).collect()
}
```

### 2. Review Scheduling

Focus on middle items that suffer from position:

```rust
pub fn prioritize_review(learner: &LearnerModel) -> Vec<String> {
    let mut items_with_position: Vec<_> = learner.node_embeddings
        .iter()
        .map(|(id, embed)| {
            let position_penalty = calculate_middle_penalty(embed.position);
            let memory = learner.get_retention_probability(id);
            (id.clone(), memory - position_penalty)
        })
        .collect();
    
    items_with_position.sort_by(|a, b| 
        a.1.partial_cmp(&b.1).unwrap()
    );
    
    items_with_position.into_iter()
        .take(5)  // Top 5 items needing review
        .map(|(id, _)| id)
        .collect()
}
```

### 3. Chunk Breaking

Use serial position to identify natural chunks:

```rust
pub fn identify_chunks(retention_curve: &[f64]) -> Vec<usize> {
    let mut boundaries = Vec::new();
    
    for i in 1..retention_curve.len()-1 {
        let local_min = retention_curve[i] < retention_curve[i-1] &&
                       retention_curve[i] < retention_curve[i+1];
        let threshold = mean(retention_curve) - 0.1 * std_dev(retention_curve);
        
        if local_min && retention_curve[i] < threshold {
            boundaries.push(i);
        }
    }
    
    boundaries
}
```

## Neurological Basis

Serial position effects reflect different memory systems:

1. **Primacy → Long-term Memory**
   - Hippocampal consolidation
   - Semantic encoding
   - Slower, more durable

2. **Recency → Working Memory**
   - Prefrontal cortex maintenance
   - Phonological loop
   - Fast but fragile

Our model captures these distinctions through different decay rates and update mechanisms.

## Validation Against Literature

Our implementation reproduces classic findings:

| Study | Finding | Our Model |
|-------|---------|-----------|
| Murdock (1962) | U-shaped free recall | ✓ Reproduced |
| Glanzer & Cunitz (1966) | Delay eliminates recency | ✓ Reproduced |
| Rundus (1971) | Rehearsal drives primacy | ✓ Modeled |
| Howard & Kahana (1999) | Temporal context effects | ✓ Implemented |

## Practical Implications

1. **Education**: Structure lessons with important content at beginning/end
2. **UI Design**: Place critical options at list edges
3. **Marketing**: Position key messages strategically
4. **Clinical Assessment**: Detect memory impairments through altered curves

## Next Steps

- Explore [Power Law of Practice](./power_law.md)
- Learn about [Spacing Effects](./spacing.md)
- Understand [Strategy Shifts](./strategies.md)
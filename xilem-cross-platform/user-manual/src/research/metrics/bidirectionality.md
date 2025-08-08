# Bidirectionality Index

The Bidirectionality Index is a core cognitive metric in the Adaptive Learning System that quantifies the asymmetry between forward and backward recall in sequential learning. This metric provides crucial insights into how learners organize and retrieve sequential information.

## Theoretical Background

### Cognitive Foundation

Sequential knowledge is not always symmetrical. Consider these examples:

- **Alphabet**: Most people can easily recite "A, B, C, D..." but struggle with "...D, C, B, A"
- **Days of Week**: "Monday, Tuesday, Wednesday" flows naturally, but "Wednesday, Tuesday, Monday" requires effort
- **Numbers**: Counting forward (1, 2, 3) is automatic, counting backward (3, 2, 1) is deliberate

This asymmetry reveals fundamental properties of how sequential information is encoded and retrieved in memory.

### Historical Context

The concept builds on several research traditions:

1. **Serial Position Effects** (Ebbinghaus, 1885): Memory strength varies by position
2. **Directional Associations** (Murdock, 1962): Forward associations stronger than backward
3. **Cognitive Maps** (Tolman, 1948): Mental representations have preferred directions
4. **Sequential Learning** (Restle, 1970): Pattern detection in ordered stimuli

## Mathematical Definition

### Basic Formula

The Bidirectionality Index (BI) is calculated as:

```
BI = (P_forward - P_backward) / (P_forward + P_backward)
```

Where:
- `P_forward` = Proportion correct on forward tasks
- `P_backward` = Proportion correct on backward tasks

### Range and Interpretation

- **Range**: -1.0 to +1.0
- **+1.0**: Perfect forward recall, no backward recall
- **0.0**: Equal performance in both directions
- **-1.0**: Perfect backward recall, no forward recall (rare)

### Statistical Properties

```python
def calculate_bidirectionality(responses):
    """
    Calculate bidirectionality index with confidence intervals
    
    Args:
        responses: List of task responses with direction labels
    
    Returns:
        dict: BI value, confidence interval, and significance
    """
    forward = [r for r in responses if r.direction == 'forward']
    backward = [r for r in responses if r.direction == 'backward']
    
    # Performance metrics
    p_f = np.mean([r.correct for r in forward])
    p_b = np.mean([r.correct for r in backward])
    
    # Bidirectionality index
    bi = (p_f - p_b) / (p_f + p_b) if (p_f + p_b) > 0 else 0
    
    # Standard error (using delta method)
    n_f, n_b = len(forward), len(backward)
    se_f = np.sqrt(p_f * (1 - p_f) / n_f)
    se_b = np.sqrt(p_b * (1 - p_b) / n_b)
    
    # Approximate confidence interval
    se_bi = np.sqrt((4 * p_b**2 * se_f**2 + 4 * p_f**2 * se_b**2) / 
                    (p_f + p_b)**4)
    
    ci_lower = bi - 1.96 * se_bi
    ci_upper = bi + 1.96 * se_bi
    
    # Significance test (two-proportion z-test)
    z_score = (p_f - p_b) / np.sqrt(se_f**2 + se_b**2)
    p_value = 2 * (1 - stats.norm.cdf(abs(z_score)))
    
    return {
        'bi': bi,
        'ci_95': (ci_lower, ci_upper),
        'p_value': p_value,
        'n_forward': n_f,
        'n_backward': n_b,
        'effect_size': abs(p_f - p_b)
    }
```

## Task Examples

### Forward Tasks
Tasks that follow the natural sequence order:

1. **Next Item**: "What comes after C?" → "D"
2. **Sequence Completion**: "A, B, ?" → "C"
3. **Order Judgment**: "Does B come before D?" → "Yes"
4. **Distance Estimation**: "How many letters between A and E?" → "3"

### Backward Tasks
Tasks that reverse the natural sequence order:

1. **Previous Item**: "What comes before C?" → "B"
2. **Reverse Completion**: "?, D, E" → "C"
3. **Reverse Order**: "Does D come before B?" → "No"
4. **Backward Distance**: "How many letters back from E to A?" → "4"

## Interpretation Guidelines

### Typical Values by Domain

| Domain | Typical BI | Interpretation |
|--------|------------|----------------|
| Alphabet | 0.15-0.35 | Moderate forward bias |
| Days of Week | 0.05-0.20 | Slight forward bias (cyclic) |
| Numbers | 0.10-0.25 | Forward counting advantage |
| Music Notes | 0.20-0.40 | Strong scale direction |

### Developmental Patterns

```python
# Age-related changes in bidirectionality
age_patterns = {
    '5-7 years': 0.40,   # Strong forward bias
    '8-10 years': 0.30,  # Reducing asymmetry
    '11-13 years': 0.20, # Developing flexibility
    '14-16 years': 0.15, # Near-adult levels
    'Adults': 0.10       # Flexible retrieval
}
```

### Learning Trajectories

The BI typically changes over learning:

1. **Early Learning** (BI ≈ 0.5): Strong forward bias, backward retrieval difficult
2. **Intermediate** (BI ≈ 0.3): Improving backward access
3. **Advanced** (BI ≈ 0.1): Flexible bidirectional retrieval
4. **Expert** (BI ≈ 0.0): Equal facility in both directions

## Research Applications

### 1. Learning Strategy Detection

Different learning strategies produce distinct BI signatures:

```python
def classify_strategy(bi_trajectory):
    """Identify learning strategy from BI changes"""
    
    if bi_trajectory.slope < -0.05:
        return "active_reversal"  # Practicing backward retrieval
    elif bi_trajectory.variance < 0.01:
        return "rote_forward"  # Memorizing forward only
    elif bi_trajectory.final < 0.1:
        return "flexible_encoding"  # Building bidirectional links
    else:
        return "standard_progression"
```

### 2. Individual Differences

BI correlates with cognitive abilities:

- **Working Memory**: Lower BI → Higher WM capacity (r = -0.35)
- **Cognitive Flexibility**: Lower BI → Better task switching (r = -0.42)
- **Age**: Lower BI → Older/more experienced (r = -0.28)
- **Expertise**: Lower BI → Domain expertise (r = -0.51)

### 3. Clinical Assessment

Abnormal BI values may indicate:

- **High BI (>0.5)**: Rigid sequential processing
- **Negative BI (<0)**: Unusual encoding strategies
- **Unstable BI**: Inconsistent retrieval processes
- **No change with practice**: Learning difficulties

## Experimental Manipulations

### Testing Bidirectionality

#### Balanced Design
```python
def generate_balanced_trials(n_trials=100):
    """Create equal forward and backward trials"""
    
    trials = []
    for i in range(n_trials // 2):
        trials.append(create_forward_trial())
        trials.append(create_backward_trial())
    
    random.shuffle(trials)
    return trials
```

#### Blocked vs. Interleaved
- **Blocked**: All forward, then all backward
- **Interleaved**: Random mixing
- **Adaptive**: Based on performance

### Interventions to Improve BI

1. **Reverse Practice**: Explicitly train backward retrieval
2. **Bidirectional Encoding**: Learn sequences in both directions
3. **Mental Rotation**: Visualize sequence reversals
4. **Chunk Reorganization**: Create flexible chunk boundaries

## Statistical Considerations

### Sample Size Requirements

For detecting meaningful BI differences:

```python
def calculate_sample_size(effect_size=0.2, power=0.8, alpha=0.05):
    """Calculate required N for BI comparisons"""
    
    from statsmodels.stats.power import TTestPower
    
    analysis = TTestPower()
    n = analysis.solve_power(
        effect_size=effect_size,
        power=power,
        alpha=alpha,
        ratio=1.0,  # Equal forward/backward trials
        alternative='two-sided'
    )
    
    return int(np.ceil(n))

# Typical requirements:
# Small effect (d=0.2): N=394 trials
# Medium effect (d=0.5): N=64 trials  
# Large effect (d=0.8): N=26 trials
```

### Reliability Analysis

```python
def assess_bi_reliability(session_data):
    """Calculate split-half reliability of BI"""
    
    # Split trials into halves
    first_half = session_data[:len(session_data)//2]
    second_half = session_data[len(session_data)//2:]
    
    # Calculate BI for each half
    bi_1 = calculate_bidirectionality(first_half)['bi']
    bi_2 = calculate_bidirectionality(second_half)['bi']
    
    # Correlation between halves
    reliability = np.corrcoef(bi_1, bi_2)[0, 1]
    
    # Spearman-Brown correction
    adjusted = (2 * reliability) / (1 + reliability)
    
    return adjusted
```

## Visualization Techniques

### BI Over Time
```python
def plot_bi_trajectory(session_data):
    """Visualize BI changes during learning"""
    
    window_size = 20  # trials
    bi_values = []
    
    for i in range(window_size, len(session_data)):
        window = session_data[i-window_size:i]
        bi = calculate_bidirectionality(window)['bi']
        bi_values.append(bi)
    
    plt.figure(figsize=(10, 6))
    plt.plot(bi_values, 'b-', linewidth=2)
    plt.axhline(y=0, color='r', linestyle='--', alpha=0.5)
    plt.xlabel('Trial Window')
    plt.ylabel('Bidirectionality Index')
    plt.title('Learning Trajectory: Bidirectional Access')
    plt.grid(True, alpha=0.3)
```

### Direction Comparison
```python
def plot_direction_comparison(forward_acc, backward_acc):
    """Compare forward vs backward performance"""
    
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))
    
    # Accuracy comparison
    ax1.bar(['Forward', 'Backward'], 
            [forward_acc, backward_acc],
            color=['green', 'orange'])
    ax1.set_ylabel('Accuracy')
    ax1.set_title('Directional Performance')
    
    # BI visualization
    bi = (forward_acc - backward_acc) / (forward_acc + backward_acc)
    ax2.barh(['BI'], [bi], color='blue')
    ax2.set_xlim(-1, 1)
    ax2.axvline(x=0, color='black', linestyle='-', alpha=0.3)
    ax2.set_xlabel('Bidirectionality Index')
    ax2.set_title(f'BI = {bi:.3f}')
```

## Best Practices

### For Researchers

1. **Ensure Adequate Sampling**: Minimum 20 trials per direction
2. **Control for Difficulty**: Match forward/backward task complexity
3. **Consider Order Effects**: Counterbalance presentation
4. **Monitor Fatigue**: BI can decline with tiredness
5. **Report Confidence Intervals**: Not just point estimates

### For Educators

1. **Target BI Reduction**: Aim for flexible retrieval (BI < 0.2)
2. **Practice Both Directions**: Don't neglect backward practice
3. **Use BI for Placement**: High BI suggests need for basics
4. **Track Progress**: Decreasing BI indicates mastery
5. **Individualize Training**: Some learners need more reversal practice

## References

- Kahana, M. J. (2002). Associative symmetry and memory theory. *Memory & Cognition*, 30(6), 823-840.
- Li, S. C., & Lewandowsky, S. (1995). Forward and backward recall: Different retrieval processes. *JEP: LMC*, 21(4), 837.
- Reber, P. J., & Kotovsky, K. (1997). Implicit learning in problem solving: The role of working memory capacity. *JEP: General*, 126(2), 178.
- Yang, L., & Lewandowsky, S. (2004). Knowledge partitioning in categorization: Constraints on exemplar models. *JEP: LMC*, 30(5), 1045.

---

*Next: [Symbolic Distance Effect](./distance.md) - Understanding the role of item separation in sequential learning*
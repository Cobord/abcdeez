# Hint Generation

The hint system provides adaptive support when learners struggle, balancing assistance with the need for productive struggle. Hints are generated dynamically based on error patterns and learning state.

## Hint Architecture

```rust
pub struct HintSystem {
    pub struggle_detector: StruggleDetector,
    pub hint_generator: HintGenerator,
    pub intervention_controller: InterventionController,
}

#[derive(Debug, Clone)]
pub enum HintLevel {
    Nudge,           // Minimal guidance
    Cue,             // Partial information
    Scaffold,        // Structured support
    Example,         // Worked example
    Solution,        // Complete answer
}

#[derive(Debug, Clone)]
pub struct Hint {
    pub level: HintLevel,
    pub content: String,
    pub cognitive_load: f64,
    pub expected_benefit: f64,
}
```

## Struggle Detection

### Error Pattern Analysis

```rust
pub struct StruggleDetector {
    pub error_threshold: usize,
    pub time_threshold: Duration,
    pub frustration_model: FrustrationModel,
}

impl StruggleDetector {
    pub fn detect_struggle(&self, history: &[ResponseData]) -> StruggleState {
        let recent = &history[history.len().saturating_sub(10)..];
        
        // Count consecutive errors
        let consecutive_errors = recent.iter()
            .rev()
            .take_while(|r| !r.correct)
            .count();
        
        // Check response time patterns
        let rt_pattern = self.analyze_response_times(recent);
        
        // Detect frustration indicators
        let frustration = self.frustration_model.compute(recent);
        
        StruggleState {
            severity: self.compute_severity(consecutive_errors, rt_pattern, frustration),
            struggle_type: self.classify_struggle(recent),
            confidence: self.compute_confidence(recent.len()),
        }
    }
    
    fn classify_struggle(&self, responses: &[ResponseData]) -> StruggleType {
        let error_patterns = self.extract_error_patterns(responses);
        
        if self.is_systematic_error(&error_patterns) {
            StruggleType::ConceptualMisunderstanding
        } else if self.is_random_guessing(&error_patterns) {
            StruggleType::Guessing
        } else if self.is_retrieval_failure(&error_patterns) {
            StruggleType::RetrievalFailure
        } else if self.is_procedural_error(&error_patterns) {
            StruggleType::ProceduralError
        } else {
            StruggleType::Unknown
        }
    }
    
    fn is_systematic_error(&self, patterns: &[ErrorPattern]) -> bool {
        // Check for consistent wrong answers
        let consistency = patterns.iter()
            .filter(|p| p.frequency > 0.7)
            .count();
        
        consistency > patterns.len() / 2
    }
}
```

## Hint Generation Strategies

### Adaptive Hint Selection

```rust
impl HintGenerator {
    pub fn generate_hint(&self, task: &Task, struggle: &StruggleState, learner: &LearnerModel) -> Hint {
        // Select appropriate hint level
        let level = self.select_hint_level(struggle, learner);
        
        // Generate hint content based on level
        let content = match level {
            HintLevel::Nudge => self.generate_nudge(task),
            HintLevel::Cue => self.generate_cue(task, learner),
            HintLevel::Scaffold => self.generate_scaffold(task, struggle),
            HintLevel::Example => self.generate_example(task),
            HintLevel::Solution => self.generate_solution(task),
        };
        
        Hint {
            level,
            content,
            cognitive_load: self.estimate_cognitive_load(&level),
            expected_benefit: self.predict_benefit(struggle, &level),
        }
    }
    
    fn select_hint_level(&self, struggle: &StruggleState, learner: &LearnerModel) -> HintLevel {
        match struggle.severity {
            Severity::Low => HintLevel::Nudge,
            Severity::Medium => {
                if learner.self_efficacy > 0.7 {
                    HintLevel::Cue
                } else {
                    HintLevel::Scaffold
                }
            }
            Severity::High => {
                if struggle.struggle_type == StruggleType::ConceptualMisunderstanding {
                    HintLevel::Example
                } else {
                    HintLevel::Scaffold
                }
            }
            Severity::Critical => HintLevel::Solution,
        }
    }
}
```

### Content Generation

```rust
impl HintGenerator {
    fn generate_nudge(&self, task: &Task) -> String {
        match &task.task_type {
            TaskType::Successor { item } => {
                format!("Think about what comes after {} in the sequence", item)
            }
            TaskType::KJump { start, k } => {
                format!("Count {} steps forward from {}", k, start)
            }
            _ => "Take your time and think about the pattern".to_string()
        }
    }
    
    fn generate_cue(&self, task: &Task, learner: &LearnerModel) -> String {
        // Provide partial information based on learner's knowledge
        let known_items = learner.get_well_known_items();
        
        match &task.task_type {
            TaskType::Successor { item } => {
                if let Some(anchor) = self.find_nearest_known(item, &known_items) {
                    format!("{} comes {} steps after {}", 
                           self.get_answer(task),
                           self.distance(item, &anchor),
                           anchor)
                } else {
                    self.generate_nudge(task)
                }
            }
            _ => self.generate_contextual_cue(task, learner)
        }
    }
    
    fn generate_scaffold(&self, task: &Task, struggle: &StruggleState) -> String {
        let steps = self.decompose_task(task);
        let mut scaffold = String::new();
        
        scaffold.push_str("Let's break this down:\n");
        
        for (i, step) in steps.iter().enumerate() {
            scaffold.push_str(&format!("{}. {}\n", i + 1, step));
            
            // Add more detail for problematic steps
            if self.is_problematic_step(step, struggle) {
                scaffold.push_str(&format!("   Hint: {}\n", self.step_hint(step)));
            }
        }
        
        scaffold
    }
    
    fn generate_example(&self, task: &Task) -> String {
        // Find similar but simpler task
        let similar = self.find_similar_task(task, 0.7);
        
        format!("Here's a similar example:\n\
                Question: {}\n\
                Answer: {}\n\
                Because: {}\n\n\
                Now try applying this to your task.",
                similar.prompt,
                similar.correct_answer,
                self.explain_solution(&similar))
    }
}
```

## Intervention Control

### Optimal Timing

```rust
pub struct InterventionController {
    pub min_struggle_time: Duration,
    pub max_struggle_time: Duration,
    pub productive_struggle_model: ProductiveStruggleModel,
}

impl InterventionController {
    pub fn should_intervene(&self, struggle: &StruggleState, elapsed: Duration) -> bool {
        if elapsed < self.min_struggle_time {
            // Allow productive struggle
            false
        } else if elapsed > self.max_struggle_time {
            // Prevent excessive frustration
            true
        } else {
            // Check if struggle is still productive
            !self.productive_struggle_model.is_productive(struggle, elapsed)
        }
    }
    
    pub fn select_intervention(&self, options: Vec<Intervention>) -> Intervention {
        // Multi-armed bandit approach
        let mut best_intervention = &options[0];
        let mut best_value = 0.0;
        
        for intervention in &options {
            let expected_value = self.compute_expected_value(intervention);
            let exploration_bonus = self.compute_ucb_bonus(intervention);
            
            let value = expected_value + exploration_bonus;
            
            if value > best_value {
                best_value = value;
                best_intervention = intervention;
            }
        }
        
        best_intervention.clone()
    }
}
```

## Error-Specific Hints

### Conceptual Errors

```rust
impl HintGenerator {
    fn generate_conceptual_hint(&self, error: &ConceptualError) -> Hint {
        match error.error_type {
            ConceptErrorType::Overgeneralization => {
                Hint {
                    level: HintLevel::Scaffold,
                    content: format!("Be careful: {} doesn't always mean {}. \
                                     Consider the specific context here.",
                                    error.generalized_rule,
                                    error.incorrect_application),
                    cognitive_load: 0.6,
                    expected_benefit: 0.8,
                }
            }
            ConceptErrorType::Misconception => {
                Hint {
                    level: HintLevel::Example,
                    content: self.generate_counterexample(&error.misconception),
                    cognitive_load: 0.7,
                    expected_benefit: 0.9,
                }
            }
            _ => self.generate_generic_conceptual_hint(error)
        }
    }
}
```

### Procedural Errors

```rust
impl HintGenerator {
    fn generate_procedural_hint(&self, error: &ProceduralError) -> Hint {
        let missing_step = self.identify_missing_step(&error.attempted_procedure);
        
        Hint {
            level: HintLevel::Scaffold,
            content: format!("You're on the right track, but you missed a step: {}",
                           missing_step),
            cognitive_load: 0.5,
            expected_benefit: 0.85,
        }
    }
}
```

## Adaptive Fading

Gradually reduce hint support:

```rust
pub struct HintFader {
    pub fading_schedule: FadingSchedule,
    pub mastery_threshold: f64,
}

impl HintFader {
    pub fn adjust_hint_level(&self, base_level: HintLevel, mastery: f64) -> HintLevel {
        if mastery > self.mastery_threshold {
            // Fade to less support
            match base_level {
                HintLevel::Solution => HintLevel::Example,
                HintLevel::Example => HintLevel::Scaffold,
                HintLevel::Scaffold => HintLevel::Cue,
                HintLevel::Cue => HintLevel::Nudge,
                HintLevel::Nudge => HintLevel::Nudge, // Minimum support
            }
        } else {
            base_level
        }
    }
    
    pub fn compute_fading_rate(&self, performance_history: &[f64]) -> f64 {
        let trend = self.compute_trend(performance_history);
        
        if trend > 0.1 {
            // Improving: fade faster
            1.5
        } else if trend < -0.1 {
            // Declining: fade slower or reverse
            0.5
        } else {
            // Stable: normal fading
            1.0
        }
    }
}
```

## Metacognitive Hints

Support learning about learning:

```rust
impl HintGenerator {
    fn generate_metacognitive_hint(&self, task: &Task, learner: &LearnerModel) -> Hint {
        let strategy = self.suggest_strategy(task, learner);
        
        Hint {
            level: HintLevel::Cue,
            content: format!("Strategy tip: {}\n\
                            This works well for this type of task because {}",
                           strategy.description,
                           strategy.rationale),
            cognitive_load: 0.4,
            expected_benefit: 0.7,
        }
    }
    
    fn suggest_strategy(&self, task: &Task, learner: &LearnerModel) -> Strategy {
        match task.difficulty {
            d if d < 0.3 => Strategy {
                description: "Try to retrieve quickly from memory",
                rationale: "This is well within your knowledge",
            },
            d if d < 0.7 => Strategy {
                description: "Use mental stepping through the sequence",
                rationale: "This requires careful navigation",
            },
            _ => Strategy {
                description: "Break it down into smaller parts",
                rationale: "Complex tasks benefit from decomposition",
            }
        }
    }
}
```

## Hint Effectiveness Tracking

```rust
pub struct HintEffectivenessTracker {
    pub hint_outcomes: Vec<HintOutcome>,
    pub effectiveness_model: EffectivenessModel,
}

impl HintEffectivenessTracker {
    pub fn track_outcome(&mut self, hint: &Hint, before: &ResponseData, after: &ResponseData) {
        let outcome = HintOutcome {
            hint: hint.clone(),
            improvement: self.compute_improvement(before, after),
            time_to_success: after.timestamp - before.timestamp,
            followup_needed: !after.correct,
        };
        
        self.hint_outcomes.push(outcome.clone());
        self.effectiveness_model.update(&outcome);
    }
    
    pub fn get_effectiveness(&self, hint_type: &HintLevel) -> f64 {
        let relevant_outcomes: Vec<_> = self.hint_outcomes.iter()
            .filter(|o| o.hint.level == *hint_type)
            .collect();
        
        if relevant_outcomes.is_empty() {
            return 0.5; // Default effectiveness
        }
        
        let success_rate = relevant_outcomes.iter()
            .filter(|o| o.improvement > 0.0)
            .count() as f64 / relevant_outcomes.len() as f64;
        
        success_rate
    }
}
```

## Personalized Hint Adaptation

```rust
impl HintSystem {
    pub fn personalize_hints(&mut self, learner: &LearnerModel) {
        // Adjust hint preferences based on learner profile
        self.hint_generator.preferences = HintPreferences {
            preferred_level: self.infer_preferred_level(learner),
            visual_hints: learner.learning_style == LearningStyle::Visual,
            example_based: learner.benefits_from_examples(),
            step_by_step: learner.prefers_scaffolding(),
        };
        
        // Adjust timing based on frustration tolerance
        self.intervention_controller.min_struggle_time = 
            Duration::seconds((30.0 * learner.frustration_tolerance) as i64);
        
        // Calibrate struggle detection
        self.struggle_detector.error_threshold = 
            (3.0 * (1.0 + learner.persistence)).round() as usize;
    }
    
    fn infer_preferred_level(&self, learner: &LearnerModel) -> HintLevel {
        let hint_history = learner.get_hint_history();
        
        // Find most effective hint level for this learner
        let mut best_level = HintLevel::Cue;
        let mut best_effectiveness = 0.0;
        
        for level in [HintLevel::Nudge, HintLevel::Cue, HintLevel::Scaffold, HintLevel::Example] {
            let effectiveness = self.compute_personal_effectiveness(&hint_history, level);
            
            if effectiveness > best_effectiveness {
                best_effectiveness = effectiveness;
                best_level = level;
            }
        }
        
        best_level
    }
}
```

## Summary

The hint system provides:
- **Struggle detection**: Identify when learners need help
- **Adaptive generation**: Create appropriate hints based on struggle type
- **Level selection**: Choose hint specificity based on need
- **Timing control**: Balance productive struggle with frustration
- **Error-specific hints**: Target conceptual vs procedural errors
- **Adaptive fading**: Gradually reduce support
- **Metacognitive hints**: Support learning strategies
- **Effectiveness tracking**: Monitor and improve hint quality
- **Personalization**: Adapt to individual preferences

This creates a supportive learning environment that provides help when needed while maintaining appropriate challenge.
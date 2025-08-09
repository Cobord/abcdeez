# R Statistical Analysis

The Adaptive Learning System provides comprehensive R analysis scripts for statistical modeling, hypothesis testing, and publication-ready visualizations. The R analysis pipeline is designed for rigorous cognitive science research following best practices for reproducible science.

## Installation and Setup

### Required R Packages

```r
# Core data manipulation and visualization
install.packages(c(
  "tidyverse",      # Data manipulation and visualization
  "lubridate",      # Date/time handling
  "jsonlite",       # JSON parsing
  "data.table"      # High-performance data operations
))

# Statistical modeling packages  
install.packages(c(
  "lme4",           # Linear mixed-effects models
  "lmerTest",       # p-values for mixed models
  "brms",           # Bayesian regression modeling
  "bayestestR",     # Bayesian analysis tools
  "performance",    # Model diagnostics
  "effectsize",     # Effect size calculations
  "emmeans",        # Estimated marginal means
  "multcomp"        # Multiple comparisons
))

# Analysis and visualization
install.packages(c(
  "ggpubr",         # Publication-ready plots
  "patchwork",      # Combining plots
  "viridis",        # Color scales
  "corrplot",       # Correlation matrices
  "psych",          # Psychological statistics
  "zoo",            # Time series analysis
  "pwr"             # Power analysis
))

# Load all required libraries
library(tidyverse)
library(lme4)
library(brms)
library(ggpubr)
```

### Project Structure

Create a standardized analysis directory structure:

```
analysis/
├── data/
│   ├── raw/              # Raw exported data
│   └── processed/        # Cleaned and preprocessed data
├── scripts/
│   ├── 01_preprocessing.R
│   ├── 02_hypothesis_tests.R
│   ├── 03_visualizations.R
│   └── 04_generate_report.R
├── functions/
│   ├── analysis_helpers.R
│   └── plotting_functions.R
└── output/
    ├── figures/
    ├── tables/
    └── reports/
```

## Data Import and Preprocessing

### Load Data from API Export

```r
# Import data exported from the Adaptive Learning System
load_abcdeez_data <- function(data_dir = "data/raw/") {
  
  # Load main response data
  responses_df <- read_csv(file.path(data_dir, "responses.csv"))
  
  # Load participant metadata  
  participants_df <- read_csv(file.path(data_dir, "participants.csv"))
  
  # Load session summaries
  sessions_df <- read_csv(file.path(data_dir, "sessions.csv"))
  
  # Parse JSON task parameters
  responses_df <- responses_df %>%
    mutate(
      task_params = map(task_json, ~ fromJSON(.x, flatten = TRUE))
    )
  
  # Merge datasets
  full_data <- responses_df %>%
    left_join(participants_df, by = "learner_id") %>%
    left_join(sessions_df, by = c("learner_id", "session_id"))
  
  return(full_data)
}

# Load data
raw_data <- load_abcdeez_data()
```

### Data Cleaning and Feature Engineering

```r
# Comprehensive data preprocessing pipeline
preprocess_data <- function(raw_data) {
  
  processed <- raw_data %>%
    # Basic cleaning
    filter(
      !is.na(rt_ms), 
      !is.na(correct),
      rt_ms >= 200,      # Remove implausibly fast responses
      rt_ms <= 10000     # Remove extremely slow responses
    ) %>%
    
    # Extract task-specific features
    mutate(
      # Parse task parameters from JSON
      item_a = case_when(
        task_type == "PairwiseOrder" ~ map_chr(task_params, ~ .x$a),
        TRUE ~ NA_character_
      ),
      item_b = case_when(
        task_type == "PairwiseOrder" ~ map_chr(task_params, ~ .x$b),
        TRUE ~ NA_character_
      ),
      k_value = case_when(
        task_type == "KJump" ~ map_dbl(task_params, ~ .x$k),
        TRUE ~ NA_real_
      ),
      
      # Calculate symbolic distances
      symbolic_distance = case_when(
        task_type == "PairwiseOrder" ~ abs(match(item_a, LETTERS) - match(item_b, LETTERS)),
        task_type == "KJump" ~ abs(k_value),
        task_type == "Segment" ~ map_dbl(task_params, ~ .x$count),
        TRUE ~ 1
      ),
      
      # Log-transform reaction times
      log_rt = log(rt_ms),
      
      # Create trial blocks for learning curves
      trial_block = cut(
        trial_number, 
        breaks = seq(0, max(trial_number), by = 20),
        labels = FALSE
      ),
      
      # Learning phases
      learning_phase = case_when(
        trial_number <= 100 ~ "early",
        trial_number <= 300 ~ "middle", 
        TRUE ~ "late"
      ),
      
      # Chunk boundary analysis (every 7 letters)
      crosses_boundary = case_when(
        task_type == "Segment" ~ {
          start_pos <- match(map_chr(task_params, ~ .x$start), LETTERS)
          segment_len <- map_dbl(task_params, ~ .x$count)
          (start_pos %/% 7) != ((start_pos + segment_len) %/% 7)
        },
        TRUE ~ FALSE
      ),
      
      # Center predictors for modeling
      symbolic_distance_c = scale(symbolic_distance, center = TRUE, scale = TRUE)[,1],
      trial_number_c = scale(trial_number, center = TRUE, scale = TRUE)[,1]
    ) %>%
    
    # Arrange by participant and trial
    arrange(learner_id, timestamp)
  
  return(processed)
}

# Apply preprocessing
data <- preprocess_data(raw_data)

# Save processed data
write_csv(data, "data/processed/processed_responses.csv")
```

## Statistical Analysis

### Learning Curve Analysis

```r
# Analyze learning trajectories with mixed-effects models
analyze_learning_curves <- function(data) {
  
  # Fit logistic growth curve model
  learning_model <- glmer(
    correct ~ I(trial_number^0.5) * group + 
      (1 + I(trial_number^0.5) | learner_id),
    data = data,
    family = binomial(),
    control = glmerControl(optimizer = "bobyqa")
  )
  
  # Extract individual learning rates
  learning_rates <- coef(learning_model)$learner_id %>%
    rownames_to_column("learner_id") %>%
    rename(intercept = `(Intercept)`, 
           learning_rate = `I(trial_number^0.5)`)
  
  # Group comparisons
  learning_anova <- anova(learning_model, type = "III")
  
  # Effect sizes
  learning_effects <- effectsize::eta_squared(learning_anova)
  
  return(list(
    model = learning_model,
    rates = learning_rates,
    anova = learning_anova,
    effects = learning_effects
  ))
}

# Run learning curve analysis
learning_results <- analyze_learning_curves(data)
```

### Symbolic Distance Effect Analysis

```r
# Test for representational shift (H1: Distance effect reduction)
test_symbolic_distance_effect <- function(data) {
  
  # Filter to pairwise comparison tasks
  pairwise_data <- data %>%
    filter(task_type == "PairwiseOrder", !is.na(symbolic_distance))
  
  # Hierarchical model with three-way interaction
  distance_model <- lmer(
    log_rt ~ symbolic_distance_c * group * trial_block + 
      (1 + symbolic_distance_c | learner_id),
    data = pairwise_data,
    REML = FALSE,
    control = lmerControl(optimizer = "bobyqa")
  )
  
  # Extract distance effect slopes over time
  distance_slopes <- pairwise_data %>%
    group_by(group, trial_block, learner_id) %>%
    summarise(
      n_trials = n(),
      .groups = "drop"
    ) %>%
    filter(n_trials >= 5) %>%  # Ensure sufficient data
    left_join(
      pairwise_data %>%
        nest(data = -c(group, trial_block, learner_id)) %>%
        mutate(
          model = map(data, ~ safely(lm)(log_rt ~ symbolic_distance_c, data = .x)),
          slope = map_dbl(model, ~ .x$result$coefficients[2] %||% NA_real_)
        ) %>%
        select(-data, -model),
      by = c("group", "trial_block", "learner_id")
    )
  
  # Test slope reduction over time
  slope_change <- distance_slopes %>%
    group_by(group, learner_id) %>%
    summarise(
      early_slope = mean(slope[trial_block <= 3], na.rm = TRUE),
      late_slope = mean(slope[trial_block >= 8], na.rm = TRUE),
      slope_reduction = early_slope - late_slope,
      .groups = "drop"
    )
  
  # Group comparison of slope reduction
  slope_test <- t.test(
    slope_reduction ~ group, 
    data = slope_change[slope_change$group %in% c("adaptive", "linear"),]
  )
  
  return(list(
    model = distance_model,
    slopes = distance_slopes,
    slope_change = slope_change,
    group_test = slope_test
  ))
}

# Run symbolic distance analysis
distance_results <- test_symbolic_distance_effect(data)
```

### Chunk Boundary Effect Analysis

```r
# Test for chunk boundary effects (H2: Seam smoothing)
analyze_chunk_boundaries <- function(data) {
  
  segment_data <- data %>%
    filter(task_type == "Segment")
  
  # Accuracy model: boundary crossing penalty
  boundary_accuracy_model <- glmer(
    correct ~ crosses_boundary * group * learning_phase + 
      (1 + crosses_boundary | learner_id),
    data = segment_data,
    family = binomial(),
    control = glmerControl(optimizer = "bobyqa")
  )
  
  # Reaction time model: boundary crossing cost
  boundary_rt_model <- lmer(
    log_rt ~ crosses_boundary * group * learning_phase + 
      (1 + crosses_boundary | learner_id),
    data = segment_data,
    REML = FALSE
  )
  
  # Calculate boundary penalties
  boundary_effects <- segment_data %>%
    group_by(group, learning_phase, crosses_boundary) %>%
    summarise(
      mean_accuracy = mean(correct, na.rm = TRUE),
      mean_rt = mean(rt_ms, na.rm = TRUE),
      se_accuracy = sd(correct, na.rm = TRUE) / sqrt(n()),
      se_rt = sd(rt_ms, na.rm = TRUE) / sqrt(n()),
      .groups = "drop"
    ) %>%
    pivot_wider(
      names_from = crosses_boundary,
      values_from = c(mean_accuracy, mean_rt, se_accuracy, se_rt),
      names_sep = "_"
    ) %>%
    mutate(
      accuracy_penalty = mean_accuracy_FALSE - mean_accuracy_TRUE,
      rt_penalty = mean_rt_TRUE - mean_rt_FALSE
    )
  
  return(list(
    accuracy_model = boundary_accuracy_model,
    rt_model = boundary_rt_model,
    boundary_effects = boundary_effects
  ))
}

# Run boundary analysis
boundary_results <- analyze_chunk_boundaries(data)
```

### Strategy Detection and Classification

```r
# Detect individual learning strategies
detect_learning_strategies <- function(data, window_size = 50) {
  
  # Focus on tasks with meaningful distance effects
  strategy_data <- data %>%
    filter(task_type %in% c("PairwiseOrder", "KJump")) %>%
    group_by(learner_id) %>%
    mutate(window_id = (row_number() - 1) %/% window_size) %>%
    ungroup()
  
  # Calculate RT-distance correlations within windows
  strategy_windows <- strategy_data %>%
    group_by(learner_id, window_id) %>%
    filter(n() >= 10) %>%  # Minimum trials per window
    summarise(
      rt_distance_corr = cor(rt_ms, symbolic_distance, use = "complete.obs"),
      mean_rt = mean(rt_ms, na.rm = TRUE),
      accuracy = mean(correct, na.rm = TRUE),
      trial_start = min(trial_number),
      trial_end = max(trial_number),
      n_trials = n(),
      .groups = "drop"
    ) %>%
    mutate(
      # Classify strategy based on RT-distance correlation
      strategy = case_when(
        rt_distance_corr >= 0.7 ~ "serial_scan",
        rt_distance_corr >= 0.5 ~ "mixed_serial", 
        rt_distance_corr >= 0.3 ~ "transitioning",
        rt_distance_corr >= 0.1 ~ "mixed_direct",
        rt_distance_corr > -0.1 ~ "direct_access",
        TRUE ~ "inverse_distance"  # Rare but theoretically possible
      )
    )
  
  # Strategy transitions over time
  strategy_transitions <- strategy_windows %>%
    arrange(learner_id, window_id) %>%
    group_by(learner_id) %>%
    mutate(
      previous_strategy = lag(strategy),
      strategy_change = strategy != previous_strategy
    ) %>%
    ungroup()
  
  # Final strategy classification per participant
  final_strategies <- strategy_windows %>%
    group_by(learner_id) %>%
    slice_tail(n = 3) %>%  # Last 3 windows
    count(strategy) %>%
    slice_max(n) %>%
    select(learner_id, final_strategy = strategy)
  
  return(list(
    windows = strategy_windows,
    transitions = strategy_transitions,
    final = final_strategies
  ))
}

# Run strategy analysis
strategy_results <- detect_learning_strategies(data)
```

## Bayesian Analysis

```r
# Bayesian hierarchical modeling with brms
run_bayesian_analysis <- function(data) {
  
  # Set up parallel processing
  options(mc.cores = parallel::detectCores())
  
  # Bayesian distance effect model
  bayes_distance_model <- brm(
    log_rt ~ symbolic_distance_c * group * trial_block + 
      (1 + symbolic_distance_c | learner_id),
    data = filter(data, task_type == "PairwiseOrder"),
    prior = c(
      prior(normal(0, 0.5), class = b),
      prior(exponential(1), class = sd),
      prior(exponential(1), class = sigma)
    ),
    iter = 4000,
    chains = 4,
    cores = 4,
    control = list(adapt_delta = 0.95)
  )
  
  # Posterior summaries
  posterior_summary <- posterior_summary(bayes_distance_model)
  
  # Hypothesis testing
  hypotheses <- hypothesis(
    bayes_distance_model,
    c("symbolic_distance_c:groupadaptive < symbolic_distance_c:grouplinear",
      "symbolic_distance_c:trial_block < 0"),
    class = "b"
  )
  
  # Bayes factors
  bayes_factors <- bayes_factor(bayes_distance_model)
  
  return(list(
    model = bayes_distance_model,
    summary = posterior_summary,
    hypotheses = hypotheses,
    bayes_factors = bayes_factors
  ))
}

# Run Bayesian analysis (computationally intensive)
# bayes_results <- run_bayesian_analysis(data)
```

## Visualization Functions

```r
# Create publication-ready learning curve plots
plot_learning_curves <- function(data, group_var = "group") {
  
  # Calculate learning trajectories with confidence intervals
  learning_curves <- data %>%
    group_by(!!sym(group_var), trial_block) %>%
    summarise(
      accuracy = mean(correct, na.rm = TRUE),
      se = sd(correct, na.rm = TRUE) / sqrt(n()),
      ci_lower = accuracy - 1.96 * se,
      ci_upper = accuracy + 1.96 * se,
      trial_midpoint = mean(trial_number, na.rm = TRUE),
      .groups = "drop"
    )
  
  # Create plot
  ggplot(learning_curves, aes(x = trial_midpoint, y = accuracy, 
                             color = !!sym(group_var), fill = !!sym(group_var))) +
    geom_ribbon(aes(ymin = ci_lower, ymax = ci_upper), alpha = 0.3, color = NA) +
    geom_line(size = 1.2) +
    geom_point(size = 2) +
    scale_color_viridis_d(end = 0.8) +
    scale_fill_viridis_d(end = 0.8) +
    labs(
      x = "Trial Number",
      y = "Accuracy",
      color = str_to_title(group_var),
      fill = str_to_title(group_var),
      title = "Learning Curves by Condition"
    ) +
    theme_pubr() +
    theme(legend.position = "bottom")
}

# Plot symbolic distance effects over time
plot_distance_effects <- function(data, phases = c("early", "middle", "late")) {
  
  # Calculate distance effects by phase
  distance_effects <- data %>%
    filter(task_type == "PairwiseOrder", learning_phase %in% phases) %>%
    group_by(group, learning_phase, symbolic_distance) %>%
    summarise(
      mean_rt = mean(rt_ms, na.rm = TRUE),
      se_rt = sd(rt_ms, na.rm = TRUE) / sqrt(n()),
      .groups = "drop"
    )
  
  # Create plot
  ggplot(distance_effects, aes(x = symbolic_distance, y = mean_rt, 
                              color = group, shape = learning_phase)) +
    geom_point(size = 3, alpha = 0.8) +
    geom_smooth(method = "lm", se = TRUE, alpha = 0.2) +
    scale_color_viridis_d(end = 0.8) +
    labs(
      x = "Symbolic Distance",
      y = "Mean Reaction Time (ms)",
      color = "Group",
      shape = "Learning Phase",
      title = "Symbolic Distance Effect Evolution"
    ) +
    theme_pubr() +
    theme(legend.position = "bottom")
}

# Strategy evolution visualization
plot_strategy_evolution <- function(strategy_data) {
  
  # Calculate strategy proportions over time
  strategy_props <- strategy_data$windows %>%
    group_by(window_id, strategy) %>%
    summarise(n = n(), .groups = "drop") %>%
    group_by(window_id) %>%
    mutate(
      prop = n / sum(n),
      total = sum(n)
    ) %>%
    ungroup()
  
  # Create stacked area plot
  ggplot(strategy_props, aes(x = window_id, y = prop, fill = strategy)) +
    geom_area(alpha = 0.8) +
    scale_fill_viridis_d() +
    scale_y_continuous(labels = scales::percent) +
    labs(
      x = "Learning Window",
      y = "Proportion of Participants",
      fill = "Strategy",
      title = "Strategy Evolution Over Learning"
    ) +
    theme_pubr() +
    theme(legend.position = "bottom")
}

# Generate all plots
learning_plot <- plot_learning_curves(data)
distance_plot <- plot_distance_effects(data)
strategy_plot <- plot_strategy_evolution(strategy_results)

# Combine plots
combined_plot <- learning_plot / distance_plot / strategy_plot
ggsave("output/figures/main_results.pdf", combined_plot, width = 12, height = 16)
```

## Report Generation

```r
# Generate comprehensive statistical report
generate_analysis_report <- function(data, results_list, output_file = "analysis_report.html") {
  
  # Create summary statistics table
  summary_stats <- data %>%
    group_by(group) %>%
    summarise(
      n_participants = n_distinct(learner_id),
      n_trials = n(),
      mean_accuracy = mean(correct, na.rm = TRUE),
      se_accuracy = sd(correct, na.rm = TRUE) / sqrt(n()),
      mean_rt = mean(rt_ms, na.rm = TRUE),
      median_rt = median(rt_ms, na.rm = TRUE),
      .groups = "drop"
    )
  
  # Hypothesis test results
  hypothesis_results <- tibble(
    hypothesis = c("H1: Distance Effect Reduction", 
                   "H2: Boundary Effect Attenuation",
                   "H3: Strategy Transition",
                   "H4: Transfer Efficiency"),
    p_value = c(
      anova(results_list$distance$model)["symbolic_distance_c:group:trial_block", "Pr(>F)"],
      anova(results_list$boundary$rt_model)["crosses_boundary:group:learning_phase", "Pr(>F)"],
      NA,  # Strategy analysis doesn't have single p-value
      NA   # Transfer analysis not implemented in this example
    ),
    effect_size = c(NA, NA, NA, NA),  # Calculate actual effect sizes
    conclusion = c("", "", "", "")
  )
  
  # Save results
  write_csv(summary_stats, "output/tables/summary_statistics.csv")
  write_csv(hypothesis_results, "output/tables/hypothesis_tests.csv")
  
  return(list(
    summary = summary_stats,
    hypotheses = hypothesis_results
  ))
}

# Generate report
report_results <- generate_analysis_report(data, list(
  distance = distance_results,
  boundary = boundary_results,
  strategy = strategy_results
))
```

## Power Analysis and Sample Size Planning

```r
# Post-hoc power analysis
calculate_power_analysis <- function(results) {
  
  # Extract effect sizes from models
  distance_effect <- summary(results$distance$model)$coefficients["symbolic_distance_c:groupadaptive", "t value"]
  
  # Power analysis for future studies
  power_results <- pwr.t.test(
    d = 0.5,  # Expected effect size
    sig.level = 0.05,
    power = 0.80,
    type = "two.sample"
  )
  
  return(power_results)
}

# Run power analysis
power_analysis <- calculate_power_analysis(list(
  distance = distance_results,
  boundary = boundary_results
))

print(power_analysis)
```

## Best Practices and Recommendations

### Data Quality Checks

```r
# Comprehensive data quality assessment
assess_data_quality <- function(data) {
  
  quality_report <- list(
    # Missing data patterns
    missing_patterns = data %>%
      summarise_all(~sum(is.na(.))) %>%
      gather(variable, missing_count) %>%
      mutate(missing_prop = missing_count / nrow(data)),
    
    # Outlier detection
    outliers = data %>%
      group_by(learner_id) %>%
      mutate(
        rt_z = abs(scale(rt_ms)[,1]),
        is_outlier = rt_z > 3
      ) %>%
      summarise(
        n_outliers = sum(is_outlier),
        prop_outliers = mean(is_outlier),
        .groups = "drop"
      ),
    
    # Completion rates
    completion = data %>%
      group_by(learner_id, group) %>%
      summarise(
        n_trials = n(),
        completion_rate = n_trials / max(data$trial_number),
        .groups = "drop"
      )
  )
  
  return(quality_report)
}

# Run quality assessment
quality_report <- assess_data_quality(data)
```

### Reproducible Research Workflow

1. **Version Control**: Use git for tracking analysis script changes
2. **Documented Code**: Include detailed comments explaining statistical choices
3. **Random Seeds**: Set seeds for reproducible results
4. **Package Versions**: Record package versions using `renv` or `packrat`
5. **Pre-registration**: Follow pre-registered analysis plans when possible

### Statistical Reporting Standards

```r
# Format results for publication
format_results_for_publication <- function(model) {
  
  # Extract key statistics
  model_summary <- summary(model)
  
  # Format with appropriate precision
  formatted <- model_summary$coefficients %>%
    as_tibble(rownames = "term") %>%
    mutate(
      estimate_formatted = sprintf("%.3f", Estimate),
      se_formatted = sprintf("%.3f", `Std. Error`),
      t_formatted = sprintf("%.2f", `t value`),
      p_formatted = case_when(
        `Pr(>|t|)` < 0.001 ~ "< .001",
        `Pr(>|t|)` < 0.01 ~ sprintf("= %.3f", `Pr(>|t|)`),
        TRUE ~ sprintf("= %.2f", `Pr(>|t|)`)
      )
    )
  
  return(formatted)
}

# Example: Format distance effect results
distance_formatted <- format_results_for_publication(distance_results$model)
print(distance_formatted)
```

This comprehensive R analysis framework provides researchers with all the tools needed for rigorous analysis of adaptive learning data, from data preprocessing through publication-ready statistical modeling and visualization.
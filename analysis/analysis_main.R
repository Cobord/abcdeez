# Main Analysis Script for Alphabet Terminal Prototype
# Adaptive Training for Flexible Skill Acquisition
# 
# This script analyzes the experimental data to test hypotheses H1-H4
# as specified in the pre-registration

# Load required libraries
library(tidyverse)
library(lme4)
library(lmerTest)
library(brms)
library(bayestestR)
library(performance)
library(ggpubr)
library(jsonlite)
library(pwr)

# Set options
options(mc.cores = parallel::detectCores())
options(scipen = 999)

# Source helper functions
source("analysis_helpers.R")
source("preprocessing.R")
source("hypothesis_tests.R")
source("visualizations.R")

# =====================================
# 1. DATA LOADING
# =====================================

# Load data from CSV export
load_experimental_data <- function(data_dir = "../data/") {
  # Load main response data
  responses_df <- read_csv(file.path(data_dir, "responses.csv"))
  
  # Load participant metadata
  participants_df <- read_csv(file.path(data_dir, "participants.csv"))
  
  # Load session summaries
  sessions_df <- read_csv(file.path(data_dir, "sessions.csv"))
  
  # Merge datasets
  full_data <- responses_df %>%
    left_join(participants_df, by = "learner_id") %>%
    left_join(sessions_df, by = c("learner_id", "session_id"))
  
  return(full_data)
}

# =====================================
# 2. DATA PREPROCESSING
# =====================================

preprocess_data <- function(raw_data) {
  processed <- raw_data %>%
    # Filter outliers
    filter(rt_ms > 200, rt_ms < 10000) %>%
    
    # Create derived variables
    mutate(
      # Log transform RT
      log_rt = log(rt_ms),
      
      # Calculate symbolic distance for pairwise tasks
      symbolic_distance = case_when(
        task_type == "PairwiseOrder" ~ abs(item_a_position - item_b_position),
        task_type == "KJump" ~ abs(k_value),
        task_type == "Segment" ~ segment_count,
        TRUE ~ 1
      ),
      
      # Identify chunk boundaries (every 6-7 letters)
      crosses_boundary = case_when(
        task_type == "Segment" ~ (start_position %/% 7) != ((start_position + segment_count) %/% 7),
        task_type == "KJump" ~ (start_position %/% 7) != ((start_position + k_value) %/% 7),
        TRUE ~ FALSE
      ),
      
      # Trial blocks for learning curves
      trial_block = cut(trial_number, breaks = seq(0, max(trial_number), by = 20)),
      
      # Strategy classification based on RT-distance correlation
      strategy_type = case_when(
        rt_distance_correlation > 0.7 ~ "serial_scan",
        rt_distance_correlation < 0.3 ~ "direct_index",
        TRUE ~ "hybrid"
      )
    ) %>%
    
    # Center and scale predictors
    mutate(
      symbolic_distance_c = scale(symbolic_distance, center = TRUE, scale = TRUE),
      trial_number_c = scale(trial_number, center = TRUE, scale = TRUE),
      difficulty_c = scale(difficulty, center = TRUE, scale = TRUE)
    )
  
  return(processed)
}

# =====================================
# 3. HYPOTHESIS TESTING
# =====================================

# H1: Representational Shift (Symbolic Distance Effect)
test_h1_distance_effect <- function(data) {
  # Filter to pairwise comparison tasks
  pairwise_data <- data %>%
    filter(task_type == "PairwiseOrder")
  
  # Fit hierarchical model
  h1_model <- lmer(
    log_rt ~ symbolic_distance_c * group * trial_block + 
      (1 + symbolic_distance_c | learner_id),
    data = pairwise_data,
    REML = FALSE
  )
  
  # Extract slopes for each group over time
  h1_slopes <- pairwise_data %>%
    group_by(group, trial_block) %>%
    do(tidy(lm(log_rt ~ symbolic_distance_c, data = .))) %>%
    filter(term == "symbolic_distance_c") %>%
    rename(slope = estimate)
  
  # Test group differences in slope reduction
  slope_change <- h1_slopes %>%
    group_by(group) %>%
    summarise(
      initial_slope = first(slope),
      final_slope = last(slope),
      slope_reduction = initial_slope - final_slope
    )
  
  # ANOVA for group differences
  h1_anova <- anova(h1_model)
  
  # Effect sizes
  h1_effect_size <- effectsize::eta_squared(h1_anova)
  
  return(list(
    model = h1_model,
    slopes = h1_slopes,
    slope_change = slope_change,
    anova = h1_anova,
    effect_size = h1_effect_size
  ))
}

# H2: Seam Smoothing (Chunk Boundary Effect)
test_h2_boundary_effect <- function(data) {
  # Filter to segment tasks
  segment_data <- data %>%
    filter(task_type == "Segment")
  
  # Model boundary crossing effect
  h2_model <- glmer(
    correct ~ crosses_boundary * group * trial_block + 
      (1 + crosses_boundary | learner_id),
    data = segment_data,
    family = binomial()
  )
  
  # RT model for boundaries
  h2_rt_model <- lmer(
    log_rt ~ crosses_boundary * group * trial_block + 
      (1 + crosses_boundary | learner_id),
    data = segment_data
  )
  
  # Calculate boundary penalties
  boundary_penalties <- segment_data %>%
    group_by(group, trial_block, crosses_boundary) %>%
    summarise(
      mean_rt = mean(rt_ms),
      accuracy = mean(correct),
      .groups = "drop"
    ) %>%
    pivot_wider(
      names_from = crosses_boundary,
      values_from = c(mean_rt, accuracy)
    ) %>%
    mutate(
      rt_penalty = mean_rt_TRUE - mean_rt_FALSE,
      accuracy_penalty = accuracy_FALSE - accuracy_TRUE
    )
  
  return(list(
    accuracy_model = h2_model,
    rt_model = h2_rt_model,
    penalties = boundary_penalties
  ))
}

# H3: Operator Disentangling (Complex Operation Proficiency)
test_h3_operation_proficiency <- function(data) {
  # Focus on complex operations
  complex_data <- data %>%
    filter(task_type %in% c("KJump", "Segment")) %>%
    mutate(
      operation_complexity = case_when(
        task_type == "KJump" ~ abs(k_value),
        task_type == "Segment" & reverse == TRUE ~ segment_count * 1.5,
        task_type == "Segment" ~ segment_count,
        TRUE ~ 1
      )
    )
  
  # Model proficiency gains
  h3_model <- glmer(
    correct ~ operation_complexity * group * trial_block + 
      (1 + operation_complexity | learner_id),
    data = complex_data,
    family = binomial()
  )
  
  # Calculate proficiency trajectories
  proficiency_curves <- complex_data %>%
    group_by(group, trial_block, task_type) %>%
    summarise(
      accuracy = mean(correct),
      mean_rt = mean(rt_ms),
      .groups = "drop"
    )
  
  return(list(
    model = h3_model,
    proficiency_curves = proficiency_curves
  ))
}

# H4: Transfer Efficiency
test_h4_transfer <- function(data) {
  # Filter to transfer phase data
  transfer_data <- data %>%
    filter(phase == "transfer")
  
  # Model learning rate in transfer domain
  h4_model <- glmer(
    correct ~ trial_number_c * group + 
      (1 + trial_number_c | learner_id),
    data = transfer_data,
    family = binomial()
  )
  
  # Calculate learning rates
  learning_rates <- transfer_data %>%
    group_by(group, learner_id) %>%
    do(tidy(glm(correct ~ trial_number, data = ., family = binomial()))) %>%
    filter(term == "trial_number") %>%
    rename(learning_rate = estimate)
  
  # Compare initial performance
  initial_transfer <- transfer_data %>%
    filter(trial_number <= 10) %>%
    group_by(group) %>%
    summarise(
      initial_accuracy = mean(correct),
      initial_rt = mean(rt_ms)
    )
  
  return(list(
    model = h4_model,
    learning_rates = learning_rates,
    initial_performance = initial_transfer
  ))
}

# =====================================
# 4. BAYESIAN ANALYSIS
# =====================================

run_bayesian_analysis <- function(data) {
  # Bayesian version of H1 model
  h1_bayes <- brm(
    log_rt ~ symbolic_distance_c * group * trial_block + 
      (1 + symbolic_distance_c | learner_id),
    data = filter(data, task_type == "PairwiseOrder"),
    prior = c(
      prior(normal(0, 1), class = b),
      prior(exponential(1), class = sd)
    ),
    iter = 4000,
    chains = 4,
    cores = 4
  )
  
  # Extract posterior distributions
  posteriors <- posterior_samples(h1_bayes)
  
  # Calculate contrasts
  contrasts <- hypothesis(
    h1_bayes,
    c("symbolic_distance_c:groupAdaptive < symbolic_distance_c:groupLinear",
      "symbolic_distance_c:groupAdaptive < symbolic_distance_c:groupYoked")
  )
  
  return(list(
    model = h1_bayes,
    posteriors = posteriors,
    contrasts = contrasts
  ))
}

# =====================================
# 5. MULTIPLE COMPARISON CORRECTIONS
# =====================================

apply_corrections <- function(p_values) {
  corrections <- data.frame(
    test = names(p_values),
    p_raw = unlist(p_values),
    p_bonferroni = p.adjust(unlist(p_values), method = "bonferroni"),
    p_fdr = p.adjust(unlist(p_values), method = "fdr"),
    p_holm = p.adjust(unlist(p_values), method = "holm")
  ) %>%
    mutate(
      sig_raw = p_raw < 0.05,
      sig_bonferroni = p_bonferroni < 0.05,
      sig_fdr = p_fdr < 0.05,
      sig_holm = p_holm < 0.05
    )
  
  return(corrections)
}

# =====================================
# 6. MAIN ANALYSIS PIPELINE
# =====================================

run_main_analysis <- function() {
  # Load data
  cat("Loading data...\n")
  raw_data <- load_experimental_data()
  
  # Preprocess
  cat("Preprocessing data...\n")
  data <- preprocess_data(raw_data)
  
  # Save preprocessed data
  write_csv(data, "../output/preprocessed_data.csv")
  
  # Test hypotheses
  cat("Testing H1: Representational Shift...\n")
  h1_results <- test_h1_distance_effect(data)
  
  cat("Testing H2: Seam Smoothing...\n")
  h2_results <- test_h2_boundary_effect(data)
  
  cat("Testing H3: Operator Disentangling...\n")
  h3_results <- test_h3_operation_proficiency(data)
  
  cat("Testing H4: Transfer Efficiency...\n")
  h4_results <- test_h4_transfer(data)
  
  # Collect p-values
  p_values <- list(
    h1_interaction = anova(h1_results$model)["symbolic_distance_c:group:trial_block", "Pr(>F)"],
    h2_accuracy = summary(h2_results$accuracy_model)$coefficients["crosses_boundaryTRUE:groupAdaptive", "Pr(>|z|)"],
    h2_rt = anova(h2_results$rt_model)["crosses_boundary:group:trial_block", "Pr(>F)"],
    h3_proficiency = summary(h3_results$model)$coefficients["operation_complexity:groupAdaptive", "Pr(>|z|)"],
    h4_transfer = summary(h4_results$model)$coefficients["trial_number_c:groupAdaptive", "Pr(>|z|)"]
  )
  
  # Apply corrections
  cat("Applying multiple comparison corrections...\n")
  corrections <- apply_corrections(p_values)
  
  # Run Bayesian analysis
  cat("Running Bayesian analysis...\n")
  bayes_results <- run_bayesian_analysis(data)
  
  # Save results
  cat("Saving results...\n")
  save(h1_results, h2_results, h3_results, h4_results, 
       corrections, bayes_results,
       file = "../output/analysis_results.RData")
  
  # Generate report
  cat("Generating report...\n")
  source("generate_report.R")
  
  return(list(
    h1 = h1_results,
    h2 = h2_results,
    h3 = h3_results,
    h4 = h4_results,
    corrections = corrections,
    bayes = bayes_results
  ))
}

# Run analysis if executed directly
if (!interactive()) {
  results <- run_main_analysis()
  print(results$corrections)
}
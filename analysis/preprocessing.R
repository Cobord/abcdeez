# Data Preprocessing Functions
# Handles data cleaning, transformation, and feature engineering

library(tidyverse)
library(lubridate)

# =====================================
# DATA CLEANING
# =====================================

clean_response_data <- function(df) {
  df %>%
    # Remove invalid responses
    filter(!is.na(rt_ms), !is.na(correct)) %>%
    
    # Remove extreme outliers (likely errors)
    filter(rt_ms >= 100, rt_ms <= 30000) %>%
    
    # Fix data types
    mutate(
      learner_id = as.factor(learner_id),
      session_id = as.factor(session_id),
      task_type = as.factor(task_type),
      correct = as.logical(correct),
      timestamp = ymd_hms(timestamp)
    ) %>%
    
    # Sort by participant and time
    arrange(learner_id, timestamp)
}

# =====================================
# FEATURE ENGINEERING
# =====================================

extract_task_features <- function(df) {
  df %>%
    mutate(
      # Parse task-specific parameters from JSON if needed
      task_params = map(task_json, ~ fromJSON(.x)),
      
      # Extract specific features based on task type
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
      segment_count = case_when(
        task_type == "Segment" ~ map_dbl(task_params, ~ .x$count),
        TRUE ~ NA_real_
      ),
      reverse = case_when(
        task_type == "Segment" ~ map_lgl(task_params, ~ .x$reverse),
        TRUE ~ NA
      )
    )
}

# Calculate symbolic distances
calculate_distances <- function(df, alphabet_mapping = NULL) {
  # Default alphabet mapping if not provided
  if (is.null(alphabet_mapping)) {
    alphabet_mapping <- setNames(1:26, LETTERS)
  }
  
  df %>%
    mutate(
      # Map items to positions
      item_a_position = alphabet_mapping[item_a],
      item_b_position = alphabet_mapping[item_b],
      start_position = alphabet_mapping[start_item],
      
      # Calculate distances
      pairwise_distance = abs(item_a_position - item_b_position),
      jump_distance = abs(k_value),
      segment_length = segment_count
    ) %>%
    
    # Unified distance measure
    mutate(
      task_distance = case_when(
        task_type == "PairwiseOrder" ~ pairwise_distance,
        task_type == "KJump" ~ jump_distance,
        task_type == "Segment" ~ segment_length,
        task_type == "Successor" ~ 1,
        task_type == "Predecessor" ~ 1,
        TRUE ~ NA_real_
      )
    )
}

# =====================================
# LEARNING METRICS
# =====================================

calculate_learning_metrics <- function(df, window_size = 20) {
  df %>%
    group_by(learner_id) %>%
    arrange(trial_number) %>%
    mutate(
      # Rolling accuracy
      rolling_accuracy = zoo::rollmean(
        correct, 
        k = window_size, 
        fill = NA, 
        align = "right"
      ),
      
      # Rolling RT
      rolling_rt = zoo::rollmean(
        rt_ms, 
        k = window_size, 
        fill = NA, 
        align = "right"
      ),
      
      # Cumulative accuracy
      cumulative_accuracy = cummean(correct),
      
      # Trial blocks
      trial_block = cut(
        trial_number, 
        breaks = seq(0, max(trial_number), by = window_size),
        labels = FALSE
      ),
      
      # Learning phase
      learning_phase = case_when(
        trial_number <= 100 ~ "early",
        trial_number <= 300 ~ "middle",
        TRUE ~ "late"
      )
    ) %>%
    ungroup()
}

# =====================================
# STRATEGY DETECTION
# =====================================

detect_strategies <- function(df, window_size = 50) {
  df %>%
    filter(task_type %in% c("PairwiseOrder", "KJump")) %>%
    group_by(learner_id) %>%
    arrange(trial_number) %>%
    mutate(
      # Calculate rolling correlation between RT and distance
      window_id = (row_number() - 1) %/% window_size
    ) %>%
    group_by(learner_id, window_id) %>%
    summarise(
      rt_distance_corr = cor(rt_ms, task_distance, use = "complete.obs"),
      mean_rt = mean(rt_ms),
      sd_rt = sd(rt_ms),
      trial_start = min(trial_number),
      trial_end = max(trial_number),
      .groups = "drop"
    ) %>%
    mutate(
      # Classify strategy based on correlation
      strategy = case_when(
        rt_distance_corr > 0.7 ~ "serial_scan",
        rt_distance_corr > 0.5 ~ "mixed_serial",
        rt_distance_corr > 0.3 ~ "transitioning",
        rt_distance_corr > 0.1 ~ "mixed_direct",
        TRUE ~ "direct_access"
      )
    )
}

# =====================================
# CHUNK BOUNDARY ANALYSIS
# =====================================

identify_chunk_boundaries <- function(df, chunk_size = 7) {
  df %>%
    mutate(
      # Standard chunk boundaries (e.g., every 7 letters for alphabet)
      start_chunk = (start_position - 1) %/% chunk_size,
      end_chunk = (start_position + task_distance - 1) %/% chunk_size,
      
      # Does task cross a chunk boundary?
      crosses_chunk = start_chunk != end_chunk,
      
      # Number of boundaries crossed
      n_boundaries_crossed = abs(end_chunk - start_chunk),
      
      # Is this at a chunk edge?
      at_chunk_edge = (start_position - 1) %% chunk_size %in% c(0, chunk_size - 1)
    )
}

# =====================================
# ERROR ANALYSIS
# =====================================

analyze_error_patterns <- function(df) {
  error_df <- df %>%
    filter(!correct) %>%
    group_by(learner_id, task_type) %>%
    summarise(
      n_errors = n(),
      mean_error_rt = mean(rt_ms),
      .groups = "drop"
    )
  
  # Confusion matrix for pairwise comparisons
  confusion_matrix <- df %>%
    filter(task_type == "PairwiseOrder", !correct) %>%
    count(item_a, item_b, name = "confusion_count") %>%
    arrange(desc(confusion_count))
  
  # Error clustering analysis
  error_distances <- df %>%
    filter(!correct, task_type %in% c("PairwiseOrder", "KJump")) %>%
    group_by(task_distance) %>%
    summarise(
      error_rate = mean(!correct),
      n_trials = n(),
      .groups = "drop"
    )
  
  return(list(
    summary = error_df,
    confusions = confusion_matrix,
    by_distance = error_distances
  ))
}

# =====================================
# MAIN PREPROCESSING PIPELINE
# =====================================

preprocess_pipeline <- function(raw_file_path, output_dir = "../data/processed/") {
  # Create output directory
  dir.create(output_dir, recursive = TRUE, showWarnings = FALSE)
  
  # Load raw data
  raw_df <- read_csv(raw_file_path)
  
  # Apply preprocessing steps
  processed_df <- raw_df %>%
    clean_response_data() %>%
    extract_task_features() %>%
    calculate_distances() %>%
    calculate_learning_metrics() %>%
    identify_chunk_boundaries()
  
  # Calculate additional summaries
  strategies <- detect_strategies(processed_df)
  errors <- analyze_error_patterns(processed_df)
  
  # Save processed data
  write_csv(processed_df, file.path(output_dir, "processed_responses.csv"))
  write_csv(strategies, file.path(output_dir, "strategy_analysis.csv"))
  saveRDS(errors, file.path(output_dir, "error_analysis.rds"))
  
  # Create participant summary
  participant_summary <- processed_df %>%
    group_by(learner_id, group) %>%
    summarise(
      n_trials = n(),
      overall_accuracy = mean(correct),
      mean_rt = mean(rt_ms),
      median_rt = median(rt_ms),
      final_accuracy = mean(correct[learning_phase == "late"]),
      initial_accuracy = mean(correct[learning_phase == "early"]),
      learning_gain = final_accuracy - initial_accuracy,
      .groups = "drop"
    )
  
  write_csv(participant_summary, file.path(output_dir, "participant_summary.csv"))
  
  return(list(
    data = processed_df,
    strategies = strategies,
    errors = errors,
    summary = participant_summary
  ))
}

# Example usage
if (FALSE) {
  preprocessed <- preprocess_pipeline("../data/raw/responses.csv")
  print(summary(preprocessed$data))
}
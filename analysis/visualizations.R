# Visualization Functions for Analysis Results
# Creates publication-quality figures for the paper

library(tidyverse)
library(ggplot2)
library(ggpubr)
library(patchwork)
library(viridis)

# Set theme for all plots
theme_set(theme_pubr(base_size = 12))

# =====================================
# H1: SYMBOLIC DISTANCE EFFECT
# =====================================

plot_distance_effect <- function(data, save_path = NULL) {
  # Calculate mean RT by distance and group
  distance_summary <- data %>%
    filter(task_type == "PairwiseOrder") %>%
    group_by(group, task_distance, learning_phase) %>%
    summarise(
      mean_rt = mean(rt_ms),
      se_rt = sd(rt_ms) / sqrt(n()),
      .groups = "drop"
    )
  
  p1 <- ggplot(distance_summary, 
               aes(x = task_distance, y = mean_rt, 
                   color = group, linetype = learning_phase)) +
    geom_line(size = 1) +
    geom_point(size = 2) +
    geom_errorbar(aes(ymin = mean_rt - se_rt, 
                      ymax = mean_rt + se_rt), 
                  width = 0.2, alpha = 0.5) +
    scale_color_manual(values = c("Adaptive" = "#2E7D32", 
                                   "Linear" = "#1565C0", 
                                   "Yoked" = "#E65100")) +
    labs(
      title = "Symbolic Distance Effect by Group and Phase",
      x = "Symbolic Distance",
      y = "Response Time (ms)",
      color = "Training Group",
      linetype = "Learning Phase"
    ) +
    theme(legend.position = "bottom")
  
  # Slope reduction over time
  slope_data <- data %>%
    filter(task_type == "PairwiseOrder") %>%
    group_by(group, trial_block) %>%
    do(tidy(lm(log(rt_ms) ~ task_distance, data = .))) %>%
    filter(term == "task_distance")
  
  p2 <- ggplot(slope_data, 
               aes(x = trial_block, y = estimate, 
                   color = group, fill = group)) +
    geom_smooth(method = "loess", alpha = 0.2) +
    geom_point(size = 2) +
    geom_hline(yintercept = 0, linetype = "dashed", alpha = 0.5) +
    scale_color_manual(values = c("Adaptive" = "#2E7D32", 
                                   "Linear" = "#1565C0", 
                                   "Yoked" = "#E65100")) +
    scale_fill_manual(values = c("Adaptive" = "#2E7D32", 
                                  "Linear" = "#1565C0", 
                                  "Yoked" = "#E65100")) +
    labs(
      title = "Distance Effect Slope Over Training",
      x = "Trial Block",
      y = "RT-Distance Slope (log ms/item)",
      color = "Training Group",
      fill = "Training Group"
    ) +
    theme(legend.position = "bottom")
  
  # Combine plots
  combined <- p1 + p2 + plot_layout(ncol = 2)
  
  if (!is.null(save_path)) {
    ggsave(save_path, combined, width = 12, height = 6, dpi = 300)
  }
  
  return(combined)
}

# =====================================
# H2: CHUNK BOUNDARY EFFECT
# =====================================

plot_boundary_effect <- function(data, save_path = NULL) {
  # Boundary crossing penalties
  boundary_summary <- data %>%
    filter(task_type == "Segment") %>%
    group_by(group, crosses_chunk, trial_block) %>%
    summarise(
      mean_rt = mean(rt_ms),
      accuracy = mean(correct),
      se_rt = sd(rt_ms) / sqrt(n()),
      .groups = "drop"
    )
  
  # RT penalty plot
  p1 <- boundary_summary %>%
    pivot_wider(names_from = crosses_chunk, 
                values_from = c(mean_rt, se_rt)) %>%
    mutate(penalty = mean_rt_TRUE - mean_rt_FALSE) %>%
    ggplot(aes(x = trial_block, y = penalty, 
               color = group, fill = group)) +
    geom_smooth(method = "loess", alpha = 0.2) +
    geom_point(size = 2) +
    geom_hline(yintercept = 0, linetype = "dashed", alpha = 0.5) +
    scale_color_manual(values = c("Adaptive" = "#2E7D32", 
                                   "Linear" = "#1565C0", 
                                   "Yoked" = "#E65100")) +
    scale_fill_manual(values = c("Adaptive" = "#2E7D32", 
                                  "Linear" = "#1565C0", 
                                  "Yoked" = "#E65100")) +
    labs(
      title = "Chunk Boundary RT Penalty",
      x = "Trial Block",
      y = "RT Penalty (ms)",
      subtitle = "Difference: Crossing vs. Within Chunk"
    ) +
    theme(legend.position = "bottom")
  
  # Accuracy difference plot
  p2 <- boundary_summary %>%
    pivot_wider(names_from = crosses_chunk, 
                values_from = accuracy) %>%
    mutate(accuracy_drop = `FALSE` - `TRUE`) %>%
    ggplot(aes(x = trial_block, y = accuracy_drop, 
               color = group, fill = group)) +
    geom_smooth(method = "loess", alpha = 0.2) +
    geom_point(size = 2) +
    geom_hline(yintercept = 0, linetype = "dashed", alpha = 0.5) +
    scale_color_manual(values = c("Adaptive" = "#2E7D32", 
                                   "Linear" = "#1565C0", 
                                   "Yoked" = "#E65100")) +
    scale_fill_manual(values = c("Adaptive" = "#2E7D32", 
                                  "Linear" = "#1565C0", 
                                  "Yoked" = "#E65100")) +
    labs(
      title = "Chunk Boundary Accuracy Drop",
      x = "Trial Block",
      y = "Accuracy Difference",
      subtitle = "Within - Crossing Chunk"
    ) +
    theme(legend.position = "bottom")
  
  combined <- p1 + p2 + plot_layout(ncol = 2)
  
  if (!is.null(save_path)) {
    ggsave(save_path, combined, width = 12, height = 6, dpi = 300)
  }
  
  return(combined)
}

# =====================================
# H3: OPERATION PROFICIENCY
# =====================================

plot_operation_proficiency <- function(data, save_path = NULL) {
  # Proficiency by operation type
  proficiency_data <- data %>%
    filter(task_type %in% c("KJump", "Segment", "Successor", "Predecessor")) %>%
    group_by(group, task_type, trial_block) %>%
    summarise(
      accuracy = mean(correct),
      mean_rt = mean(rt_ms),
      se_accuracy = sd(correct) / sqrt(n()),
      .groups = "drop"
    )
  
  p1 <- ggplot(proficiency_data, 
               aes(x = trial_block, y = accuracy, 
                   color = task_type)) +
    geom_line(size = 1) +
    geom_point(size = 1.5) +
    geom_ribbon(aes(ymin = accuracy - se_accuracy,
                    ymax = accuracy + se_accuracy,
                    fill = task_type), alpha = 0.2) +
    facet_wrap(~ group, ncol = 3) +
    scale_color_viridis_d() +
    scale_fill_viridis_d() +
    labs(
      title = "Operation Proficiency Development",
      x = "Trial Block",
      y = "Accuracy",
      color = "Operation",
      fill = "Operation"
    ) +
    theme(legend.position = "bottom")
  
  # Complex vs simple operations
  complexity_data <- data %>%
    mutate(
      complexity = case_when(
        task_type %in% c("Successor", "Predecessor") ~ "Simple",
        task_type %in% c("KJump", "Segment") ~ "Complex",
        TRUE ~ "Other"
      )
    ) %>%
    filter(complexity != "Other") %>%
    group_by(group, complexity, trial_block) %>%
    summarise(
      accuracy = mean(correct),
      mean_rt = mean(rt_ms),
      .groups = "drop"
    )
  
  p2 <- ggplot(complexity_data, 
               aes(x = trial_block, y = accuracy, 
                   color = group, linetype = complexity)) +
    geom_smooth(method = "loess", se = TRUE, alpha = 0.2) +
    scale_color_manual(values = c("Adaptive" = "#2E7D32", 
                                   "Linear" = "#1565C0", 
                                   "Yoked" = "#E65100")) +
    labs(
      title = "Simple vs Complex Operation Mastery",
      x = "Trial Block",
      y = "Accuracy",
      color = "Training Group",
      linetype = "Operation Complexity"
    ) +
    theme(legend.position = "bottom")
  
  combined <- p1 / p2 + plot_layout(heights = c(1, 1))
  
  if (!is.null(save_path)) {
    ggsave(save_path, combined, width = 10, height = 10, dpi = 300)
  }
  
  return(combined)
}

# =====================================
# H4: TRANSFER EFFICIENCY
# =====================================

plot_transfer_performance <- function(data, save_path = NULL) {
  transfer_data <- data %>%
    filter(phase == "transfer")
  
  # Learning curves in transfer domain
  p1 <- transfer_data %>%
    group_by(group, trial_number) %>%
    summarise(
      accuracy = mean(correct),
      se = sd(correct) / sqrt(n()),
      .groups = "drop"
    ) %>%
    ggplot(aes(x = trial_number, y = accuracy, 
               color = group, fill = group)) +
    geom_smooth(method = "loess", alpha = 0.2) +
    geom_ribbon(aes(ymin = accuracy - se, ymax = accuracy + se), 
                alpha = 0.1) +
    scale_color_manual(values = c("Adaptive" = "#2E7D32", 
                                   "Linear" = "#1565C0", 
                                   "Yoked" = "#E65100")) +
    scale_fill_manual(values = c("Adaptive" = "#2E7D32", 
                                  "Linear" = "#1565C0", 
                                  "Yoked" = "#E65100")) +
    labs(
      title = "Transfer Domain Learning Curves",
      x = "Trial Number",
      y = "Accuracy",
      color = "Training Group",
      fill = "Training Group"
    ) +
    theme(legend.position = "bottom")
  
  # Initial vs final performance
  performance_summary <- transfer_data %>%
    mutate(
      phase = ifelse(trial_number <= 20, "Initial", "Final")
    ) %>%
    group_by(group, phase) %>%
    summarise(
      accuracy = mean(correct),
      mean_rt = mean(rt_ms),
      se_accuracy = sd(correct) / sqrt(n()),
      se_rt = sd(rt_ms) / sqrt(n()),
      .groups = "drop"
    )
  
  p2 <- ggplot(performance_summary, 
               aes(x = phase, y = accuracy, 
                   fill = group)) +
    geom_bar(stat = "identity", position = position_dodge(0.9)) +
    geom_errorbar(aes(ymin = accuracy - se_accuracy,
                      ymax = accuracy + se_accuracy),
                  position = position_dodge(0.9),
                  width = 0.2) +
    scale_fill_manual(values = c("Adaptive" = "#2E7D32", 
                                  "Linear" = "#1565C0", 
                                  "Yoked" = "#E65100")) +
    labs(
      title = "Initial vs Final Transfer Performance",
      x = "Transfer Phase",
      y = "Accuracy",
      fill = "Training Group"
    ) +
    theme(legend.position = "bottom")
  
  combined <- p1 + p2 + plot_layout(ncol = 2)
  
  if (!is.null(save_path)) {
    ggsave(save_path, combined, width = 12, height = 6, dpi = 300)
  }
  
  return(combined)
}

# =====================================
# STRATEGY ANALYSIS
# =====================================

plot_strategy_evolution <- function(strategy_data, save_path = NULL) {
  # Strategy proportions over time
  strategy_props <- strategy_data %>%
    group_by(learner_id) %>%
    mutate(window_num = row_number()) %>%
    ungroup() %>%
    group_by(group, window_num, strategy) %>%
    summarise(n = n(), .groups = "drop") %>%
    group_by(group, window_num) %>%
    mutate(prop = n / sum(n)) %>%
    ungroup()
  
  p <- ggplot(strategy_props, 
              aes(x = window_num, y = prop, fill = strategy)) +
    geom_area(alpha = 0.7) +
    facet_wrap(~ group, ncol = 1) +
    scale_fill_viridis_d(option = "C") +
    labs(
      title = "Strategy Evolution Over Training",
      x = "Time Window",
      y = "Proportion of Participants",
      fill = "Strategy"
    ) +
    theme(legend.position = "bottom")
  
  if (!is.null(save_path)) {
    ggsave(save_path, p, width = 10, height = 8, dpi = 300)
  }
  
  return(p)
}

# =====================================
# SUMMARY FIGURE FOR PAPER
# =====================================

create_summary_figure <- function(data, save_path = NULL) {
  # Create key panels for each hypothesis
  
  # H1: Distance effect
  h1_data <- data %>%
    filter(task_type == "PairwiseOrder") %>%
    group_by(group, learning_phase, task_distance) %>%
    summarise(mean_rt = mean(log(rt_ms)), .groups = "drop")
  
  p1 <- ggplot(h1_data %>% filter(learning_phase %in% c("early", "late")), 
               aes(x = task_distance, y = mean_rt, 
                   color = group, linetype = learning_phase)) +
    geom_line(size = 1) +
    scale_color_manual(values = c("Adaptive" = "#2E7D32", 
                                   "Linear" = "#1565C0", 
                                   "Yoked" = "#E65100")) +
    labs(title = "A. Symbolic Distance Effect",
         x = "Distance", y = "Log RT") +
    theme(legend.position = "none")
  
  # H2: Boundary effect
  h2_data <- data %>%
    filter(task_type == "Segment") %>%
    group_by(group, trial_block, crosses_chunk) %>%
    summarise(mean_rt = mean(rt_ms), .groups = "drop") %>%
    pivot_wider(names_from = crosses_chunk, values_from = mean_rt) %>%
    mutate(penalty = `TRUE` - `FALSE`)
  
  p2 <- ggplot(h2_data, 
               aes(x = trial_block, y = penalty, color = group)) +
    geom_smooth(method = "loess", se = FALSE) +
    scale_color_manual(values = c("Adaptive" = "#2E7D32", 
                                   "Linear" = "#1565C0", 
                                   "Yoked" = "#E65100")) +
    labs(title = "B. Chunk Boundary Penalty",
         x = "Trial Block", y = "RT Penalty (ms)") +
    theme(legend.position = "none")
  
  # H3: Operation proficiency
  h3_data <- data %>%
    mutate(complex = task_type %in% c("KJump", "Segment")) %>%
    filter(!is.na(complex)) %>%
    group_by(group, trial_block, complex) %>%
    summarise(accuracy = mean(correct), .groups = "drop")
  
  p3 <- ggplot(h3_data %>% filter(complex), 
               aes(x = trial_block, y = accuracy, color = group)) +
    geom_smooth(method = "loess", se = FALSE) +
    scale_color_manual(values = c("Adaptive" = "#2E7D32", 
                                   "Linear" = "#1565C0", 
                                   "Yoked" = "#E65100")) +
    labs(title = "C. Complex Operation Proficiency",
         x = "Trial Block", y = "Accuracy") +
    theme(legend.position = "none")
  
  # H4: Transfer
  h4_data <- data %>%
    filter(phase == "transfer") %>%
    group_by(group, trial_number) %>%
    summarise(accuracy = mean(correct), .groups = "drop")
  
  p4 <- ggplot(h4_data, 
               aes(x = trial_number, y = accuracy, color = group)) +
    geom_smooth(method = "loess", se = FALSE) +
    scale_color_manual(values = c("Adaptive" = "#2E7D32", 
                                   "Linear" = "#1565C0", 
                                   "Yoked" = "#E65100")) +
    labs(title = "D. Transfer Performance",
         x = "Transfer Trial", y = "Accuracy") +
    theme(legend.position = "bottom") +
    guides(color = guide_legend(title = "Training Group"))
  
  # Combine all panels
  combined <- (p1 | p2) / (p3 | p4) + 
    plot_annotation(
      title = "Adaptive Training Builds Flexible Mental Models",
      theme = theme(plot.title = element_text(size = 16, face = "bold"))
    )
  
  if (!is.null(save_path)) {
    ggsave(save_path, combined, width = 12, height = 10, dpi = 300)
  }
  
  return(combined)
}

# Example usage
if (FALSE) {
  data <- read_csv("../data/processed/processed_responses.csv")
  
  # Create all figures
  plot_distance_effect(data, "../figures/h1_distance_effect.pdf")
  plot_boundary_effect(data, "../figures/h2_boundary_effect.pdf")
  plot_operation_proficiency(data, "../figures/h3_proficiency.pdf")
  plot_transfer_performance(data, "../figures/h4_transfer.pdf")
  
  # Create summary figure for paper
  create_summary_figure(data, "../figures/main_results.pdf")
}
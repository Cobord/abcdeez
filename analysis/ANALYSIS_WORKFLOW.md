# Analysis Workflow
## Reproducing the Statistical Analysis

This document provides a complete workflow for exporting data from the Rust application and analyzing it using the provided R scripts.

---

## 1. Data Export from Rust

### Step 1.1: Export Individual Learner Data

```rust
// In your Rust application
use graph_learning_core::export::{LearnerDataExport, PopulationAnalyzer};
use std::path::Path;

// For a single learner
let learner_export = LearnerDataExport::from_learner_model(
    &learner_model,
    sessions,
    Some("adaptive_group_01".to_string())
);

// Export all formats
learner_export.export_all_formats(Path::new("data/learner_01/"))?;
```

### Step 1.2: Export Population Data

```rust
// For multiple learners
let mut all_learners = Vec::new();
for (learner_id, model, sessions) in participants {
    let export = LearnerDataExport::from_learner_model(
        &model,
        sessions,
        Some(format!("{}_group", experiment_condition))
    );
    all_learners.push(export);
}

// Export combined population data
let analyzer = PopulationAnalyzer::new(all_learners);
analyzer.export_population_data(Path::new("data/population/"))?;
```

### Step 1.3: Generated Files

After export, you'll have:
```
data/
├── population/
│   ├── responses.csv         # All trial data
│   ├── participants.csv      # Participant metadata
│   ├── sessions.csv          # Session summaries
│   └── model_parameters.csv  # Model states
└── learner_01/
    ├── P001_full.json        # Complete data (JSON)
    ├── P001_responses.csv    # Trial data
    ├── P001_sessions.csv     # Session summaries
    └── P001_parameters.csv   # Model parameters
```

---

## 2. R Analysis Pipeline

### Step 2.1: Install Required R Packages

```r
# Install if not already installed
install.packages(c(
  "tidyverse",      # Data manipulation
  "lme4",           # Mixed models
  "lmerTest",       # Model testing
  "brms",           # Bayesian models
  "bayestestR",     # Bayesian analysis
  "performance",    # Model diagnostics
  "ggpubr",         # Publication plots
  "jsonlite",       # JSON parsing
  "pwr",            # Power analysis
  "patchwork",      # Plot composition
  "viridis"         # Color scales
))
```

### Step 2.2: Run Preprocessing

```r
# Set working directory to analysis folder
setwd("alphabet-terminal-prototype/analysis/")

# Load preprocessing functions
source("preprocessing.R")

# Preprocess the exported data
preprocessed <- preprocess_pipeline(
  raw_file_path = "../data/population/responses.csv",
  output_dir = "../data/processed/"
)

# View summary
summary(preprocessed$data)
print(preprocessed$summary)
```

### Step 2.3: Run Main Analysis

```r
# Load and run main analysis
source("analysis_main.R")

# Run complete analysis pipeline
results <- run_main_analysis()

# View corrected p-values
print(results$corrections)

# View Bayesian contrasts
print(results$bayes$contrasts)
```

### Step 2.4: Generate Visualizations

```r
# Load visualization functions
source("visualizations.R")

# Load processed data
data <- read_csv("../data/processed/processed_responses.csv")

# Create all hypothesis figures
plot_distance_effect(data, "../figures/h1_distance_effect.pdf")
plot_boundary_effect(data, "../figures/h2_boundary_effect.pdf")
plot_operation_proficiency(data, "../figures/h3_proficiency.pdf")
plot_transfer_performance(data, "../figures/h4_transfer.pdf")

# Create main summary figure for paper
create_summary_figure(data, "../figures/main_results.pdf")

# Strategy evolution plot
strategy_data <- read_csv("../data/processed/strategy_analysis.csv")
plot_strategy_evolution(strategy_data, "../figures/strategy_evolution.pdf")
```

---

## 3. Custom Analyses

### Example 3.1: Individual Participant Analysis

```r
# Load individual participant data
p001_data <- read_csv("../data/learner_01/P001_responses.csv")

# Analyze learning curve
library(ggplot2)
ggplot(p001_data, aes(x = trial_number, y = correct)) +
  geom_smooth(method = "loess", span = 0.2) +
  labs(title = "Individual Learning Curve",
       x = "Trial Number", 
       y = "Accuracy") +
  theme_minimal()

# Analyze RT-distance correlation over time
p001_data %>%
  filter(task_type == "PairwiseOrder") %>%
  mutate(block = cut(trial_number, breaks = 10)) %>%
  group_by(block) %>%
  summarise(
    correlation = cor(rt_ms, task_distance, use = "complete.obs"),
    mean_rt = mean(rt_ms)
  ) %>%
  ggplot(aes(x = block, y = correlation)) +
  geom_line(group = 1) +
  geom_point() +
  labs(title = "Strategy Evolution",
       x = "Trial Block",
       y = "RT-Distance Correlation")
```

### Example 3.2: Group Comparisons

```r
# Load participant summary
participants <- read_csv("../data/population/participants.csv")

# Compare groups
library(ggpubr)
ggboxplot(participants, 
          x = "group", 
          y = "overall_accuracy",
          color = "group",
          palette = c("#2E7D32", "#1565C0", "#E65100"),
          add = "jitter") +
  stat_compare_means(method = "anova") +
  stat_compare_means(comparisons = list(
    c("Adaptive", "Linear"),
    c("Adaptive", "Yoked"),
    c("Linear", "Yoked")
  ))
```

### Example 3.3: Model Parameter Analysis

```r
# Load model parameters
params <- read_csv("../data/population/model_parameters.csv")

# Analyze node embedding evolution
node_embeddings <- params %>%
  filter(parameter_type == "node_embedding") %>%
  mutate(
    node = str_extract(parameter_name, "[A-Z]$"),
    position_idx = as.numeric(factor(node, levels = LETTERS))
  )

# Plot learned vs true positions
ggplot(node_embeddings, aes(x = position_idx, y = value)) +
  geom_point(aes(color = learner_id), alpha = 0.5) +
  geom_smooth(method = "lm", se = FALSE) +
  geom_abline(slope = 1, intercept = 0, linetype = "dashed") +
  labs(title = "Learned vs True Alphabet Positions",
       x = "True Position",
       y = "Learned Position") +
  theme_minimal()
```

---

## 4. Statistical Power Analysis

```r
# Post-hoc power analysis
source("preprocessing.R")
data <- read_csv("../data/processed/processed_responses.csv")

# Calculate achieved power for H1
library(pwr)
h1_data <- data %>%
  filter(task_type == "PairwiseOrder") %>%
  group_by(group) %>%
  summarise(
    n = n_distinct(learner_id),
    slope = cor(rt_ms, task_distance)
  )

effect_size <- (h1_data$slope[1] - h1_data$slope[3]) / 
               sd(c(h1_data$slope[1], h1_data$slope[3]))

power_h1 <- pwr.t.test(
  n = mean(h1_data$n),
  d = effect_size,
  sig.level = 0.05,
  type = "two.sample"
)

print(power_h1)
```

---

## 5. Report Generation

### Generate Analysis Report

```r
# Create comprehensive report
rmarkdown::render(
  "analysis_report.Rmd",
  output_format = "html_document",
  output_file = "../output/analysis_report.html",
  params = list(
    data_path = "../data/processed/processed_responses.csv",
    results_path = "../output/analysis_results.RData"
  )
)
```

### Export Results Tables

```r
# Load results
load("../output/analysis_results.RData")

# Create publication-ready tables
library(gt)

# P-value correction table
corrections %>%
  gt() %>%
  tab_header(
    title = "Multiple Comparison Corrections",
    subtitle = "Primary Hypothesis Tests"
  ) %>%
  fmt_number(
    columns = contains("p_"),
    decimals = 4
  ) %>%
  tab_style(
    style = cell_fill(color = "#E8F5E9"),
    locations = cells_body(
      columns = everything(),
      rows = sig_fdr == TRUE
    )
  ) %>%
  gtsave("../tables/corrections_table.html")
```

---

## 6. Troubleshooting

### Common Issues

1. **Missing data columns**: Ensure export format matches DATA_DICTIONARY.md
2. **Memory issues with large datasets**: Process in chunks
3. **Convergence warnings in models**: Check data distribution and consider transformations

### Data Validation

```r
# Validate exported data
validate_data <- function(df) {
  checks <- list(
    has_required_columns = all(c("learner_id", "trial_number", "correct", "rt_ms") %in% names(df)),
    rt_positive = all(df$rt_ms > 0, na.rm = TRUE),
    accuracy_valid = all(df$correct %in% c(TRUE, FALSE), na.rm = TRUE),
    timestamps_ordered = !is.unsorted(df$timestamp)
  )
  
  if (!all(unlist(checks))) {
    warning("Data validation failed:")
    print(checks)
  } else {
    message("Data validation passed!")
  }
  
  return(checks)
}

# Run validation
data <- read_csv("../data/population/responses.csv")
validate_data(data)
```

---

## 7. Reproducibility Checklist

- [ ] Export data using consistent experiment IDs
- [ ] Document any data filtering or exclusion criteria
- [ ] Set random seed for reproducible analyses
- [ ] Save session info for R environment
- [ ] Archive raw data before preprocessing
- [ ] Document any deviations from pre-registration
- [ ] Create data availability statement

```r
# Save session information
sessionInfo() %>% 
  capture.output() %>% 
  writeLines("../output/session_info.txt")
```

---

## Contact

For questions about the analysis pipeline:
- Review the DATA_DICTIONARY.md for data format details
- Check the individual R script headers for function documentation
- Submit issues to the repository for bugs or clarifications
# Data Dictionary
## Alphabet Terminal Prototype Export Format

This document describes the data format exported by the Rust application for analysis in R or other statistical software.

---

## File Structure

The system exports data in multiple CSV files for easy analysis:

1. **responses.csv** - Trial-by-trial response data
2. **participants.csv** - Participant-level metadata
3. **sessions.csv** - Session-level summaries
4. **model_parameters.csv** - Learner model parameters over time

---

## 1. responses.csv - Trial Data

Primary data file containing one row per trial.

| Column | Type | Description | Example |
|--------|------|-------------|---------|
| **learner_id** | string | Unique participant identifier | "P001" |
| **session_id** | string | Session identifier | "S001_2024_01_15" |
| **trial_number** | integer | Trial number within participant | 1, 2, 3... |
| **timestamp** | datetime | ISO 8601 timestamp | "2024-01-15T14:30:00Z" |
| **group** | string | Experimental condition | "Adaptive", "Linear", "Yoked" |
| **phase** | string | Experimental phase | "training", "test", "transfer" |
| **task_type** | string | Type of task | "PairwiseOrder", "Successor", "KJump", "Segment" |
| **task_json** | json | Full task parameters | {"type": "KJump", "start": "G", "k": 3} |
| **correct** | boolean | Whether response was correct | true, false |
| **rt_ms** | numeric | Response time in milliseconds | 1234.56 |
| **user_answer** | string | Participant's response | "J" |
| **correct_answer** | string | Expected response | "J" |
| **difficulty** | numeric | Task difficulty (0-1) | 0.45 |

### Task-Specific Columns

Additional columns parsed from task_json:

#### PairwiseOrder Tasks
| Column | Type | Description |
|--------|------|-------------|
| **item_a** | string | First item in comparison | "C" |
| **item_b** | string | Second item in comparison | "F" |

#### KJump Tasks
| Column | Type | Description |
|--------|------|-------------|
| **start_item** | string | Starting position | "D" |
| **k_value** | integer | Jump distance (can be negative) | 3, -2 |

#### Segment Tasks
| Column | Type | Description |
|--------|------|-------------|
| **start_item** | string | Starting position | "M" |
| **segment_count** | integer | Number of items | 4 |
| **reverse** | boolean | Whether traversal is reversed | true, false |

---

## 2. participants.csv - Participant Metadata

One row per participant.

| Column | Type | Description | Example |
|--------|------|-------------|---------|
| **learner_id** | string | Unique participant identifier | "P001" |
| **group** | string | Experimental condition | "Adaptive" |
| **age** | integer | Participant age (if collected) | 23 |
| **gender** | string | Self-reported gender | "F", "M", "NB", "Other" |
| **start_date** | date | First session date | "2024-01-15" |
| **end_date** | date | Last session date | "2024-01-22" |
| **n_sessions** | integer | Total sessions completed | 7 |
| **total_trials** | integer | Total trials completed | 700 |
| **overall_accuracy** | numeric | Overall accuracy (0-1) | 0.823 |
| **mean_rt** | numeric | Mean response time (ms) | 1456.78 |
| **completed** | boolean | Completed all sessions | true |

---

## 3. sessions.csv - Session Summaries

One row per session.

| Column | Type | Description | Example |
|--------|------|-------------|---------|
| **learner_id** | string | Participant identifier | "P001" |
| **session_id** | string | Session identifier | "S001_2024_01_15" |
| **session_number** | integer | Session number (1-7) | 1 |
| **start_time** | datetime | Session start | "2024-01-15T14:00:00Z" |
| **end_time** | datetime | Session end | "2024-01-15T14:30:00Z" |
| **duration_minutes** | numeric | Session duration | 30.5 |
| **n_trials** | integer | Trials in session | 100 |
| **accuracy** | numeric | Session accuracy (0-1) | 0.76 |
| **mean_rt_ms** | numeric | Mean RT for session | 1234.56 |
| **median_rt_ms** | numeric | Median RT for session | 1100.00 |
| **strategy_detected** | string | Dominant strategy | "SerialScan", "DirectIndex", "Hybrid" |

---

## 4. model_parameters.csv - Learner Model States

Model parameters sampled at regular intervals.

| Column | Type | Description | Example |
|--------|------|-------------|---------|
| **learner_id** | string | Participant identifier | "P001" |
| **timestamp** | datetime | When parameters were saved | "2024-01-15T14:15:00Z" |
| **trial_number** | integer | Trial at time of snapshot | 250 |
| **parameter_type** | string | Type of parameter | "node_embedding", "operation_proficiency" |
| **parameter_name** | string | Specific parameter | "node_A", "Successor" |
| **value** | numeric | Parameter value | 0.823 |
| **uncertainty** | numeric | Uncertainty/variance | 0.15 |

### Parameter Types

- **node_embedding**: Position estimates for each item (A-Z)
- **operation_proficiency**: Skill level for each operation type
- **memory_strength**: Retention strength for each item
- **chunk_boundary**: Detected chunk boundaries and strengths
- **confusability**: Pairwise confusion probabilities

---

## 5. Derived Variables (Created in R)

The preprocessing scripts create additional variables:

| Variable | Description | Calculation |
|----------|-------------|-------------|
| **task_distance** | Unified distance measure | Varies by task type |
| **crosses_chunk** | Whether task crosses chunk boundary | Based on 7-item chunks |
| **trial_block** | Grouped trials (20 per block) | ceiling(trial_number / 20) |
| **learning_phase** | Early/Middle/Late | Based on trial_number |
| **rolling_accuracy** | Moving average accuracy | 20-trial window |
| **rt_distance_correlation** | RT-distance correlation | 50-trial windows |
| **strategy_classification** | Inferred strategy | Based on correlation |

---

## Missing Data Codes

- **NA**: Not applicable for this task type
- **-999**: Missing due to technical error
- **-888**: Participant skipped/timed out

---

## Example Data Loading in R

```r
library(tidyverse)

# Load all data files
responses <- read_csv("data/responses.csv")
participants <- read_csv("data/participants.csv")
sessions <- read_csv("data/sessions.csv")
model_params <- read_csv("data/model_parameters.csv")

# Join datasets
full_data <- responses %>%
  left_join(participants, by = "learner_id") %>%
  left_join(sessions, by = c("learner_id", "session_id"))

# Parse JSON task parameters
library(jsonlite)
task_params <- full_data %>%
  mutate(params = map(task_json, ~ fromJSON(.x)))
```

---

## Data Quality Checks

Before analysis, verify:

1. **Completeness**: Check for missing values in critical columns
2. **Range checks**: RT > 0, accuracy in [0,1], difficulty in [0,1]
3. **Consistency**: Timestamps increase monotonically
4. **Balance**: Similar n per group
5. **Outliers**: Flag RT < 200ms or > 10000ms

---

## Version History

- **v1.0** (2024-01): Initial export format
- **v1.1** (2024-02): Added strategy detection fields
- **v1.2** (2024-03): Added model parameter exports

---

## Contact

For questions about the data format:
- GitHub: https://github.com/emberian/abcdeez
- Email: ember@lunar.town
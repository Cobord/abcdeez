# OSF Pre-Registration Template
## Adaptive Training for Flexible Skill Acquisition: Building Navigable Mental Models of Graph-Structured Tasks

---

## 1. Study Information

### 1.1 Title
Adaptive Training for Flexible Skill Acquisition: Building Navigable Mental Models of Graph-Structured Tasks

### 1.2 Authors
[To be filled]

### 1.3 Research Questions
1. Does adaptive training using Expected Information Gain (EIG) lead to more flexible graph-coded mental models compared to linear training?
2. Do learners trained with the adaptive system show reduced symbolic distance effects in response times?
3. Does adaptive training reduce chunk boundary penalties in sequential recall tasks?
4. Do adaptively-trained learners show superior transfer to isomorphic domains?

### 1.4 Hypotheses

#### H1: Representational Shift
The Adaptive-GCM group will show a significantly greater decrease in the symbolic distance slope (λ₁ → 0) for RTs on pairwise comparison tasks compared to both control groups, indicating a shift from serial scanning to direct, indexed retrieval.

#### H2: Seam Smoothing
The Adaptive-GCM group will exhibit a significantly larger decrease in the chunk boundary penalty (δ_boundary) for both RT and accuracy, relative to control groups, suggesting successful integration of previously disparate "chunks" into a cohesive mental model.

#### H3: Operator Disentangling
The Adaptive-GCM group will demonstrate greater proficiency gains in complex operations (e.g., k-jump, reverse segment tasks), as measured by θ_o, and these gains will generalize across varying difficulties.

#### H4: Transfer
On a novel, isomorphic graph, the Adaptive-GCM group will demonstrate significantly faster initial learning and higher transfer efficiency compared to both control groups.

---

## 2. Design Plan

### 2.1 Study Type
**Experiment** - A researcher randomly assigns treatments to study subjects

### 2.2 Blinding
**Single blind** - Participants do not know which treatment group they are in

### 2.3 Study Design
Between-subjects design with three groups:
1. **Adaptive-GCM Group** (n=20-30): Full adaptive system with EIG-based task selection
2. **Non-Adaptive Linear Group** (n=20-30): Fixed linear traversal training
3. **Yoked Adaptive Control Group** (n=20-30): Receives same task sequence as matched Adaptive-GCM participant

### 2.4 Randomization
Participants will be randomly assigned to conditions using block randomization (blocks of 6) to ensure balanced group sizes throughout data collection.

---

## 3. Sampling Plan

### 3.1 Existing Data
**Registration prior to creation of data**

### 3.2 Sample Size
**Target N = 60-90 participants**

Based on power analysis (α=0.05, power=0.80, d=0.5):
- Minimum 20 per group for between-group comparisons
- Target 30 per group for robust effect estimation

### 3.3 Sample Size Rationale
Power analysis conducted using:
```
PowerAnalysis::new(0.05, 0.80, 0.5).calculate_sample_size_anova(3)
```
Indicates 60 participants minimum for detecting medium effect sizes in 3-group ANOVA.

### 3.4 Stopping Rule
Data collection will stop when:
1. Target sample size reached (N=90), OR
2. Pre-specified date reached [DATE], OR
3. Minimum sample (N=60) reached with significant results in interim analysis

---

## 4. Variables

### 4.1 Manipulated Variables
**Training Type** (between-subjects):
- Adaptive-GCM: EIG-optimized task selection
- Linear: Fixed forward sequence training
- Yoked: Matched task sequence without adaptation

### 4.2 Measured Variables

#### Primary Outcomes:
1. **Symbolic Distance Slope (λ₁)**: RT slope vs. item distance in pairwise comparisons
2. **Chunk Boundary Penalty (δ_boundary)**: RT/accuracy cost at chunk boundaries
3. **Operation Proficiency (θ_o)**: Accuracy on complex operations
4. **Transfer Efficiency**: Learning rate on isomorphic domain

#### Secondary Outcomes:
1. **Bidirectionality Index**: ΔRT_reverse-forward
2. **Wrap Penalty**: Cost for cyclic wraparound tasks
3. **Locality of Errors**: Error concentration by graph distance
4. **Strategy Shift Index (π_t)**: Proportion using direct vs. scanning strategy

#### Process Measures:
1. Response time (ms) per trial
2. Accuracy (correct/incorrect) per trial
3. Task difficulty level
4. Trial number

### 4.3 Indices
**Composite Measures:**
- Graph-Coded Mastery Score = weighted combination of primary outcomes
- Flexibility Index = average of bidirectionality and wrap penalty measures

---

## 5. Analysis Plan

### 5.1 Statistical Models
Hierarchical Bayesian models will be fit using Stan:
```
y ~ training_group + trial + (1 + trial | participant)
```

### 5.2 Transformations
- Response times: log transformation if skewed
- Accuracy: logit transformation for proportion data

### 5.3 Inference Criteria
- Bayesian: 95% credible intervals excluding zero
- Frequentist backup: p < 0.05 with multiple comparison corrections

### 5.4 Data Exclusion
Trial-level exclusions:
- RT < 200ms (anticipatory responses)
- RT > 10000ms (attention lapses)

Participant-level exclusions:
- Overall accuracy < 20% (non-engagement)
- Missing > 30% of trials

### 5.5 Missing Data
- Single imputation for isolated missing trials (<5%)
- Listwise deletion for systematic missingness

### 5.6 Exploratory Analyses
1. Individual differences in learning trajectories
2. Correlation between working memory span and chunk boundary effects
3. Strategy discovery timing analysis

---

## 6. Scripts and Software

### 6.1 Software Packages
- **Data Collection**: alphabet-terminal-prototype (Rust)
- **Statistical Analysis**: R with packages:
  - brms (Bayesian models)
  - tidyverse (data manipulation)
  - lme4 (mixed effects backup)

### 6.2 Code Availability
Analysis scripts will be available at: [GITHUB REPOSITORY URL]

### 6.3 Multiple Comparison Corrections
```rust
MultipleComparisonCorrection::new(CorrectionMethod::BenjaminiHochberg, 0.05)
```

---

## 7. Other Information

### 7.1 Ethics Approval
IRB approval pending from [INSTITUTION]
Protocol #: [NUMBER]

### 7.2 Data Management
- Raw data: JSON format, one file per participant
- Processed data: CSV with standardized column names
- Data dictionary: Provided in repository

### 7.3 Deviations from Pre-registration
Any deviations will be explicitly noted in the final manuscript with justification.

### 7.4 Timeline
- IRB Submission: [DATE]
- Pilot Testing: [DATE RANGE]
- Data Collection: [DATE RANGE]
- Analysis Completion: [DATE]

---

## 8. References

[Include key references from PAPER.md]

---

## Pre-Registration Confirmation

By submitting this pre-registration, we commit to:
1. Following the specified analysis plan
2. Reporting all pre-registered analyses
3. Clearly distinguishing exploratory from confirmatory analyses
4. Making data and code publicly available upon publication

**Date Submitted**: [DATE]
**OSF Project**: [URL]
**Registration DOI**: [To be assigned]
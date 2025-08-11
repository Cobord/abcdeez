# Statistical Analysis Architecture

## Requirements and Dataflow

### Core Requirements
- Comprehensive descriptive statistics with robust implementations
- Specialized response time analysis using Ex-Gaussian distribution modeling
- Multiple outlier detection methods with different sensitivity levels
- Statistical hypothesis testing for educational research validation
- Bootstrap confidence intervals for non-parametric uncertainty quantification
- Cognitive modeling through response time distribution analysis

### Data Flow Patterns
1. **Descriptive Analysis**: Dataset → Validation → Statistical Computation → Summary Statistics → Report
2. **Hypothesis Testing**: Groups → Assumptions Check → Test Statistic → P-value Calculation → Significance Assessment
3. **Outlier Detection**: Data → Multiple Methods → Consensus Analysis → Outlier Classification → Cleaned Dataset
4. **Bootstrap Analysis**: Sample → Resampling → Statistic Calculation → Distribution → Confidence Interval
5. **Response Time Modeling**: Times → Distribution Fitting → Parameter Estimation → Model Validation → Insights

## High-level Purpose and Responsibilities

### Primary Purpose
Provides comprehensive statistical analysis capabilities specifically designed for educational research, learning analytics, and cognitive modeling with emphasis on response time analysis and robust statistical inference.

### Core Responsibilities
- **Descriptive Statistics**: Central tendency, dispersion, and shape measures with numerical stability
- **Educational Research**: Specialized statistical tests for learning outcome analysis
- **Response Time Analysis**: Ex-Gaussian distribution fitting for cognitive load assessment
- **Outlier Management**: Multiple detection methods for data quality assurance
- **Hypothesis Testing**: Parametric and non-parametric tests for experimental validation
- **Robust Inference**: Bootstrap methods for distribution-free confidence intervals

## Key Abstractions and Interfaces

### Descriptive Statistics
- **mean(), median(), mode()**: Central tendency measures with edge case handling
- **variance(), std_dev()**: Dispersion measures with proper bias correction
- **skewness(), kurtosis()**: Distribution shape analysis with sample size adjustments
- **percentile(), iqr()**: Quantile-based statistics for robust analysis

### Specialized Educational Analysis
- **ExGaussianParams**: Response time distribution parameters for cognitive modeling
- **fit_ex_gaussian()**: Method-of-moments parameter estimation for response times
- **ex_gaussian_pdf()**: Probability density function for cognitive load analysis
- **ResponseTimeAnalysis**: Comprehensive response time modeling with uncertainty quantification

### Hypothesis Testing Framework
- **TTestResult**: Two-sample comparison with Welch's correction for unequal variances
- **AnovaResult**: One-way ANOVA with F-statistic and multiple group analysis
- **t_test_two_sample()**: Robust two-sample comparison with proper degrees of freedom
- **one_way_anova()**: Multiple group comparison with post-hoc analysis support

### Outlier Detection System
- **OutlierAnalysis**: Multi-method outlier detection with consensus reporting
- **outliers_iqr()**: Interquartile range method for robust outlier identification
- **outliers_z_score()**: Standard score method for normal distribution assumptions
- **outliers_modified_z_score()**: Median-based method for non-normal distributions

## Data Transformations and Flow

### Statistical Analysis Pipeline
```
Raw Data → Quality Check → Outlier Detection → Distribution Analysis → Statistical Testing → Interpretation
```

### Ex-Gaussian Modeling Flow
```
Response Times → Moment Calculation → Parameter Estimation → Model Validation → Cognitive Insights
```

### Hypothesis Testing Process
```
Group Data → Assumption Checking → Test Selection → Statistic Calculation → P-value → Significance
```

### Bootstrap Confidence Interval Generation
```
Original Sample → Resampling → Statistic Calculation → Bootstrap Distribution → Confidence Interval
```

## Dependencies and Interactions

### External Dependencies
- **statrs**: Professional statistical distributions (Student's t, F-distribution) for hypothesis testing
- **rand**: Random number generation for bootstrap resampling and Monte Carlo methods
- **serde**: Serialization support for statistical results and model parameters

### Internal System Interactions
- **Math Module**: Utilizes safe mathematical operations for numerical stability
- **Learning Analytics**: Response time modeling for cognitive load assessment
- **Research Platform**: Hypothesis testing for experimental validation
- **Adaptive Systems**: Statistical validation of learning algorithm effectiveness

## Architectural Patterns

### Robust Statistical Computing
- Multiple outlier detection methods provide consensus-based data quality assessment
- Bootstrap confidence intervals offer distribution-free uncertainty quantification
- Ex-Gaussian modeling specifically addresses cognitive response time distributions
- Comprehensive input validation ensures statistical assumptions are met

### Educational Research Specialization
- Ex-Gaussian distribution modeling designed for cognitive task response times
- Statistical tests appropriate for educational research sample sizes and assumptions
- Response time analysis framework for learning efficiency assessment
- Cognitive load inference through distributional parameter interpretation

### Methodological Rigor
- Proper statistical test selection with assumption checking and validation
- Multiple comparison corrections for experimental research validity
- Bootstrap methods for robust confidence intervals without distributional assumptions
- Comprehensive outlier analysis using established psychometric methods

### Performance and Scalability
- Efficient algorithms for large educational datasets with O(n log n) complexity
- Memory-conscious implementations for bootstrap and permutation procedures
- Specialized approximations for statistical distributions and special functions
- Vectorized operations for improved computational performance on large samples

### Quality Assurance Framework
- Extensive test coverage for statistical accuracy and edge case handling
- Numerical precision validation for floating-point statistical computations
- Cross-validation of results against established statistical software packages
- Comprehensive error handling for invalid inputs and degenerate cases
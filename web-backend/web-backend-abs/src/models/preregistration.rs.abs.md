# Pre-registration Models Architecture

## Requirements and Dataflow

### Core Requirements
- Scientific research transparency through pre-registration of studies
- Immutable registration records with cryptographic hashing
- Structured hypothesis specification with statistical validation
- Analysis plan documentation with power calculations
- Deviation tracking and impact assessment
- Transparency reporting for regulatory compliance

### Data Flow Patterns
1. **Registration Creation**: CreatePreRegistration → Validation → Hash Generation → PreRegistrationDb
2. **Draft Updates**: UpdatePreRegistration → Status Check → Versioning → Database Update
3. **Registration Finalization**: Draft → Validation → Hash Locking → Immutable Record
4. **Deviation Tracking**: Runtime Analysis → Pre-registration Comparison → Deviation Record → Impact Assessment
5. **Transparency Generation**: PreRegistration → Deviation Analysis → TransparencyReport → Public Record

## High-level Purpose and Responsibilities

### Primary Purpose
Implements a comprehensive pre-registration system that ensures scientific rigor and transparency by requiring researchers to specify hypotheses, analysis plans, and data collection procedures before conducting studies.

### Core Responsibilities
- **Registration Management**: Create, update, and finalize research pre-registrations
- **Hypothesis Framework**: Structure primary and secondary hypotheses with statistical specifications
- **Analysis Planning**: Document planned analyses, power calculations, and robustness checks
- **Data Collection Protocol**: Define sampling, randomization, and stopping rules
- **Deviation Tracking**: Monitor and document departures from original plans
- **Transparency Reporting**: Generate compliance reports for regulatory bodies

## Key Abstractions and Interfaces

### Core Entities
- **PreRegistrationDb**: Persistent storage model with JSON fields and versioning
- **CreatePreRegistration**: API input model for new registrations
- **UpdatePreRegistration**: API input model for draft modifications
- **PreRegistrationResponse**: API output model with editing permissions

### Research Framework
- **StudyMetadata**: Researcher, institutional, and ethical approval information
- **Hypotheses**: Primary and secondary hypotheses with effect predictions
- **AnalysisPlan**: Statistical models, power analysis, and robustness checks
- **DataCollectionPlan**: Sampling, randomization, and quality control procedures

### Validation and Compliance
- **PreRegistrationDeviation**: Documented departures from original plans
- **AnalysisValidation**: Comparison between planned and actual analyses
- **TransparencyReport**: Public compliance report with deviation summaries

## Data Transformations and Flow

### Registration Lifecycle
```
Draft Creation → Iterative Updates → Validation → Hash Generation → Immutable Lock
```

### Hypothesis Specification
```
Hypothesis Description → Operationalization → Effect Prediction → Statistical Test → Alpha Level
```

### Analysis Planning
```
Research Question → Statistical Model → Power Analysis → Sample Size → Robustness Checks
```

### Deviation Management
```
Runtime Analysis → Pre-registration Comparison → Deviation Detection → Justification → Impact Assessment
```

## Dependencies and Interactions

### External Dependencies
- **chrono**: Timestamp management for registration and deviation tracking
- **serde**: JSON serialization for complex nested structures
- **sqlx**: Database persistence with FromRow mapping
- **std::collections**: HashMap for interpretation guidelines

### Internal System Interactions
- **Handlers**: Pre-registration API endpoints consume these models for CRUD operations
- **Services**: Validation and hashing services use these models for business logic
- **Database**: Complex JSON fields store structured research plans
- **Analytics**: Transparency reports integrate with compliance monitoring systems

## Architectural Patterns

### Immutability by Design
- Registration hash prevents tampering after finalization
- Version tracking for draft updates
- Deviation records maintain audit trail
- Parent-child relationships for registration amendments

### Scientific Rigor
- Structured hypothesis specification with effect size predictions
- Power analysis integration with sample size calculations
- Multiple comparison correction procedures
- Robustness check specifications for assumption validation

### Compliance Framework
- Ethical approval tracking and documentation
- Conflict of interest declarations
- Funding source transparency
- Institutional affiliation requirements

### Flexibility with Structure
- Tagged enum patterns for effect predictions and stopping rules
- Optional fields for adaptive study designs
- Extensible metadata structures via JSON storage
- Multiple analysis plan support (primary/secondary)

### Transparency Mechanisms
- Public transparency reports with deviation summaries
- Analysis validation comparing planned vs. actual procedures
- Timestamped deviation records with justifications
- Registration hash verification for authenticity
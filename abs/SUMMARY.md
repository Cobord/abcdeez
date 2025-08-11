# Summary

[Introduction](./README.md)

# Core Module

- [Core](./core/src/lib.rs.abs.md)
  - [Backend](./core/src/core/backend.rs.abs.md)
  - [Configuration](./core/src/core/config.rs.abs.md)
  - [Topology](./core/src/core/topology.rs.abs.md)
  - [Error Handling](./core/src/core/error.rs.abs.md)

## Learning System

- [Learning](./core/src/learning/mod.rs.abs.md)
  - [Adaptive Learning](./core/src/learning/adaptive.rs.abs.md)
  - [Bayesian Methods](./core/src/learning/bayesian.rs.abs.md)
  - [Hierarchical Bayes](./core/src/learning/hierarchical_bayes.rs.abs.md)
  - [Learner Model](./core/src/learning/learner.rs.abs.md)
  - [Macro Learning](./core/src/learning/macro_learning.rs.abs.md)
  - [Strategy Mixture](./core/src/learning/strategy_mixture.rs.abs.md)
  - [Transfer Learning](./core/src/learning/transfer_learning.rs.abs.md)

## Tasks

- [Tasks](./core/src/tasks/mod.rs.abs.md)
  - [Core Tasks](./core/src/tasks/core.rs.abs.md)
  - [Extended Tasks](./core/src/tasks/extended.rs.abs.md)
  - [Music Tasks](./core/src/tasks/music.rs.abs.md)
  - [Navigation Tasks](./core/src/tasks/navigation.rs.abs.md)
  - [Boundaries](./core/src/tasks/boundaries.rs.abs.md)

## Experiments

- [Experiments](./core/src/experiments/mod.rs.abs.md)
  - [Core Experiments](./core/src/experiments/core.rs.abs.md)
  - [Design](./core/src/experiments/design.rs.abs.md)
  - [A/B Testing](./core/src/experiments/ab_testing.rs.abs.md)
  - [Multi-Session](./core/src/experiments/multi_session.rs.abs.md)

## Statistics

- [Statistics](./core/src/statistics/mod.rs.abs.md)
  - [Core Statistics](./core/src/statistics/core.rs.abs.md)
  - [Math Validation](./core/src/statistics/math_validation.rs.abs.md)
  - [Mixed Effects](./core/src/statistics/mixed_effects.rs.abs.md)
  - [Power Analysis](./core/src/statistics/power_analysis.rs.abs.md)
  - [Prediction](./core/src/statistics/prediction.rs.abs.md)
  - [Validation](./core/src/statistics/validation.rs.abs.md)

## Data Management

- [Data](./core/src/data/mod.rs.abs.md)
  - [Interaction Tracking](./core/src/data/interaction_tracking.rs.abs.md)
  - [Performance Tracing](./core/src/data/performance_tracing.rs.abs.md)
  - [Audio Recording](./core/src/data/audio_recording.rs.abs.md)
  - [Sensor Integration](./core/src/data/sensor_integration.rs.abs.md)
  - [Export](./core/src/data/export.rs.abs.md)

## Compliance

- [Compliance](./core/src/compliance/mod.rs.abs.md)
  - [Audit Trail](./core/src/compliance/audit_trail.rs.abs.md)
  - [Citation Manager](./core/src/compliance/citation_manager.rs.abs.md)
  - [IRB](./core/src/compliance/irb.rs.abs.md)
  - [Preregistration](./core/src/compliance/preregistration.rs.abs.md)

## Protocol

- [Protocol](./core/src/protocol/mod.rs.abs.md)
  - [Seed Management](./core/src/protocol/seed_management.rs.abs.md)
  - [Version Control](./core/src/protocol/version_control.rs.abs.md)
  - [Versioning](./core/src/protocol/versioning.rs.abs.md)

# Web Backend

- [Web Backend](./web-backend/src/lib.rs.abs.md)
  - [Configuration](./web-backend/src/config.rs.abs.md)
  - [State Management](./web-backend/src/state.rs.abs.md)
  - [Database](./web-backend/src/db.rs.abs.md)
  - [Cache](./web-backend/src/cache.rs.abs.md)
  - [TLS](./web-backend/src/tls.rs.abs.md)
  - [Error Handling](./web-backend/src/error.rs.abs.md)

## Handlers

- [Handlers](./web-backend/src/handlers/mod.rs.abs.md)
  - [Authentication](./web-backend/src/handlers/auth.rs.abs.md)
  - [Admin](./web-backend/src/handlers/admin.rs.abs.md)
  - [Dashboard](./web-backend/src/handlers/dashboard.rs.abs.md)
  - [Session Management](./web-backend/src/handlers/session.rs.abs.md)
  - [Task Handlers](./web-backend/src/handlers/task.rs.abs.md)
  - [Experiment Handlers](./web-backend/src/handlers/experiment.rs.abs.md)
  - [Analytics](./web-backend/src/handlers/analytics.rs.abs.md)

## Models

- [Models](./web-backend/src/models/mod.rs.abs.md)
  - [User](./web-backend/src/models/user.rs.abs.md)
  - [Session](./web-backend/src/models/session.rs.abs.md)
  - [Learner](./web-backend/src/models/learner.rs.abs.md)
  - [Protocol](./web-backend/src/models/protocol.rs.abs.md)
  - [Experiment](./web-backend/src/models/experiment.rs.abs.md)

## Monitoring

- [Monitoring](./web-backend/src/monitoring/mod.rs.abs.md)
  - [Health Checks](./web-backend/src/monitoring/health.rs.abs.md)
  - [Metrics](./web-backend/src/monitoring/metrics.rs.abs.md)
  - [Performance](./web-backend/src/monitoring/performance.rs.abs.md)
  - [OpenTelemetry](./web-backend/src/monitoring/otel.rs.abs.md)

# Xilem Application

- [Xilem App](./xilem-app/README.md)
  - [Application](./xilem-app/src/app.rs.abs.md)
  - [Main](./xilem-app/src/main.rs.abs.md)

## Components

- [Components](./xilem-app/src/components/mod.rs.abs.md)

## Services

- [Services](./xilem-app/src/services/mod.rs.abs.md)
  - [API Client](./xilem-app/src/services/api.rs.abs.md)
  - [Adaptive Learning](./xilem-app/src/services/adaptive_learning.rs.abs.md)
  - [WebSocket](./xilem-app/src/services/websocket.rs.abs.md)
  - [Storage](./xilem-app/src/services/storage.rs.abs.md)
  - [Sync](./xilem-app/src/services/sync.rs.abs.md)

## Views

- [Views](./xilem-app/src/views/mod.rs.abs.md)
  - [Authentication](./xilem-app/src/views/auth.rs.abs.md)
  - [Dashboard](./xilem-app/src/views/dashboard.rs.abs.md)
  - [Core Task Visual](./xilem-app/src/views/core_task_visual.rs.abs.md)
  - [Extended Task Visual](./xilem-app/src/views/extended_task_visual.rs.abs.md)
  - [Learning](./xilem-app/src/views/learning.rs.abs.md)
  - [Settings](./xilem-app/src/views/settings.rs.abs.md)
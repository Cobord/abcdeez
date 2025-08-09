"""
abcdeez - Python client for the Graph Learning System

A comprehensive client library for interacting with the Graph Learning backend API.
"""

__version__ = "0.1.0"

from .client import GraphLearningClient
from .models import (
    User,
    Session,
    Task,
    TaskResponse,
    Analytics,
    LearnerModel,
    SessionConfig,
)
from .exceptions import (
    AbcdeezError,
    AuthenticationError,
    RateLimitError,
    ValidationError,
    NetworkError,
)
from .analysis import (
    analyze_session,
    compute_learning_curve,
    plot_performance,
    export_data,
)

__all__ = [
    "GraphLearningClient",
    "User",
    "Session",
    "Task",
    "TaskResponse",
    "Analytics",
    "LearnerModel",
    "SessionConfig",
    "AbcdeezError",
    "AuthenticationError",
    "RateLimitError",
    "ValidationError",
    "NetworkError",
    "analyze_session",
    "compute_learning_curve",
    "plot_performance",
    "export_data",
]
"""
Data models for the Graph Learning System.
"""

from typing import Optional, List, Dict, Any
from datetime import datetime
from pydantic import BaseModel, Field, ConfigDict


class User(BaseModel):
    """User model."""
    model_config = ConfigDict(populate_by_name=True)
    
    user_id: str
    email: str
    display_name: str
    created_at: datetime
    updated_at: Optional[datetime] = None


class SessionConfig(BaseModel):
    """Session configuration."""
    difficulty: float = Field(default=0.5, ge=0.0, le=1.0)
    hint_probability: float = Field(default=0.1, ge=0.0, le=1.0)
    max_trials: int = Field(default=100, ge=1)
    adaptive: bool = True
    randomize: bool = True


class Session(BaseModel):
    """Learning session model."""
    model_config = ConfigDict(populate_by_name=True)
    
    session_id: str
    domain: str
    config: SessionConfig
    started_at: datetime
    ended_at: Optional[datetime] = None
    status: str  # active, completed, abandoned
    total_trials: Optional[int] = None
    correct_trials: Optional[int] = None
    average_rt: Optional[float] = None
    metrics: Optional[Dict[str, Any]] = None


class Task(BaseModel):
    """Task model."""
    task_id: str
    session_id: str
    stimulus: str
    choices: List[str]
    task_type: str
    difficulty: float
    hint: Optional[str] = None


class TaskResponse(BaseModel):
    """Task response model."""
    task_id: str
    response: str
    response_time: float
    correct: Optional[bool] = None
    client_timestamp: datetime


class Trial(BaseModel):
    """Trial data model."""
    trial_id: str
    session_id: str
    task_id: str
    stimulus: str
    response: str
    correct: bool
    response_time: float
    difficulty: float
    hint_used: bool
    timestamp: datetime


class Analytics(BaseModel):
    """Analytics data model."""
    user_id: str
    period: Dict[str, datetime]
    summary: Dict[str, Any]
    time_series: List[Dict[str, Any]]
    domain_breakdown: Dict[str, Dict[str, Any]]


class LearnerModel(BaseModel):
    """Learner model parameters."""
    user_id: str
    domain: str
    parameters: Dict[str, float]
    updated_at: datetime
    confidence: float
    predictions: Optional[Dict[str, float]] = None


class BatchTrialSubmission(BaseModel):
    """Batch trial submission model."""
    trials: List[Dict[str, Any]]
    client_info: Dict[str, Any]


class ExportRequest(BaseModel):
    """Data export request model."""
    format: str = Field(default="json", pattern="^(json|csv|parquet)$")
    include: List[str] = Field(default=["sessions", "trials", "analytics"])
    from_date: Optional[datetime] = None
    to_date: Optional[datetime] = None
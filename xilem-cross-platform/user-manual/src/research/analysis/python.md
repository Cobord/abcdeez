# Python Integration

The abcdeez Python client library provides comprehensive programmatic access to the Adaptive Learning System for data collection, analysis, and research workflows.

## Installation

### Basic Installation
```bash
pip install abcdeez
```

### Development Installation
```bash
# For async support
pip install abcdeez[async]

# For development
pip install abcdeez[dev]

# For analysis extras (pandas, scipy, matplotlib)
pip install abcdeez[analysis]
```

## Quick Start

### Basic Authentication and Session Management
```python
from abcdeez import GraphLearningClient
from abcdeez.models import SessionConfig

# Initialize client
client = GraphLearningClient(base_url="https://abcdeez.fg-goose.online/api/v1")

# Register new user or login
user = client.register("researcher@university.edu", "secure_password", "Dr. Jane Researcher")
# Or login to existing account
user = client.login("researcher@university.edu", "secure_password")

# Create learning session with configuration
config = SessionConfig(
    difficulty=0.5,
    hint_probability=0.1,
    max_trials=200,
    domain_specific_params={'adaptive_scheduling': True}
)
session = client.create_session("alphabet", config)

print(f"Created session: {session.session_id}")
```

### Data Collection
```python
# Run interactive learning session
def collect_session_data(client, session_id, n_trials=50):
    responses = []
    
    for trial in range(n_trials):
        # Get next adaptive task
        task = client.get_next_task(session_id)
        
        # Present task to participant (your experiment code here)
        print(f"Task {trial+1}: {task.stimulus}")
        print(f"Choices: {task.choices}")
        
        # Simulate or collect actual response
        response = input("Response: ")
        start_time = time.time()
        # ... present task to participant ...
        response_time = (time.time() - start_time) * 1000  # ms
        
        # Submit response
        result = client.submit_response(
            session_id,
            task.task_id,
            response=response,
            response_time=response_time
        )
        
        responses.append({
            'trial': trial + 1,
            'stimulus': task.stimulus,
            'response': response,
            'correct': result['correct'],
            'rt_ms': response_time
        })
        
        print(f"Correct: {result['correct']}")
    
    return responses

# Collect data
session_data = collect_session_data(client, session.session_id)
```

### Batch Operations for Offline Studies
```python
# Collect trials offline, then batch submit
def batch_experiment(client, session_id, offline_data):
    """
    Submit pre-collected experimental data in batches.
    Useful for offline studies or imported data.
    """
    trials = []
    
    for trial_data in offline_data:
        trials.append({
            "stimulus": trial_data['stimulus'],
            "response": trial_data['response'], 
            "response_time": trial_data['rt_ms'],
            "correct": trial_data['correct'],
            "client_timestamp": trial_data['timestamp'].isoformat(),
            "trial_metadata": {
                "condition": trial_data.get('condition'),
                "block": trial_data.get('block')
            }
        })
    
    # Submit in batches of 100
    for i in range(0, len(trials), 100):
        batch = trials[i:i+100]
        result = client.batch_submit_trials(session_id, batch)
        print(f"Batch {i//100 + 1}: {result['accepted']}/{len(batch)} trials accepted")

# Usage
batch_experiment(client, session.session_id, your_offline_data)
```

## Data Analysis

### Export and Basic Analysis
```python
import pandas as pd
import numpy as np
from abcdeez.analysis import analyze_session, plot_performance

# Export session data
data = client.export_data(
    format="json",
    include=["sessions", "trials", "analytics"],
    from_date=datetime(2024, 1, 1),
    to_date=datetime.now()
)

# Convert to DataFrame
df = pd.DataFrame(data['trials'])

# Basic descriptive statistics
print("Session Summary:")
print(f"Total trials: {len(df)}")
print(f"Overall accuracy: {df['correct'].mean():.3f}")
print(f"Mean RT: {df['response_time_ms'].mean():.1f}ms")
print(f"Median RT: {df['response_time_ms'].median():.1f}ms")

# Accuracy by trial block
df['trial_block'] = pd.cut(df['trial_number'], bins=10)
accuracy_by_block = df.groupby('trial_block')['correct'].mean()
print("\nLearning curve (accuracy by trial block):")
print(accuracy_by_block)
```

### Advanced Analytics
```python
from abcdeez.analysis import (
    calculate_learning_curves,
    detect_strategy_shifts,
    analyze_symbolic_distance,
    compute_transfer_metrics
)

# Learning curve analysis
learning_curves = calculate_learning_curves(df, window_size=20)

# Strategy detection (serial scan vs. direct access)
strategy_analysis = detect_strategy_shifts(
    df[df['task_type'] == 'PairwiseOrder'],
    window_size=50
)

print(f"Detected strategy shifts: {len(strategy_analysis['shift_points'])}")
print(f"Final strategy: {strategy_analysis['final_strategy']}")

# Symbolic distance effect analysis
if 'symbolic_distance' in df.columns:
    distance_effect = analyze_symbolic_distance(df)
    print(f"Distance effect slope: {distance_effect['slope']:.4f}")
    print(f"R² = {distance_effect['r_squared']:.3f}")

# Transfer learning metrics
transfer_metrics = compute_transfer_metrics(df, baseline_domain='alphabet')
print(f"Transfer efficiency: {transfer_metrics['efficiency']:.3f}")
```

### Visualization
```python
import matplotlib.pyplot as plt
import seaborn as sns
from abcdeez.visualization import (
    plot_learning_curve,
    plot_rt_distributions,
    plot_strategy_evolution,
    plot_distance_effects
)

# Learning curve with confidence intervals
fig, ax = plt.subplots(1, 1, figsize=(10, 6))
plot_learning_curve(df, ax=ax, confidence_interval=True)
ax.set_title('Learning Curve - Accuracy Over Trials')
plt.show()

# Response time distributions by condition
plot_rt_distributions(df, group_by='task_type', log_scale=True)

# Strategy evolution over time
if strategy_analysis['temporal_data']:
    plot_strategy_evolution(strategy_analysis['temporal_data'])

# Distance effects (if applicable)
pairwise_data = df[df['task_type'] == 'PairwiseOrder']
if not pairwise_data.empty:
    plot_distance_effects(pairwise_data, phases=['early', 'middle', 'late'])
```

## Async Client for High-Throughput Studies

```python
import asyncio
from abcdeez.async_client import AsyncGraphLearningClient

async def run_concurrent_sessions():
    """Run multiple sessions concurrently for large-scale studies."""
    
    async with AsyncGraphLearningClient() as client:
        await client.login("researcher@university.edu", "password")
        
        # Create multiple sessions for different conditions
        sessions = await asyncio.gather(
            client.create_session("alphabet", SessionConfig(difficulty=0.3)),
            client.create_session("alphabet", SessionConfig(difficulty=0.5)),
            client.create_session("alphabet", SessionConfig(difficulty=0.7)),
            client.create_session("music", SessionConfig(difficulty=0.5))
        )
        
        # Process sessions concurrently
        async def run_session(session, n_trials=100):
            responses = []
            for _ in range(n_trials):
                task = await client.get_next_task(session.session_id)
                # Simulate automated response or connect to experiment software
                response = simulate_participant_response(task)
                result = await client.submit_response(
                    session.session_id,
                    task.task_id,
                    response['answer'],
                    response['rt_ms']
                )
                responses.append(result)
            return responses
        
        # Run all sessions concurrently
        all_results = await asyncio.gather(
            *[run_session(session) for session in sessions]
        )
        
        return all_results

# Execute concurrent sessions
results = asyncio.run(run_concurrent_sessions())
```

## Command Line Interface

The abcdeez package includes a CLI for common research workflows:

```bash
# Login and store credentials
abcdeez login --email researcher@university.edu

# Create and run training session
abcdeez session create --domain alphabet --difficulty 0.5 --trials 200

# Interactive training mode
abcdeez train --domain music --adaptive true

# Export data for analysis
abcdeez export --format csv --output study1_data.csv --from-date 2024-01-01

# View analytics dashboard in terminal
abcdeez analytics --period month --domain alphabet

# Generate analysis report
abcdeez report --input study1_data.csv --output study1_report.pdf
```

## Integration with Experimental Software

### PsychoPy Integration
```python
from psychopy import visual, core, event
from abcdeez import GraphLearningClient

# Initialize PsychoPy and abcdeez
win = visual.Window([800, 600])
client = GraphLearningClient()
client.login("researcher@university.edu", "password")
session = client.create_session("alphabet")

def run_psychopy_trial(task):
    # Display stimulus
    stimulus_text = visual.TextStim(win, text=task.stimulus)
    choice_texts = [visual.TextStim(win, text=choice, pos=(i*200-200, -100)) 
                   for i, choice in enumerate(task.choices)]
    
    # Present stimuli
    stimulus_text.draw()
    for choice_text in choice_texts:
        choice_text.draw()
    win.flip()
    
    # Collect response
    start_time = core.getTime()
    keys = event.waitKeys(keyList=['1', '2', '3', '4'])
    rt_ms = (core.getTime() - start_time) * 1000
    
    response = task.choices[int(keys[0]) - 1]
    
    return response, rt_ms

# Run experiment
for trial in range(100):
    task = client.get_next_task(session.session_id)
    response, rt_ms = run_psychopy_trial(task)
    client.submit_response(session.session_id, task.task_id, response, rt_ms)
```

### jsPsych Integration
```python
# Export session configuration for web-based experiments
def export_jspsych_config(client, domain, n_trials=200):
    session = client.create_session(domain)
    
    tasks = []
    for _ in range(n_trials):
        task = client.get_next_task(session.session_id)
        tasks.append({
            'stimulus': task.stimulus,
            'choices': task.choices,
            'task_id': task.task_id,
            'session_id': session.session_id
        })
    
    return {
        'session_id': session.session_id,
        'tasks': tasks,
        'submit_url': f"{client.base_url}/sessions/{session.session_id}/tasks/"
    }

# Generate configuration for jsPsych
config = export_jspsych_config(client, "alphabet", 150)

# Save for use in web experiment
import json
with open('experiment_config.json', 'w') as f:
    json.dump(config, f, indent=2)
```

## Error Handling and Best Practices

```python
from abcdeez.exceptions import (
    AbcdeezError,
    AuthenticationError,
    RateLimitError,
    ValidationError,
    NetworkError
)

def robust_data_collection(client, session_id, max_trials=200):
    """Example of robust data collection with error handling."""
    
    collected_trials = 0
    max_retries = 3
    
    while collected_trials < max_trials:
        try:
            # Get next task
            task = client.get_next_task(session_id)
            
            # Present task (your experiment code here)
            response, rt_ms = present_task_to_participant(task)
            
            # Submit with retry logic
            for attempt in range(max_retries):
                try:
                    result = client.submit_response(
                        session_id, task.task_id, response, rt_ms
                    )
                    collected_trials += 1
                    print(f"Trial {collected_trials}/{max_trials} completed")
                    break
                    
                except RateLimitError as e:
                    print(f"Rate limited, waiting {e.retry_after} seconds...")
                    time.sleep(e.retry_after)
                    
                except NetworkError:
                    if attempt < max_retries - 1:
                        print(f"Network error, retrying in {2**attempt} seconds...")
                        time.sleep(2**attempt)
                    else:
                        print("Max retries exceeded, skipping trial")
                        break
                        
        except ValidationError as e:
            print(f"Validation error: {e}. Check your input data.")
            break
            
        except AuthenticationError:
            print("Authentication failed. Please log in again.")
            client.login("researcher@university.edu", "password")
            
    return collected_trials

# Usage with error handling
trials_completed = robust_data_collection(client, session.session_id)
print(f"Successfully completed {trials_completed} trials")
```

## Configuration and Environment Setup

### Environment Variables
```bash
# Set default configuration
export ABCDEEZ_API_URL="https://abcdeez.fg-goose.online/api/v1"
export ABCDEEZ_API_KEY="your-research-api-key"
export ABCDEEZ_TIMEOUT=60
export ABCDEEZ_RETRY_COUNT=5
```

### Configuration File
Create `~/.abcdeez/config.json`:
```json
{
  "api_url": "https://abcdeez.fg-goose.online/api/v1",
  "api_key": "your-research-api-key",
  "timeout": 60,
  "retry_count": 5,
  "default_session_config": {
    "max_trials": 200,
    "hint_probability": 0.1,
    "adaptive_difficulty": true
  },
  "analysis_settings": {
    "learning_curve_window": 20,
    "strategy_detection_window": 50,
    "outlier_threshold": 3.0
  }
}
```

## API Reference Summary

### Core Methods
- `register(email, password, display_name)` - Create new researcher account
- `login(email, password)` - Authenticate
- `create_session(domain, config)` - Start new learning session
- `get_next_task(session_id)` - Get adaptive task
- `submit_response(session_id, task_id, response, rt_ms)` - Submit response
- `batch_submit_trials(session_id, trials)` - Bulk data submission
- `export_data(format, include, from_date, to_date)` - Export research data

### Analytics Methods
- `get_analytics(user_id, from_date, to_date, domain)` - Detailed analytics
- `get_leaderboard(domain, metric, period)` - Population comparison
- `get_sessions(status, domain, date_range)` - Session management

### Models
- `User` - Researcher/participant account
- `Session` - Learning session with configuration
- `Task` - Adaptive learning task
- `TaskResponse` - Response data with metadata
- `Analytics` - Comprehensive learning metrics
- `SessionConfig` - Experimental parameters

For complete API documentation, see the [REST Endpoints](../api/endpoints.md) section.
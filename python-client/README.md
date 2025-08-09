# abcdeez Python Client

Official Python client library for the Graph Learning System (abcdeez).

## Installation

```bash
pip install abcdeez
```

For async support:
```bash
pip install abcdeez[async]
```

For development:
```bash
pip install abcdeez[dev]
```

## Quick Start

```python
from abcdeez import GraphLearningClient
from abcdeez.models import SessionConfig

# Initialize client
client = GraphLearningClient(base_url="https://api.graphlearning.app/v1")

# Register and login
user = client.register("user@example.com", "password", "John Doe")
# Or login to existing account
user = client.login("user@example.com", "password")

# Create a learning session
config = SessionConfig(difficulty=0.5, hint_probability=0.1, max_trials=100)
session = client.create_session("alphabet", config)

# Get next task
task = client.get_next_task(session.session_id)
print(f"Stimulus: {task.stimulus}")
print(f"Choices: {task.choices}")

# Submit response
result = client.submit_response(
    session.session_id,
    task.task_id,
    response="B",
    response_time=1.234
)
print(f"Correct: {result['correct']}")

# Get analytics
analytics = client.get_analytics()
print(f"Total sessions: {analytics.summary['total_sessions']}")
print(f"Average accuracy: {analytics.summary['average_accuracy']}")

# Export data
data = client.export_data(format="json")
```

## Advanced Usage

### Batch Trial Submission

For offline data collection:

```python
trials = []
for i in range(10):
    task = client.get_next_task(session.session_id)
    # Simulate response
    response = task.choices[0]
    response_time = 1.0 + i * 0.1
    
    trials.append({
        "task_id": task.task_id,
        "stimulus": task.stimulus,
        "response": response,
        "response_time": response_time,
        "correct": True,
        "client_timestamp": datetime.now().isoformat()
    })

# Submit all trials at once
result = client.batch_submit_trials(session.session_id, trials)
print(f"Accepted: {result['accepted']} trials")
```

### Data Analysis

```python
from abcdeez.analysis import analyze_session, plot_performance
import pandas as pd

# Analyze session
analysis = analyze_session(client, session.session_id)
print(f"Learning rate: {analysis['learning_rate']}")
print(f"Final accuracy: {analysis['final_accuracy']}")

# Plot performance
plot_performance(client, session.session_id, save_path="performance.png")

# Export to pandas DataFrame
sessions = client.get_sessions()
df = pd.DataFrame([s.dict() for s in sessions])
print(df.describe())
```

### Async Client

```python
import asyncio
from abcdeez.async_client import AsyncGraphLearningClient

async def main():
    async with AsyncGraphLearningClient() as client:
        await client.login("user@example.com", "password")
        
        # Create multiple sessions concurrently
        sessions = await asyncio.gather(
            client.create_session("alphabet"),
            client.create_session("numbers"),
            client.create_session("music")
        )
        
        # Process tasks
        for session in sessions:
            task = await client.get_next_task(session.session_id)
            await client.submit_response(
                session.session_id,
                task.task_id,
                response="A",
                response_time=1.0
            )

asyncio.run(main())
```

### Context Manager

```python
with GraphLearningClient() as client:
    client.login("user@example.com", "password")
    sessions = client.get_sessions()
    # Client automatically closes connection on exit
```

## Command Line Interface

```bash
# Login
abcdeez login --email user@example.com

# Create session
abcdeez session create --domain alphabet --difficulty 0.5

# Interactive training
abcdeez train --domain alphabet

# Export data
abcdeez export --format csv --output data.csv

# View analytics
abcdeez analytics --period week
```

## API Reference

### Client Methods

- `register(email, password, display_name)` - Create new account
- `login(email, password)` - Authenticate user
- `logout()` - Clear authentication
- `create_session(domain, config)` - Start learning session
- `get_sessions(limit, offset, status, domain, from_date, to_date)` - List sessions
- `get_session(session_id)` - Get session details
- `end_session(session_id)` - End active session
- `get_next_task(session_id, previous_response)` - Get next task
- `submit_response(session_id, task_id, response, response_time)` - Submit response
- `batch_submit_trials(session_id, trials, client_info)` - Batch submission
- `get_analytics(user_id, from_date, to_date, domain, granularity)` - Get analytics
- `get_leaderboard(domain, metric, period, limit)` - Get leaderboard
- `export_data(format, include, from_date, to_date)` - Export data

### Models

- `User` - User account information
- `Session` - Learning session
- `Task` - Task to solve
- `TaskResponse` - Response to task
- `Analytics` - Analytics data
- `SessionConfig` - Session configuration
- `LearnerModel` - Learner model parameters

### Exceptions

- `AbcdeezError` - Base exception
- `AuthenticationError` - Authentication failed
- `RateLimitError` - Rate limit exceeded
- `ValidationError` - Invalid input
- `NetworkError` - Network issue

## Configuration

Set environment variables:

```bash
export ABCDEEZ_API_URL="https://api.graphlearning.app/v1"
export ABCDEEZ_API_KEY="your-api-key"
export ABCDEEZ_TIMEOUT=30
```

Or use configuration file `~/.abcdeez/config.json`:

```json
{
  "api_url": "https://api.graphlearning.app/v1",
  "api_key": "your-api-key",
  "timeout": 30,
  "retry_count": 3
}
```

## Testing

Run tests:
```bash
pytest tests/
```

With coverage:
```bash
pytest --cov=abcdeez tests/
```

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing`)
3. Commit changes (`git commit -am 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing`)
5. Open Pull Request

## License

MIT License - see LICENSE file for details.

## Support

- Documentation: https://docs.graphlearning.app
- Issues: https://github.com/yourusername/abcdeez/issues
- Discord: https://discord.gg/graphlearning
- Email: support@graphlearning.app
# Backend Integration Guide

The TUI now supports real-time data reporting to a backend server for centralized data collection and experiment management.

## Features

- ✅ Automatic participant registration
- ✅ Real-time response buffering and batch sync
- ✅ Session data upload on completion
- ✅ Live metrics streaming
- ✅ Error reporting
- ✅ Offline mode with local storage
- ✅ Automatic retry with exponential backoff
- ✅ Connection status indicator in UI

## Quick Start

### 1. Using the Mock Backend (Testing)

Start the mock backend server:
```bash
cargo run --bin mock_backend --features backend-server
```

This will start a server on `http://127.0.0.1:3000` that logs all received data.

### 2. Configure the TUI

#### Option A: Environment Variables
```bash
export LEARNING_API_URL=http://127.0.0.1:3000/v1
export EXPERIMENT_ID=test_experiment
cargo run --features cli
```

#### Option B: Configuration File
Create `backend.toml`:
```toml
api_url = "http://127.0.0.1:3000/v1"
experiment_id = "test_experiment"
batch_size = 50
sync_interval_secs = 60
```

Then run:
```bash
cargo run --features cli
```

### 3. Monitor Connection Status

The TUI header shows real-time connection status:
- 🟢 Connected - Backend is reachable
- 🔄 Syncing - Currently uploading data
- 🔴 Error - Connection failed (continuing offline)
- ⚪ Offline - No backend configured

## Backend API Specification

### Endpoints

#### Health Check
```
GET /v1/health
Response: { "status": "ok", "timestamp": "..." }
```

#### Participant Registration
```
POST /v1/participants
Body: {
  "participant_id": "P001",
  "experiment_id": "exp_001",
  "group": "Adaptive",
  "timestamp": "2024-01-01T00:00:00Z",
  "metadata": {}
}
Response: {
  "token": "uuid",
  "expires_at": "2024-01-02T00:00:00Z"
}
```

#### Start Session
```
POST /v1/sessions/start
Body: {
  "token": "uuid",
  "timestamp": "2024-01-01T00:00:00Z"
}
Response: {
  "session_id": "session_uuid"
}
```

#### Batch Response Upload
```
POST /v1/responses/batch
Body: {
  "experiment_id": "exp_001",
  "responses": [
    {
      "task": { ... },
      "user_answer": "B",
      "correct": true,
      "response_time_ms": 1234,
      "timestamp": "2024-01-01T00:00:00Z"
    }
  ],
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### Complete Session Upload
```
POST /v1/sessions/complete
Body: LearnerDataExport (full session data)
```

#### Metrics Update
```
POST /v1/metrics
Body: {
  "participant_id": "P001",
  "experiment_id": "exp_001",
  "metrics": { ... },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Data Flow

1. **Registration**: When training starts, participant is registered with backend
2. **Buffering**: Responses are buffered locally (default: 50 responses)
3. **Batch Sync**: Automatic sync when buffer full or interval reached (default: 60s)
4. **Live Metrics**: Performance metrics sent after each response
5. **Session Upload**: Complete data uploaded when session ends
6. **Local Backup**: Data always saved locally as fallback

## Offline Mode

The system works seamlessly offline:
- All data stored locally
- Automatic retry when connection restored
- No data loss during disconnections
- Export functions work without backend

## Production Backend Implementation

For production, implement a backend that:

1. **Stores Data**:
   - PostgreSQL/MongoDB for responses
   - Redis for real-time metrics
   - S3/Cloud Storage for exports

2. **Provides Analytics**:
   - Real-time dashboards
   - Experiment monitoring
   - Data quality checks
   - Automated reports

3. **Manages Experiments**:
   - Group assignment
   - Condition balancing
   - Yoking control
   - Task sequencing

Example architecture:
```
┌─────────────┐     ┌──────────────┐     ┌────────────┐
│   TUI       │────▶│  API Server  │────▶│  Database  │
│  Clients    │     │   (FastAPI)  │     │ (Postgres) │
└─────────────┘     └──────────────┘     └────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │   Analytics  │
                    │  (Grafana)   │
                    └──────────────┘
```

## Security Considerations

1. **Authentication**: Use API keys or JWT tokens
2. **HTTPS**: Always use TLS in production
3. **Rate Limiting**: Prevent abuse
4. **Data Encryption**: Encrypt sensitive participant data
5. **GDPR Compliance**: Implement data retention policies

## Monitoring

Track these metrics:
- Response upload success rate
- Average sync latency
- Connection failures
- Data completeness
- Participant retention

## Troubleshooting

### Connection Issues
```bash
# Test backend connectivity
curl http://your-backend/v1/health

# Check environment
echo $LEARNING_API_URL

# Verify config file
cat backend.toml
```

### Data Not Syncing
- Check sync status indicator in UI
- Verify batch_size and sync_interval settings
- Check backend logs for errors
- Ensure API key is valid

### Performance Issues
- Reduce batch_size for faster syncs
- Increase timeout_secs for slow connections
- Use async client for better concurrency

## Example Deployment

### Docker Compose
```yaml
version: '3.8'
services:
  backend:
    image: learning-backend:latest
    ports:
      - "8000:8000"
    environment:
      DATABASE_URL: postgresql://...
      
  database:
    image: postgres:15
    volumes:
      - pgdata:/var/lib/postgresql/data
      
  redis:
    image: redis:7
    
volumes:
  pgdata:
```

### Kubernetes
```yaml
apiVersion: v1
kind: Service
metadata:
  name: learning-backend
spec:
  selector:
    app: backend
  ports:
    - port: 80
      targetPort: 8000
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: backend
  template:
    spec:
      containers:
      - name: api
        image: learning-backend:latest
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: url
```

## Backend Libraries

For implementing your own backend:

### Python (FastAPI)
```python
from fastapi import FastAPI
from pydantic import BaseModel
import asyncpg

app = FastAPI()

@app.post("/v1/participants")
async def register_participant(data: ParticipantRegistration):
    # Store in database
    # Generate token
    return {"token": "...", "expires_at": "..."}
```

### Node.js (Express)
```javascript
const express = require('express');
const { Pool } = require('pg');

const app = express();
const pool = new Pool();

app.post('/v1/participants', async (req, res) => {
  // Store in database
  // Generate token
  res.json({ token: '...', expires_at: '...' });
});
```

### Rust (Axum)
See `src/bin/mock_backend.rs` for a complete example.

## Testing

Run integration tests:
```bash
# Start mock backend
cargo run --bin mock_backend --features backend-server &

# Run TUI with test config
LEARNING_API_URL=http://127.0.0.1:3000/v1 cargo run --features cli

# Check data was received
curl http://127.0.0.1:3000/v1/stats
```

## Support

For backend integration issues:
- Check this guide first
- Review mock_backend.rs for API examples
- Open an issue with logs and configuration
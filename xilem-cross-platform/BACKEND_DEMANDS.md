# BACKEND_DEMANDS.md

This document tracks the API and data model mismatches between the Xilem UI app (`xilem-cross-platform/app`) and the Rust web backend (`web-backend`). It is a living checklist of backend adjustments that would enable the UI to run against the real API instead of the current `MockApiClient`.

Scope:
- Focus on REST and WebSocket contracts the Xilem UI needs.
- Identify mismatches in routes, payloads, and DTO field shapes/types.
- Propose concrete backend-side adjustments (without changing backend code in this PR).
- Offer UI-side fallbacks when backend changes are not feasible.

References:
- UI client and models: xilem-cross-platform/app/src/{api.rs,models.rs}
- Backend handlers: web-backend/src/handlers/{auth.rs,learner.rs,session.rs,analytics.rs,admin.rs,experiment.rs,task.rs,music.rs}
- Backend models: web-backend/src/models/{user.rs,learner.rs,session.rs,response.rs,mod.rs}
- WebSocket protocol: web-backend/src/websocket/mod.rs

---

## 1) Authentication

Current UI expectations:
- POST /api/auth/login -> TokenResponse { access_token, refresh_token, token_type, expires_in }
- GET /api/auth/me -> User (authorized with Authorization: Bearer <access_token>)

Backend status:
- Handlers exist in auth.rs for login and me. Contracts match TokenResponse and User.
- Tokens stored in Redis; Bearer header required.

Demands/Notes:
- Confirm route prefixes. UI assumes routes are under `/api/auth/*`. Ensure the router mounts `auth` at `/api/auth`.
- Ensure CORS/headers allow the UI to send `Authorization: Bearer {jwt}` with GET /api/auth/me.

Action:
- [OK] TokenResponse type matches UI.
- [OK] /api/auth/me returns a full User; UI’s `User` struct includes `metadata` skipping; currently the UI’s `User` drops metadata. This is fine for now.

Open Question:
- User.id is a UUID in backend JSON; UI uses String. Ensure UUID is serialized as a standard string (backend appears to do this; keep it consistent).

---

## 2) Learners

Current UI expectations:
- POST /api/learners with body { display_name } -> Learner (lightweight, UI constructs core model locally)
- GET /api/learners/{learner_id} -> Learner

Backend status:
- POST create (handlers/learner.rs::create) accepts CreateLearnerRequest { display_name, user_id? } and builds an alphabet topology by default (server-side).
- GET get (handlers/learner.rs::get) returns the server Learner with additional fields (last_active, total_practice_time_seconds, metadata).
- IDs represented as UUIDs.

Mismatches and Demands:
- The UI immediately constructs a client-side `core_model` tied to a client-generated topology after learner creation. This is client-only and not serialized. No backend action required, but:
  - The UI would benefit from the backend returning the topology info (type and size/data) associated with the learner’s default domain, if any. Optional.

- Route inventory:
  - [Missing for UI] GET /api/learners/{learner_id}/sessions
    - UI currently calls this to build `ExportData` but the backend doesn’t expose it.
    - Demand: Add a list sessions route by learner (basic fields, most recent first).

Action:
- [ADD] GET /api/learners/{learner_id}/sessions -> Vec<Session>

---

## 3) Sessions

Current UI expectations (ApiClient):
- POST /api/sessions with CreateSessionRequest { learner_id: String, topology_type: String, topology_data?: Value } -> Session
- GET /api/sessions/{session_id} -> Session
- POST /api/sessions/{session_id}/complete -> () (unit/empty)

Backend status:
- POST create (handlers/session.rs::create) expects:
  - CreateSessionRequest { learner_id: Uuid, topology_type: TopologyType (enum), topology_size?: usize }
  - It builds Topology server-side based on enum, stores serialized topology_data, sets status Active.
- GET /api/sessions/{id} returns Session with topology_data (serialized JSON string/value).
- POST /api/sessions/{id}/complete returns a SessionSummary JSON (not empty).

Mismatches and Demands:
- UI sends `topology_type` as a String (e.g., "alphabet", "days_of_week", "music", "mathematics") while backend expects the Rust enum TopologyType serialized. Two options:
  1) Backend: Accept both string and enum for `topology_type` (string values: ["linear","cyclic","partial_order","general_graph"]) and map UI domain to enum; or
  2) UI: Change to send the enum strings the backend expects.

- UI includes optional `topology_data` for custom or precomputed topologies. Backend ignores UI-provided topology_data (creates topology server-side). For parity with client-driven topologies:
  - Demand: Accept optional topology_data on create, and if provided, trust/use it (skip server default builder).

- Completion:
  - UI expects empty response from POST /api/sessions/{id}/complete, backend returns JSON `SessionSummary`.
  - Demand: Either return 204 No Content (preferred for UI) or the UI will adapt to parse the summary. Documented preference: 204 or stable JSON contract if kept.

Actions:
- [EITHER] Accept `topology_type` as string variants or document required enum strings for UI to send.
- [ADD/RESPECT] Optional `topology_data` passthrough on create.
- [DECIDE] Response shape for complete: adopt 204 or document JSON schema for the UI.

---

## 4) Responses (Task submissions)

Current UI expectation (ApiClient):
- POST /api/sessions/{session_id}/responses with body:
  - SubmitResponseRequest { task_type: String, task_data: Value, user_answer: String, correct: bool, response_time_ms: i32 }
  - Returns () (unit/empty)

Backend status:
- POST submit_response (handlers/session.rs::submit_response):
  - Expects TaskResponse { task_type, task_data, user_answer: Option<String>, response_time_ms: i64, hint_level: Option<i32> }
  - Correctness is computed server-side in validate_task_response().
  - Response returns JSON with fields: { response_id, sequence_number, correct, intervention, next_task }

Mismatches and Demands:
- UI currently sets `correct` client-side and expects no return body.
- Backend computes `correct` and returns JSON (including intervention and next_task hooks).
- Demand: Stabilize a single contract. Preferred:
  - Backend remains source of truth for correctness and returns JSON (status accepted). UI will adjust to parse the JSON and ignore sending `correct`.
  - Adjust request schema to remove `correct` and make `user_answer` non-optional (UI always submits string).
  - Adjust response_time_ms type consistency (i32 vs i64). Prefer i64 in API (backend currently uses i64).
- Route naming:
  - UI calls `/responses`, backend uses the same; good.

Actions:
- [CHANGE] Remove `correct` from request schema; UI will not send it.
- [DOC] Response returns JSON with `correct`, `intervention`, and optional `next_task`. UI will parse and display.

---

## 5) Session responses retrieval

Current UI expectation:
- GET /api/sessions/{session_id}/responses -> Vec<Value> (used in exports/dashboard)

Backend status:
- GET /api/sessions/{session_id}/replay returns Vec<ResponseRecord>
- There is no `/responses` list endpoint.

Demands:
- [ADD] GET /api/sessions/{session_id}/responses aliasing or delegating to `replay`.
  - Response type can remain `ResponseRecord[]` as-is; UI will adapt to consume the schema.
  - Alternatively, document `/replay` and UI will consume that route name directly.

---

## 6) Export and analytics

Current UI behavior:
- Uses local export composition; does not call backend export yet, but expects:
  - GET /api/learners/{learner_id}/sessions (missing)
  - Potentially GET /api/learners/{learner_id}/stats (exists handlers/learner.rs::stats)
- UI’s `get_learner_performance` returns defaults (no backend call yet).

Backend status:
- Analytics endpoints exist: `/api/analytics/population`, `/api/analytics/bottlenecks`, `/api/analytics/learning_curves`, `/api/analytics/compare`, `/api/analytics/live`.
- Learner stats: `handlers/learner.rs::stats` exists.

Demands:
- [OK] Keep analytics endpoints; UI can integrate later.
- [ADD] `/api/learners/{learner_id}/sessions` listing to simplify UI export composition.

Optional (nice to have):
- `/api/export/learner/{learner_id}` to mirror the UI’s comprehensive export format (server-side aggregation), though not required if UI continues local export.

---

## 7) Models and DTOs alignment

Key DTOs used by UI:

- User (UI) vs backend:
  - Backend adds `metadata`; UI can ignore unknown fields.
  - Ensure UUIDs are serialized as strings.

- Learner (UI):
  - UI’s `Learner` includes a non-serializable `core_model` (client-only). This should never be expected from backend.
  - Backend `Learner` includes `last_active`, `total_practice_time_seconds`, `metadata`; UI will ignore unknown fields.

- Session (UI):
  - UI holds `topology: Option<Topology>` and `responses: Vec<CoreTaskResponse>` (client session state).
  - Backend `Session` has `topology_data: Value` and no `responses` list inline.
  - Demand: The GET session route does not need to inline responses (UI will call `/responses`/`/replay` separately).

- CreateSessionRequest:
  - UI currently: { learner_id: String, topology_type: String, topology_data?: Value }
  - Backend: { learner_id: Uuid, topology_type: TopologyType, topology_size?: usize }
  - Demand: Contract decision on `topology_type` (string vs enum) and accepting `topology_data`.

- SubmitResponseRequest:
  - UI: { task_type: String, task_data: Value, user_answer: String, correct: bool, response_time_ms: i32 }
  - Backend: { task_type: String, task_data: Value, user_answer: Option<String>, response_time_ms: i64, hint_level?: i32 }
  - Demands: Remove `correct` from UI request; converge on `response_time_ms` i64; make `user_answer` required.

- Session completion:
  - UI expects empty; backend returns JSON `SessionSummary`.
  - Demand: Decide and document. UI can support either; preferred backend: 204 No Content for simplicity.

---

## 8) WebSocket protocol

Server-side typed protocol exists in `websocket/mod.rs`:
- ClientMessage: StartTask, SubmitResponse, RequestHint, Pause, Resume, Heartbeat
- ServerMessage: Task, Hint, Feedback, Intervention, StatsUpdate, Error, Heartbeat

Demands:
- Document the WS endpoint URL(s) and auth (e.g., `/ws/session/:id` and `/ws/analytics`?).
- Provide the JSON wire format examples for each message type (fields and types), including timestamps format.
- UI will connect to these later for real-time training and analytics; for now, HTTP is sufficient.

---

## 9) Route inventory the UI assumes

The UI’s `ApiClient` (or future real client) assumes these routes exist:

- Auth
  - POST /api/auth/login -> TokenResponse [OK]
  - GET /api/auth/me -> User [OK]

- Learners
  - POST /api/learners { display_name } -> Learner [OK]
  - GET /api/learners/{learner_id} -> Learner [OK]
  - GET /api/learners/{learner_id}/sessions -> Vec<Session> [MISSING]

- Sessions
  - POST /api/sessions { learner_id, topology_type (string), topology_data? } -> Session [MISMATCH]
  - GET /api/sessions/{session_id} -> Session [OK]
  - POST /api/sessions/{session_id}/complete -> () [MISMATCH (backend returns JSON)]

- Responses
  - POST /api/sessions/{session_id}/responses -> () [MISMATCH (backend returns JSON; schema differs)]
  - GET /api/sessions/{session_id}/responses -> Vec<ResponseRecord> [MISSING; backend has /replay]

- Analytics (future UI integration)
  - GET /api/analytics/population [OK]
  - GET /api/analytics/bottlenecks [OK]
  - GET /api/analytics/learning_curves [OK]
  - POST /api/analytics/compare [OK]
  - GET /api/analytics/live [OK]
  - GET /api/learners/{learner_id}/stats [OK]

---

## 10) Summary of required backend adjustments

Priority A (blocks UI HTTP integration):
1) Add GET /api/learners/{learner_id}/sessions -> Vec<Session>
2) Add/alias GET /api/sessions/{session_id}/responses (alias of replay) -> Vec<ResponseRecord>
3) Align POST /api/sessions/{session_id}/responses:
   - Request: remove `correct`; set `user_answer: String` (non-optional); use `response_time_ms: i64`
   - Response: return JSON { response_id, sequence_number, correct, intervention?, next_task? }
4) Decide Session completion behavior:
   - Either return 204 No Content, or document stable JSON schema for `SessionSummary`

Priority B (quality and parity):
5) Session creation request:
   - Accept `topology_type` as string variants (["linear","cyclic","partial_order","general_graph"]) or document enum serialization required by backend
   - Accept optional `topology_data` and prefer using it if provided
6) Document WebSocket endpoints (URL, auth, JSON wire schema)
7) Keep UUIDs serialized as strings consistently for all IDs

Optional (nice-to-have):
8) Provide a server-side learner export endpoint parallel to UI’s export composition

---

## 11) UI-side adaptations we can make (if backend changes are deferred)

- Change UI to call GET /api/sessions/{id}/replay instead of `/responses` until alias is added.
- Parse JSON from POST /api/sessions/{id}/responses (backend response) and ignore sending `correct`.
- Handle JSON body from POST /api/sessions/{id}/complete instead of expecting empty.
- Send enum-compatible strings for `topology_type` to match backend’s `TopologyType` `Debug` format (e.g., "Linear","Cyclic","PartialOrder","GeneralGraph") as a temporary workaround.

---

## 12) Open questions

- Should topology selection be authoritative on the client or server? (Right now server builds it. UI can adapt if server remains authoritative.)
- Do we want to support custom topologies pushed from UI (`topology_data`)?
- For interventions, will we formalize an enum/string for actions (`ProvideHint`, `ProvideWorkedExample`, `SuggestBreak`, etc.) so the UI can render affordances consistently?

---

Changelog:
- 2025-08-08: Initial draft capturing current mismatches and proposed resolutions.
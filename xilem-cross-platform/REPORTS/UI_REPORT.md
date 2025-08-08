# UI_REPORT.md

Author: assistant (acting as lead engineer)
Date: 2025-08-08

Overview
--------
This report captures the state of the Xilem-based UI in `xilem-cross-platform/app`, explains what I changed so far, and lays out a prioritized roadmap with concrete tasks to turn this small prototype into a robust, production-ready cognitive-science demo application.

Goals for the project
- Make the UI stable, strongly typed, and maintainable.
- Provide a deterministic demo flow that showcases system capabilities for portfolio/experiments.
- Integrate safely and clearly with the core library in `alphabet-terminal-prototype`.
- Make the UX complete and usable for real demonstrations (hints, adaptive scheduling, dashboards).
- Add testability, reproducible demos, and CI guidance.

Current state (short)
- The app is a small Xilem UI with five screens: `Welcome`, `DomainSelection`, `Training`, `Dashboard`, and `Settings` (`app/src/screens.rs`).
- The UI previously suffered from type errors due to mixed `impl Trait` and dynamic views; these have been aggressively cleaned so that view tuples are homogeneous and the screens compile.
- I added a `demo_showcase()` in `AppData` (in `app/src/lib.rs`) that programmatically exercises a short training flow and navigates to the Dashboard.
- I added `xilem-cross-platform/BACKEND_DEMANDS.md` enumerating the API/DTO mismatches between the UI and the backend (`web-backend`).
- I inserted a `Demo Showcase` button to the welcome screen that runs `demo_showcase()` and shows results.
- The UI now has improved placeholders and unified card shapes, so the type system is satisfied and the view code is stable.

Key code locations
- App state, core UI logic: `xilem-cross-platform/app/src/lib.rs`
- Screens and components: `xilem-cross-platform/app/src/screens.rs` and `xilem-cross-platform/app/src/components.rs`
- API client: `xilem-cross-platform/app/src/api.rs`
- Core library (models & algorithms): `abcdeez/alphabet-terminal-prototype/src/*` (lots of helpers; notably `demo.rs`, `tasks.rs`, `adaptive.rs`, `learner.rs`)

High-level roadmap (prioritized)
1. Demo & UX polish (Immediate; 1-2 days)
   - Ensure `Demo Showcase` demonstrates a full narrative: create user -> create learner -> start session -> complete tasks -> show dashboard.
   - Improve domain selection UX: show brief topology preview and enable a real `Select` button that updates UI and shows a confirmation modal.
   - Make Training controls fully usable:
     - Pause / Resume behavior persists session.
     - End session shows a confirmation modal and then navigates to Dashboard with summary.
   - Ensure hints are visible and graded ( Confirm / Next button flows ).

2. API alignment & integration (Short; 2-4 days)
   - Use `BACKEND_DEMANDS.md` to decide whether to adapt the UI or the backend:
     - Prefer adapting backend for consistency (e.g., server should compute correctness), but allow UI fallbacks.
   - Implement robust `ApiClient` usage:
     - Add typed responses for session creation, response submission, session listing.
     - Add graceful fallback to `MockApiClient` when backend not reachable.

3. Componentization & refactor (Medium; 3-7 days)
   - Break `screens.rs` into separate files: `screens/welcome.rs`, `screens/domain.rs`, `screens/training.rs`, `screens/dashboard.rs`, `screens/settings.rs`. Keep `components.rs` as library of widgets.
   - Introduce a small `viewmodel` module that prepares UI payloads (strings, arrays, booleans) so closures and widget assembly are simple and strongly typed.
   - Provide `ui::widgets` helpers for common patterns:
     - `info_card(title, rows)` -> homogeneous shapes
     - `progress_widget(...)`
     - `task_widget(UITask)` that encapsulates all interaction and returns a single `impl WidgetView<AppData>`

4. Async & background tasks (Medium; 3-5 days)
   - Replace synchronous uses of `runtime.block_on` in UI with Xilem's `task` view pattern or spawn async tasks that post results back to `AppData`.
   - Use Xilem asynchronous primitives for network calls so the UI never blocks.

5. Testing & reproducible demos (Medium; 3-5 days)
   - Add unit tests for business logic in `app` where possible (pure helpers).
   - Add integration tests that run `demo_showcase()` and assert Dashboard state (e.g., `current_session.status == "completed"`).
   - Add a deterministic demo mode: use seeded RNG in `demo_showcase()` to reproduce the same behavior reliably.

6. Observability, performance & accessibility (Long; 1-2 weeks)
   - Instrument UI with structured logging/tracing (use `tracing`): log key actions (StartTask, SubmitResponse, HintRequested).
   - Add accessibility labels, keyboard navigation, and color contrast checks for major widgets.
   - Optimize heavy visualizations: memoize histograms/learning-curve computations; avoid recomputing on every render.

Concrete tasks and file-level actions
(Each task is actionable and referenceable)

A. Demo & UX (Immediate)
- Add modal confirmations and success toasts:
  - Files: `app/src/screens.rs`
  - Actions: implement a small `confirm_dialog(title, body, on_confirm)` component in `components.rs`.
- Improve domain selection:
  - Files: `app/src/screens.rs`, `app/src/components.rs`
  - Actions: replace current `Select` label/button with `button("Select"` that opens a small preview popup (show first N nodes from `Topology`).
  - Use `Topology::sample_nodes()` (implement in core if missing) to show a preview list.
- Hook demo to visual timeline:
  - Files: `app/src/lib.rs`, `app/src/screens.rs`
  - Actions: `demo_showcase()` should set a `demo_timeline: Option<Vec<String>>` in `AppData` and the Dashboard shows an animated timeline for the portfolio.

B. API & backend alignment (Short)
- Review `BACKEND_DEMANDS.md` and pick a strategy:
  - Option 1: change backend to accept `topology_data` and return `SessionSummary` on complete (preferred).
  - Option 2: adapt UI to backend (send enum strings, parse JSON from session complete and response submissions).
- Make `api.rs` robust:
  - Use strongly typed DTOs matching backend `web-backend/src/models`.
  - Add retry/backoff for transient failures.
  - Add `get_learner_sessions(learner_id)` and `get_session_responses(session_id)` helpers.
  - Ensure `MockApiClient` matches the UI shapes used by the app.

C. Componentization & code hygiene (Medium)
- Move screens into separate files:
  - Create `app/src/screens/welcome.rs` etc. Export them from `app/src/screens/mod.rs`.
- Create `app/src/viewmodel.rs`:
  - Provide `fn training_view_model(data: &AppData) -> TrainingViewModel` to shape the UI inputs.
- Clean `components.rs`:
  - Standardize APIs (avoid returning `AnyWidgetView` unless necessary).
  - Add `confirmation_dialog`, `loader_overlay`, `toast` components.

D. Async & background tasks (Medium)
- Replace `self.runtime.block_on(...)` usages:
  - Use Xilem's `task` or `view::task` helper to kick off async operations.
- Add an `AsyncResult<T>` small helper to keep track of in-progress network operations in `AppData`.

E. Testing & CI (Medium)
- Add tests:
  - UI-level: call `demo_showcase()` and assert resulting `AppData` states.
  - Core-level: integrate `alphabet-terminal-prototype/src/demo.rs` tests as a smoke check.
- CI:
  - Add GH Actions with steps: `cargo test` (core), `cargo build` (app), clippy, fmt, run demo smoke test (headless).
  - Consider a dedicated workflow for release builds (desktop, mobile cross-compiles).

F. UX polish (Immediate+)
- Training UI:
  - Add clearly visible response timers, ability to submit “I don't know”, and a keyboard-driven answer selector.
  - Add hint history: which hints were shown and why (intervention type).
- Dashboard:
  - Make charts interactive (click on a point to view the task/response).
  - Add export options: JSON (already present), CSV, and "replay" JSON (full model snapshot + responses).
- Settings:
  - Add per-domain defaults and an advanced panel for scheduler hyperparameters to support experiments.

Production considerations
- Data safety and export:
  - Ensure `ExportData` contains only de-identified artifacts in production builds unless user opts in.
- Reproducibility:
  - Store seeds and model snapshots with each demo/replay export.
- Privacy & telemetry:
  - Telemetry must be opt-in and documented for experiments.

Timeline (rough)
- Day 0-1: Demo & immediate UX polish (complete demo flow, basic confirm modals, domain Select).
- Day 2-5: API alignment changes and `ApiClient` hardening (assumes backend cooperates).
- Day 5-12: Componentization, viewmodel pattern, async refactor.
- Day 12-20: Tests, CI, performance polishing, accessibility improvements.

Appendix
- Immediate code references:
  - UI entry and state: `xilem-cross-platform/app/src/lib.rs`
  - Screens: `xilem-cross-platform/app/src/screens.rs`
  - Widgets: `xilem-cross-platform/app/src/components.rs`
  - API client: `xilem-cross-platform/app/src/api.rs`
  - Core algorithms and demo runner: `alphabet-terminal-prototype/src/demo.rs`, `tasks.rs`, `adaptive.rs`, `learner.rs`.
- The existing `Demo Showcase` is implemented as `AppData::demo_showcase()` (in `app/src/lib.rs`) and wired to a button on the welcome screen.

Closing notes
- I focused this pass on making the UI compile and be usable as a deterministic demo/portfolio piece.
- The next high-value step is to finalize backend + API contract (see `BACKEND_DEMANDS.md`) and switch network calls from a blocking runtime to Xilem's async task model.
- Tell me which priority you want me to tackle next, and I’ll implement concrete changes (component split, full demo polish with replay export, or backend integration).

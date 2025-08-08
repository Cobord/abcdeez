// Copyright 2024 Graph Learning System Authors
// SPDX-License-Identifier: Apache-2.0

use xilem::EventLoop;

fn main() {
    // Initialize the event loop
    let event_loop = EventLoop::with_user_event();
    
    // Run the app
    graph_learning_ui::run(event_loop);
}
#!/bin/bash

# Script to view ABCDEEZ logs in real-time
# Usage: ./view_logs.sh [category]
# Categories: default, ui, network, auth, learning, gamification

SUBSYSTEM="com.abcdeez.app"
CATEGORY=${1:-""}

echo "🦀 ABCDEEZ Log Viewer"
echo "===================="
echo "Subsystem: $SUBSYSTEM"

if [ -z "$CATEGORY" ]; then
    echo "Showing all categories"
    echo ""
    # Stream all logs from our subsystem
    log stream --predicate "subsystem == '$SUBSYSTEM'" --level debug --style syslog
else
    echo "Category: $CATEGORY"
    echo ""
    # Stream logs from specific category
    log stream --predicate "subsystem == '$SUBSYSTEM' AND category == '$CATEGORY'" --level debug --style syslog
fi
#!/bin/bash

# Test runner script for local development
# Usage: ./scripts/test.sh [component] [options]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_info() {
    echo -e "${YELLOW}[ℹ]${NC} $1"
}

# Default to testing all components
COMPONENT=${1:-all}
EXTRA_ARGS="${@:2}"

# Header
echo "========================================="
echo "   Graph Learning System Test Runner"
echo "========================================="
echo ""

# Run tests based on component
case $COMPONENT in
    core)
        print_info "Testing Core Library (graph-learning-core)..."
        cargo test --package graph-learning-core $EXTRA_ARGS
        print_status "Core library tests passed!"
        ;;

    ui|app)
        print_info "Testing UI Application (graph_learning_app)..."
        cargo test --package graph_learning_app $EXTRA_ARGS
        print_status "UI application tests passed!"
        ;;

    backend|web)
        print_info "Testing Web Backend (web-backend)..."
        cargo test --package web-backend $EXTRA_ARGS
        print_status "Web backend tests passed!"
        ;;

    demo)
        print_info "Running demo integration tests..."
        cargo test --package graph_learning_app --test demo_integration $EXTRA_ARGS
        print_status "Demo integration tests passed!"
        ;;

    deterministic)
        print_info "Testing deterministic demo behavior..."
        cargo test --package graph_learning_app test_demo_showcase_deterministic -- --nocapture
        print_status "Deterministic demo tests passed!"
        ;;

    quick)
        print_info "Running quick test suite (unit tests only)..."
        cargo test --lib --all $EXTRA_ARGS
        print_status "Quick tests passed!"
        ;;

    all)
        print_info "Running complete test suite..."

        print_info "1/5 Testing Core Library..."
        cargo test --package graph-learning-core --quiet
        print_status "Core library ✓"

        print_info "2/5 Testing UI Application..."
        cargo test --package graph_learning_app --quiet
        print_status "UI application ✓"

        print_info "3/5 Testing Web Backend..."
        cargo test --package web-backend --quiet
        print_status "Web backend ✓"

        print_info "4/5 Running integration tests..."
        cargo test --test '*' --quiet
        print_status "Integration tests ✓"

        print_info "5/5 Checking documentation tests..."
        cargo test --doc --quiet
        print_status "Doc tests ✓"

        echo ""
        print_status "All tests passed successfully!"
        ;;

    coverage)
        print_info "Generating test coverage report..."

        # Check if cargo-tarpaulin is installed
        if ! command -v cargo-tarpaulin &> /dev/null; then
            print_error "cargo-tarpaulin not found. Installing..."
            cargo install cargo-tarpaulin
        fi

        cargo tarpaulin --out Html --all-features --workspace --timeout 300
        print_status "Coverage report generated at tarpaulin-report.html"

        # Try to open the report
        if command -v open &> /dev/null; then
            open tarpaulin-report.html
        elif command -v xdg-open &> /dev/null; then
            xdg-open tarpaulin-report.html
        fi
        ;;

    bench|benchmark)
        print_info "Running benchmarks..."
        cargo bench --all
        print_status "Benchmarks complete!"
        ;;

    watch)
        print_info "Starting test watcher..."

        # Check if cargo-watch is installed
        if ! command -v cargo-watch &> /dev/null; then
            print_error "cargo-watch not found. Installing..."
            cargo install cargo-watch
        fi

        cargo watch -x test
        ;;

    *)
        print_error "Unknown component: $COMPONENT"
        echo ""
        echo "Usage: $0 [component] [options]"
        echo ""
        echo "Components:"
        echo "  core          - Test core library only"
        echo "  ui, app       - Test UI application only"
        echo "  backend, web  - Test web backend only"
        echo "  demo          - Run demo integration tests"
        echo "  deterministic - Test deterministic demo behavior"
        echo "  quick         - Run quick unit tests only"
        echo "  all           - Run complete test suite (default)"
        echo "  coverage      - Generate test coverage report"
        echo "  bench         - Run benchmarks"
        echo "  watch         - Start test watcher"
        echo ""
        echo "Examples:"
        echo "  $0                    # Run all tests"
        echo "  $0 core               # Test core library"
        echo "  $0 demo --nocapture   # Run demo tests with output"
        echo "  $0 quick --release    # Run quick tests in release mode"
        exit 1
        ;;
esac

# Check for failures in important files
if [ "$COMPONENT" = "all" ]; then
    echo ""
    print_info "Running additional checks..."

    # Check formatting
    if cargo fmt --all -- --check &> /dev/null; then
        print_status "Code formatting ✓"
    else
        print_error "Code formatting issues detected. Run 'cargo fmt --all' to fix."
    fi

    # Check clippy
    if cargo clippy --all-targets --all-features -- -W warnings &> /dev/null; then
        print_status "Clippy checks ✓"
    else
        print_error "Clippy warnings detected. Run 'cargo clippy --all-targets --all-features' to see details."
    fi
fi

echo ""
echo "========================================="
echo "   Test run completed successfully!"
echo "========================================="

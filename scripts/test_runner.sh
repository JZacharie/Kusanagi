#!/bin/bash
# Kusanagi Test Runner Script
# Usage: ./scripts/test_runner.sh [options]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔮 Kusanagi Test Runner${NC}"
echo "========================"
echo ""

# Default options
RUN_UNIT=true
RUN_INTEGRATION=true
RUN_DOC=false
VERBOSE=false
COVERAGE=false
WATCH=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --unit|-u)
            RUN_INTEGRATION=false
            shift
            ;;
        --integration|-i)
            RUN_UNIT=false
            shift
            ;;
        --doc|-d)
            RUN_DOC=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --coverage|-c)
            COVERAGE=true
            shift
            ;;
        --watch|-w)
            WATCH=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  -u, --unit          Run unit tests only"
            echo "  -i, --integration   Run integration tests only"
            echo "  -d, --doc           Run documentation tests"
            echo "  -v, --verbose       Show verbose output"
            echo "  -c, --coverage      Generate coverage report"
            echo "  -w, --watch         Watch for changes and re-run"
            echo "  -h, --help          Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Build test arguments
TEST_ARGS=""
if [ "$VERBOSE" = true ]; then
    TEST_ARGS="$TEST_ARGS --nocapture"
fi

# Run tests based on options
echo -e "${BLUE}Configuration:${NC}"
echo "  Unit tests: $RUN_UNIT"
echo "  Integration tests: $RUN_INTEGRATION"
echo "  Doc tests: $RUN_DOC"
echo "  Verbose: $VERBOSE"
echo "  Coverage: $COVERAGE"
echo ""

if [ "$WATCH" = true ]; then
    echo -e "${BLUE}Starting test watcher...${NC}"
    cargo watch -x "test $TEST_ARGS"
    exit 0
fi

if [ "$COVERAGE" = true ]; then
    echo -e "${BLUE}Generating coverage report...${NC}"
    
    # Check if cargo-tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${YELLOW}cargo-tarpaulin not found. Installing...${NC}"
        cargo install cargo-tarpaulin
    fi
    
    cargo tarpaulin --out Html --output-dir coverage
    
    echo ""
    echo -e "${GREEN}✅ Coverage report generated: coverage/index.html${NC}"
    
    # Try to open browser
    if command -v xdg-open &> /dev/null; then
        xdg-open coverage/index.html
    elif command -v open &> /dev/null; then
        open coverage/index.html
    fi
    
    exit 0
fi

# Track results
TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_IGNORED=0

run_test_suite() {
    local name=$1
    local cmd=$2
    
    echo -e "${BLUE}Running $name...${NC}"
    
    if eval "$cmd" 2>&1 | tee /tmp/test_output.txt; then
        local passed=$(grep -oE "[0-9]+ passed" /tmp/test_output.txt | awk '{sum+=$1} END {print sum}')
        local failed=$(grep -oE "[0-9]+ failed" /tmp/test_output.txt | awk '{sum+=$1} END {print sum}')
        local ignored=$(grep -oE "[0-9]+ ignored" /tmp/test_output.txt | awk '{sum+=$1} END {print sum}')
        
        TOTAL_PASSED=$((TOTAL_PASSED + ${passed:-0}))
        TOTAL_FAILED=$((TOTAL_FAILED + ${failed:-0}))
        TOTAL_IGNORED=$((TOTAL_IGNORED + ${ignored:-0}))
        
        return 0
    else
        local failed=$(grep -oE "[0-9]+ failed" /tmp/test_output.txt | awk '{sum+=$1} END {print sum}')
        TOTAL_FAILED=$((TOTAL_FAILED + ${failed:-0}))
        return 1
    fi
}

# Run unit tests (lib)
if [ "$RUN_UNIT" = true ]; then
    run_test_suite "Unit Tests (lib)" "cargo test --lib $TEST_ARGS" || true
    echo ""
fi

# Run integration tests
if [ "$RUN_INTEGRATION" = true ]; then
    run_test_suite "Integration Tests" "cargo test --test '*' $TEST_ARGS" || true
    echo ""
fi

# Run doc tests
if [ "$RUN_DOC" = true ]; then
    run_test_suite "Documentation Tests" "cargo test --doc $TEST_ARGS" || true
    echo ""
fi

# Summary
echo "========================"
echo -e "${BLUE}Test Summary${NC}"
echo "========================"
echo -e "${GREEN}Passed:  $TOTAL_PASSED${NC}"

if [ $TOTAL_FAILED -gt 0 ]; then
    echo -e "${RED}Failed:  $TOTAL_FAILED${NC}"
else
    echo -e "Failed:  $TOTAL_FAILED"
fi

if [ $TOTAL_IGNORED -gt 0 ]; then
    echo -e "${YELLOW}Ignored: $TOTAL_IGNORED${NC}"
fi

echo ""
echo -e "${GREEN}✅ Test run complete!${NC}"

# Exit with error code if any tests failed
if [ $TOTAL_FAILED -gt 0 ]; then
    exit 1
fi

exit 0

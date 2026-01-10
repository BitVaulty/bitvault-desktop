#!/bin/bash
# Test Runner Script for BitVault Desktop
#
# Runs all tests with proper output formatting and coverage reporting

set -e

echo "🧪 BitVault Desktop Test Suite"
echo "=============================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test suite
run_test_suite() {
    local suite_name=$1
    local test_file=$2
    
    echo -e "${YELLOW}Running ${suite_name}...${NC}"
    
    if cargo test --test "$test_file" 2>&1 | tee /tmp/test_output.txt; then
        local count=$(grep -c "test result: ok" /tmp/test_output.txt || echo "0")
        if [ "$count" -gt 0 ]; then
            local passed=$(grep "test result: ok" /tmp/test_output.txt | grep -oE "[0-9]+ passed" | grep -oE "[0-9]+" | head -1 || echo "0")
            PASSED_TESTS=$((PASSED_TESTS + passed))
            echo -e "${GREEN}✓ ${suite_name}: ${passed} tests passed${NC}"
        else
            echo -e "${RED}✗ ${suite_name}: No tests found${NC}"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    else
        echo -e "${RED}✗ ${suite_name}: Tests failed${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
    echo ""
}

# Run all test suites
echo "Running E2E Tests..."
run_test_suite "E2E Tests" "e2e_test"

echo "Running Integration Tests..."
run_test_suite "Integration Tests" "integration_test"

echo "Running Form Validation Tests..."
run_test_suite "Form Validation Tests" "form_validation_test"

echo "Running Error Handling E2E Tests..."
run_test_suite "Error Handling E2E Tests" "error_handling_e2e_test"

echo "Running Unit Tests..."
if cargo test --lib 2>&1 | tee /tmp/unit_test_output.txt; then
    local unit_passed=$(grep "test result: ok" /tmp/unit_test_output.txt | grep -oE "[0-9]+ passed" | grep -oE "[0-9]+" | head -1 || echo "0")
    PASSED_TESTS=$((PASSED_TESTS + unit_passed))
    echo -e "${GREEN}✓ Unit Tests: ${unit_passed} tests passed${NC}"
else
    echo -e "${RED}✗ Unit Tests: Tests failed${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
echo ""

# Summary
echo "=============================="
echo "Test Summary"
echo "=============================="
echo -e "Total Tests Passed: ${GREEN}${PASSED_TESTS}${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ All test suites passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ ${FAILED_TESTS} test suite(s) failed${NC}"
    exit 1
fi

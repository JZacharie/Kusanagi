#!/bin/bash
set -e

# Get list of files modified compared to origin/main (includes uncommitted changes)
MODIFIED_FILES=$(git diff --name-only origin/main | grep '^src/.*\.rs$' || true)

if [ -z "$MODIFIED_FILES" ]; then
    echo "No .rs files modified in src/. Running full coverage..."
    # Fallback to full coverage if no specific src files modified, or maybe just exit?
    # For safe optimization, let's just exit or run everything? 
    # The user asked to ignore unmodified files, so exiting with a message is appropriate for "incremental" feel.
    # But if they want to run *something*, maybe we should run all?
    # Let's stick to the plan: reduce time. If nothing modified, nothing to test.
    echo "No changes detected in source code. Exiting."
    exit 0
fi

echo "Modified files:"
echo "$MODIFIED_FILES"

# Build inclusions and test filters
INCLUDE_ARGS=""
# We need to find the packages/binaries/libraries corresponding to these files.
# For a simple project structure, we can try to guess the test targets.
# However, tarpaulin's --include-files works on file paths.

for file in $MODIFIED_FILES; do
    INCLUDE_ARGS="$INCLUDE_ARGS --include-files $file"
done

echo "Running coverage on modified files..."
# We don't strictly filter the *tests* to run (cargo test args) in this simple version 
# because mapping file -> test is hard in Rust without more complex logic.
# However, we DO filter the *instrumentation* with --include-files, which speeds up tarpaulin significantly.
# To further speed up, we could try to pass filenames to cargo test if they are integration tests, 
# but for unit tests inside src/, running 'cargo test' is usually fast if we limit instrumentation.

# Current approach: Limit instrumentation to modified files. 
# This reduces the overhead of coverage tracking.

if ! command -v cargo-tarpaulin &> /dev/null && ! cargo --list | grep -q "tarpaulin"; then
    echo "Error: cargo-tarpaulin is not installed."
    echo "Please install it with: cargo install cargo-tarpaulin"
    exit 1
fi

cargo tarpaulin $INCLUDE_ARGS --out Html --out Xml --output-dir coverage-fast

#!/usr/bin/env bash
set -euo pipefail

COVROOT=".cov"

if ! cargo tarpaulin --version &>/dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

mkdir -p "$COVROOT"

cargo tarpaulin -o Html -o Lcov --output-dir "$COVROOT" --fail-under 30 "$@"

if [[ "$(uname)" == "Darwin" ]]; then
    open "$COVROOT/tarpaulin-report.html"
else
    echo "Coverage report: $COVROOT/tarpaulin-report.html"
fi

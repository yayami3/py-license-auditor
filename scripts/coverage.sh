#!/bin/bash
# Test coverage script

echo "🧪 Running test coverage..."

# Quick summary only
echo "📊 Quick coverage summary:"
cargo llvm-cov --all-features --workspace --summary-only

echo ""
echo "📋 Full coverage report (takes longer):"
echo "cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info"
echo "cargo llvm-cov --all-features --workspace --html --output-dir target/coverage"

echo ""
echo "🌐 View HTML report:"
echo "open target/coverage/index.html"

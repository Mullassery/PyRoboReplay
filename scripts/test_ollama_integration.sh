#!/bin/bash
# Runs the real Ollama LLM integration tests in tests/test_ollama_integration.rs
# against a genuine local Ollama server.
#
# Usage:
#   ./scripts/test_ollama_integration.sh [model-tag]
#
# Requires the `ollama` CLI to be installed (https://ollama.com). Starts
# `ollama serve` in the background if it isn't already running, pulls the
# requested model if it isn't already pulled (default: qwen2.5:0.5b, a small
# ~400MB model chosen for fast pull/inference in CI/dev use, not production
# answer quality), then runs the ignored integration tests for real.

set -euo pipefail

MODEL="${1:-qwen2.5:0.5b}"
OLLAMA_BIN="${OLLAMA_BIN:-ollama}"

if ! command -v "$OLLAMA_BIN" >/dev/null 2>&1; then
    echo "error: '$OLLAMA_BIN' not found. Install Ollama from https://ollama.com" >&2
    exit 1
fi

STARTED_SERVER=0
if ! curl -sS -m 2 http://localhost:11434/api/version >/dev/null 2>&1; then
    echo "Starting 'ollama serve' in the background..."
    "$OLLAMA_BIN" serve > /tmp/pyroboreplay_ollama_serve.log 2>&1 &
    OLLAMA_SERVE_PID=$!
    STARTED_SERVER=1
    for _ in $(seq 1 30); do
        if curl -sS -m 2 http://localhost:11434/api/version >/dev/null 2>&1; then
            break
        fi
        sleep 1
    done
    if ! curl -sS -m 2 http://localhost:11434/api/version >/dev/null 2>&1; then
        echo "error: ollama server did not become ready in time" >&2
        exit 1
    fi
fi

if ! "$OLLAMA_BIN" list | awk '{print $1}' | grep -qx "$MODEL"; then
    echo "Pulling model '$MODEL' (first run only, ~hundreds of MB)..."
    "$OLLAMA_BIN" pull "$MODEL"
fi

echo "Running real Ollama integration tests against model '$MODEL'..."
PYROBOREPLAY_TEST_OLLAMA_MODEL="$MODEL" \
    cargo test --test test_ollama_integration -- --ignored --test-threads=1

STATUS=$?

if [ "$STARTED_SERVER" -eq 1 ]; then
    echo "Stopping the 'ollama serve' process this script started (pid $OLLAMA_SERVE_PID)..."
    kill "$OLLAMA_SERVE_PID" 2>/dev/null || true
fi

exit $STATUS

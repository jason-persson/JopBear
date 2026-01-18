#!/bin/bash
set -e

case "$1" in
  build)
    echo "Running build..."
    cargo build --release
    ;;
  test)
    echo "Running tests..."
    cargo test
    ;;
  *)
    echo "Usage: $0 {build|test}"
    exit 1
    ;;
esac
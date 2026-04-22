#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../../.."

cargo test application::read_model:: -- --nocapture
cargo test application::tests:: -- --nocapture

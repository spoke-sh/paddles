set positional-arguments

# Show available recipes.
default:
  @just --list

# Build the project.
build profile="debug":
  @set -eu; \
  case "{{profile}}" in \
    debug) \
      cargo build; \
    ;; \
    release) \
      cargo build --release; \
      mkdir -p target/release; \
      echo "Release binary created at target/release/paddles"; \
    ;; \
    *) echo "unsupported build profile: {{profile}}" >&2; exit 1 ;; \
  esac

# Build the release version.
build-release:
  @just build release

# Build with CUDA support enabled.
build-cuda:
  cargo build --release --features cuda

# Install frontend workspace dependencies.
frontend-install:
  npm ci

# Run frontend workspace verification checks.
frontend-quality:
  npm run lint

# Run frontend workspace unit/build verification.
frontend-test:
  npm run test

# Run frontend workspace browser e2e verification.
frontend-e2e:
  just frontend-build
  npm run e2e

# Build the runtime frontend workspace.
frontend-build:
  npm --workspace @paddles/web run build

# Run tests.
test:
  just frontend-install
  cargo nextest run --no-tests pass
  just frontend-test
  just frontend-build
  just frontend-e2e

# Run quality checks.
quality:
  just frontend-install
  cargo fmt --all --check
  cargo clippy --all-targets --all-features -- -D warnings
  just frontend-quality

# Run the paddles CLI. Use --cuda to enable GPU support.
paddles *args:
  #!/usr/bin/env bash
  just frontend-install
  just frontend-build
  FEATURES=""
  PASSED_ARGS=()
  for arg in "$@"; do
    if [ "$arg" == "--cuda" ]; then
      FEATURES="--features cuda"
    else
      PASSED_ARGS+=("$arg")
    fi
  done
  cargo run $FEATURES -- "${PASSED_ARGS[@]}"

# Standard mission path for verification. Use --cuda to enable GPU support.
mission *args:
  #!/usr/bin/env bash
  FEATURES=""
  for arg in "$@"; do
    if [ "$arg" == "--cuda" ]; then
      FEATURES="--features cuda"
    fi
  done
  # We don't pass features to just build/test/quality directly as they are separate recipes,
  # but we want to ensure the build/test runs with the right features if requested.
  if [ -n "$FEATURES" ]; then
    just frontend-install
    cargo build $FEATURES
    cargo fmt --all --check
    cargo clippy --all-targets $FEATURES -- -D warnings
    cargo nextest run --no-tests pass $FEATURES
    just frontend-quality
    just frontend-test
    just frontend-e2e
  else
    just build quality test
  fi
  echo "Mission verified."

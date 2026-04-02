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

# Run tests.
website-install:
  npm --prefix website ci

# Run website verification checks.
website-quality:
  npm --prefix website run lint

# Run website build verification.
website-test:
  npm --prefix website run test

# Run website browser e2e verification.
website-e2e:
  npm --prefix website run e2e

# Run tests.
test:
  just website-install
  cargo nextest run --no-tests pass
  just website-test
  just website-e2e

# Run quality checks.
quality:
  just website-install
  cargo fmt --all --check
  cargo clippy --all-targets --all-features -- -D warnings
  just website-quality

# Run the paddles CLI. Use --cuda to enable GPU support.
paddles *args:
  #!/usr/bin/env bash
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
    just website-install
    cargo build $FEATURES
    cargo fmt --all --check
    cargo clippy --all-targets $FEATURES -- -D warnings
    cargo nextest run --no-tests pass $FEATURES
    just website-quality
    just website-test
  else
    just build quality test
  fi
  echo "Mission verified."

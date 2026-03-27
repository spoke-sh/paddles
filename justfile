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
test:
  cargo nextest run --no-tests pass

# Run quality checks.
quality:
  cargo fmt --all --check
  cargo clippy --all-targets --all-features -- -D warnings

# Run the keel CLI with arguments.
keel *args:
  keel {{args}}

# Run the paddles CLI. Use --cuda to enable GPU support.
paddles *args:
  #!/usr/bin/env bash
  FEATURES=""
  PASSED_ARGS=()
  for arg in {{args}}; do
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
  for arg in {{args}}; do
    if [ "$arg" == "--cuda" ]; then
      FEATURES="--features cuda"
    fi
  done
  # We don't pass features to just build/test/quality directly as they are separate recipes,
  # but we want to ensure the build/test runs with the right features if requested.
  if [ -n "$FEATURES" ]; then
    cargo build $FEATURES
    cargo fmt --all --check
    cargo clippy --all-targets $FEATURES -- -D warnings
    cargo nextest run --no-tests pass $FEATURES
  else
    just build quality test
  fi
  echo "Mission verified."

set positional-arguments

# Show available recipes.
default:
  @just --list

# Build the project.
build:
  cargo build

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

# Run the paddles CLI.
paddles *args:
  cargo run -- {{args}}

# Standard mission path for verification.
mission: build quality test
  @echo "Mission verified."

# Build with dev symbols (non production)
build-dev:
	cargo build --verbose

# Run clippy to lint the project:
lint:
	cargo clippy --all-targets --all-features -- -D warnings

lint-fix:
	cargo clippy --fix --all-targets --all-features -- -D warnings

# Run all the unit tests
test-unit:
	cargo test --lib --profile test --verbose

# Run all of the integration tests
test-integration:
	cargo test --bin juniper_relay_helpers_test --profile test

# Run all of the tests together
test: test-unit test-integration

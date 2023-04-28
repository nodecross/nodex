CARGO = cargo
ACT = act
GRCOV = grcov

build:
	$(CARGO) build --release

lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

test:
	LLVM_PROFILE_FILE='coverage/target-%p-%m.profraw' RUSTFLAGS='-Zinstrument-coverage' $(CARGO) test --verbose

test-coverage: test
	LLVM_PROFILE_FILE='coverage/target-%p-%m.profraw' $(GRCOV) . --source-dir . --binary-path target/debug --output-type lcov --branch --ignore-not-existing

test-gh-actions:
	$(ACT)

.PHONY: build

CARGO = cargo
ACT = act
GRCOV = grcov

lint:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --all-targets --all-features -- -D warnings

test:
	LLVM_PROFILE_FILE='coverage/target-%p-%m.profraw' RUSTFLAGS='-C instrument-coverage' $(CARGO) test

test-gh-actions:
	$(ACT)

build:
	$(CARGO) build --release

.PHONY: build

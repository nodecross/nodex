CARGO = cargo
ACT = act
GRCOV = grcov

build:
	$(CARGO) build --release

lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

test:
	LLVM_PROFILE_FILE='coverage/target-%p-%m.profraw' RUSTFLAGS='-Zinstrument-coverage' $(CARGO) test

test-gh-actions:
	$(ACT)

.PHONY: build

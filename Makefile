CARGO = cargo
ACT = act
GRCOV = grcov

build-windows:
	$(CARGO) build --release --target x86_64-pc-windows-msvc

build-linux:
	$(CARGO) build --release --target x86_64-unknown-linux-musl

build-macos:
	$(CARGO) build --release --target x86_64-apple-darwin

build: build-linux build-macos

lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

test:
	LLVM_PROFILE_FILE='coverage/target-%p-%m.profraw' RUSTFLAGS='-Zinstrument-coverage' $(CARGO) test --verbose

test-coverage: test
	LLVM_PROFILE_FILE='coverage/target-%p-%m.profraw' $(GRCOV) . --source-dir . --binary-path target/debug --output-type lcov --branch --ignore-not-existing

test-gh-actions:
	$(ACT)

.PHONY: build

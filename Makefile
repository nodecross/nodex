CARGO = cargo
ACT = act
GRCOV = grcov

build-renesas-ra6m5:
	$(CARGO) build --release -Zbuild-std --verbose --target thumbv8m.main-none-eabihf

build-renesas: build-renesas-ra6m5

build: build-renesas

lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

test:
	LLVM_PROFILE_FILE='coverage/target-%p-%m.profraw' RUSTFLAGS='-Zinstrument-coverage' $(CARGO) test --verbose

test-coverage: test
	LLVM_PROFILE_FILE='coverage/target-%p-%m.profraw' $(GRCOV) . --source-dir . --binary-path target/debug --output-type lcov --branch --ignore-not-existing --ignore "/*" --output-path coverage/target.lcov

test-gh-actions:
	$(ACT)

.PHONY: build

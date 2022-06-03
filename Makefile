CARGO = cargo
ACT = act
GRCOV = grcov

build-windows:
	$(CARGO) build --release -Zbuild-std --verbose --target x86_64-pc-windows-msvc

build-linux:
	$(CARGO) build --release -Zbuild-std --verbose --target x86_64-unknown-linux-gnu

build-macos:
	$(CARGO) build --release -Zbuild-std --verbose --target x86_64-apple-darwin

build-renesas-ra6m5:
	$(CARGO) build --release -Zbuild-std --verbose --target thumbv8m.main-none-eabihf

build-renesas-ra6m3:
	$(CARGO) build --release -Zbuild-std --verbose --target ./bindings/renesas/renesas_ra6m3.json

build-renesas: build-renesas-ra6m3 build-renesas-ra6m5

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

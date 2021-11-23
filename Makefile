CARGO = cargo
ACT = act

build-renesas-ra6m3:
	$(CARGO) build --release -Zbuild-std --verbose --target ./bindings/renesas/renesas_ra6m3.json

build-renesas: build-renesas-ra6m3

build: build-renesas

test:
	RUSTFLAGS='-Zinstrument-coverage' $(CARGO) test --verbose

test-gh-actions:
	$(ACT)

.PHONY: build

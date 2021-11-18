CARGO = cargo

build-renesas-ra6m3:
	$(CARGO) build --release -Zbuild-std --target ./bindings/renesas/renesas_ra6m3.json

build-renesas: build-renesas-ra6m3

build: build-renesas

test:
	$(CARGO) test

.PHONY: build

CARGO   ?= cargo
BINARY  := remote-control
VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

# Cross-compilation targets
TARGETS := x86_64-unknown-linux-gnu \
           x86_64-pc-windows-gnu \
           aarch64-unknown-linux-gnu \
           x86_64-apple-darwin \
           aarch64-apple-darwin

##@ Build

.PHONY: build
build: ## Debug build
	$(CARGO) build

.PHONY: release
release: ## Release build
	$(CARGO) build --release

##@ Quality

.PHONY: test
test: ## Run tests
	$(CARGO) test

.PHONY: lint
lint: ## Run clippy and format check
	$(CARGO) clippy -- -D warnings
	$(CARGO) fmt --check

.PHONY: fmt
fmt: ## Auto-format code
	$(CARGO) fmt

##@ Install / Clean

.PHONY: install
install: ## Install to ~/.cargo/bin
	$(CARGO) install --path .

.PHONY: clean
clean: ## Remove build artifacts
	$(CARGO) clean

##@ Cross-compilation

.PHONY: cross
cross: $(addprefix cross-,$(TARGETS)) ## Build for all targets

cross-%: ## Build for a specific target (e.g. make cross-aarch64-unknown-linux-gnu)
	$(CARGO) build --release --target $*
	@echo "-> target/$*/release/$(BINARY)"

##@ Release packaging

.PHONY: dist
dist: ## Build and package release archives for all targets
	@mkdir -p dist
	@for t in $(TARGETS); do \
		echo "==> $$t"; \
		$(CARGO) build --release --target $$t || continue; \
		if echo $$t | grep -q windows; then \
			cp target/$$t/release/$(BINARY).exe dist/$(BINARY)-$(VERSION)-$$t.exe; \
		else \
			cp target/$$t/release/$(BINARY) dist/$(BINARY)-$(VERSION)-$$t; \
		fi; \
	done
	@echo "Artifacts in dist/"

##@ Help

.PHONY: help
help: ## Show this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} \
		/^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } \
		/^[a-zA-Z_%-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

.DEFAULT_GOAL := help

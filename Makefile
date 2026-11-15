.PHONY: build test props lint fmt fmt-check cover archcheck doccheck deny tools check

# Toolchain comes from rust-toolchain.toml; every cargo call below uses it.

build:
	cargo build --workspace --all-targets

# Unit, integration and doc tests (benches are compiled by make build and make lint).
test:
	cargo test --workspace

# Property tests with many more cases than the default 256 (proptest). A failure writes
# the shrunk case to the crate's proptest-regressions/; commit it with the fix so
# make test replays it. CI runs this on Linux.
PROPTEST_CASES ?= 10000
props:
	PROPTEST_CASES=$(PROPTEST_CASES) cargo test --workspace --release

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

# Line coverage with cargo-llvm-cov; fails under 80% of lines overall (AGENTS.md).
cover:
	cargo llvm-cov --workspace --fail-under-lines 80 --summary-only

# Crate layer rules (AGENTS.md): which crates may depend on which, and which may do I/O.
archcheck:
	cargo xtask archcheck

# Every .PHONY target has a row in the AGENTS.md Commands table; every folder has a README.md.
doccheck:
	cargo xtask doccheck

# Licences, advisories, banned crates and sources (deny.toml).
deny:
	cargo deny check

# One-time install of the cargo tools make check and make cover need.
tools:
	cargo install --locked cargo-deny cargo-llvm-cov

check: fmt lint archcheck doccheck deny test

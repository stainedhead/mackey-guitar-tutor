.PHONY: build test props lint fmt fmt-check cover archcheck doccheck deny tools check ui-lint ui-test e2e bindings bindings-check app ios-sim screenshots

# Toolchain comes from rust-toolchain.toml; every cargo call below uses it.

# tauri::generate_context! fails to compile when the frontendDist folder is missing, so the
# Rust targets create a stub ui/dist/index.html when no real UI build exists (D-15). A real
# `npm --prefix ui run build` (make app, make ios-sim) replaces it.
UI_STUB := ui/dist/index.html
$(UI_STUB):
	mkdir -p ui/dist
	echo '<!doctype html><title>Mackey Guitar Tutor (UI not built)</title>' > $(UI_STUB)

# UI dependencies, installed from the lockfile whenever it changes.
ui/node_modules: ui/package-lock.json
	npm ci --prefix ui
	touch ui/node_modules

build: $(UI_STUB)
	cargo build --workspace --all-targets

# Unit, integration and doc tests (benches are compiled by make build and make lint).
test: $(UI_STUB)
	cargo test --workspace

# Property tests with many more cases than the default 256 (proptest). A failure writes
# the shrunk case to the crate's proptest-regressions/; commit it with the fix so
# make test replays it. CI runs this on Linux.
PROPTEST_CASES ?= 10000
props: $(UI_STUB)
	PROPTEST_CASES=$(PROPTEST_CASES) cargo test --workspace --release

lint: $(UI_STUB)
	cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

# Line coverage with cargo-llvm-cov; fails under 80% of lines overall (AGENTS.md).
# Excluded as thin glue that needs a window system or the Tauri runtime (D-16): the desktop
# entry point, the build script and the runtime builder; and the cpal glue of mackey-audio that
# needs a sound card (device listing, output and input streams and their threads, D-86). The
# decisions inside that glue live in io/select.rs and io/pump.rs, which stay measured.
COVER_IGNORE := src-tauri/(src/main|src/runtime|build)\.rs|crates/mackey-audio/src/io/(devices|input|output)\.rs
cover: $(UI_STUB)
	cargo llvm-cov --workspace --fail-under-lines 80 --summary-only --ignore-filename-regex '$(COVER_IGNORE)'

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

# UI checks: ESLint (typed, strict), Prettier --check and tsc --noEmit.
ui-lint: ui/node_modules
	npm --prefix ui run lint
	npm --prefix ui run typecheck

# UI unit tests (Vitest, jsdom, mocked command layer).
ui-test: ui/node_modules
	npm --prefix ui test

# Playwright specs in WebKit (desktop and iPad projects) against the mocked command layer.
# One-time: npx --prefix ui playwright install --with-deps webkit
e2e: ui/node_modules
	npm --prefix ui run e2e

# Regenerate the TypeScript bindings (ts-rs) into ui/src/bindings.
# ts-rs writes one file per type (core crates enable their `ts` feature through mackey-app);
# the same run writes commands.ts, events.ts and the recorded mock scenarios (D6, D14).
bindings: $(UI_STUB)
	cargo test -p mackey-app -p mackey-theory -p mackey-voicing -p mackey-audio --lib export_bindings

# Fails when the committed bindings differ from what the Rust types generate.
bindings-check: bindings
	@if [ -n "$$(git status --porcelain -- ui/src/bindings ui/src/mocks/scenarios.json)" ]; then \
		echo "ui/src/bindings or ui/src/mocks/scenarios.json is stale: run 'make bindings' and commit the result"; \
		git status --short -- ui/src/bindings ui/src/mocks/scenarios.json; \
		exit 1; \
	fi

# The app. On macOS: a universal (arm64 + x86_64) .app and .dmg, ad-hoc signed
# (bundle.macOS.signingIdentity "-"). Elsewhere: a host binary with no bundle, as a smoke build.
APP_ARGS ?=
# The Tauri CLI looks for src-tauri/ below the current folder, so run it from the repo root
# (npm --prefix ui run would start it in ui/).
# The Tauri CLI runs through the root package.json's "tauri" script. tauri ios init writes the
# command that started it into the Xcode "Build Rust Code" phase, which runs from
# src-tauri/gen/apple; "npm run tauri" still works there (npm finds the root package.json),
# while a node_modules/.bin path is recorded relative to the repo root and breaks.
TAURI := npm run --silent tauri --
app: ui/node_modules
ifeq ($(shell uname -s),Darwin)
	$(TAURI) build --target universal-apple-darwin $(APP_ARGS)
else
	@echo "Not macOS: building the host binary without a bundle (macOS is the real target)"
	$(TAURI) build --no-bundle $(APP_ARGS)
endif

# Compile the app for the iOS Simulator (no Apple account, unsigned). macOS with Xcode only.
# Generates src-tauri/gen/apple on first run (git-ignored, never hand-edited), then runs the
# debug simulator build; the CLI's before-build hook builds the frontend.
ios-sim: ui/node_modules
	test -d src-tauri/gen/apple || $(TAURI) ios init --ci
	$(TAURI) ios build --debug --target aarch64-sim --ci $(APP_ARGS)

# Screenshots of the real app (Rust core, WebKit webview) under Xvfb: Linux only, optional, not part
# of make check or CI. Needs xvfb, xdotool, imagemagick and openbox (see scripts/screenshots/).
# Writes docs/images/real-*.png.
screenshots: ui/node_modules
	npm --prefix ui run build
	cargo build -p mackey-app --features tauri/custom-protocol
	scripts/screenshots/capture.sh

check: fmt lint archcheck doccheck deny test ui-lint ui-test bindings-check

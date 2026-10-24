.DEFAULT_GOAL := help
.PHONY: help test version ensure-clean fetch-dependencies check-pins check-production-graph \
        install-tools tools-check install-host-tools host-tools-check install-ic-tools ic-tools-check test-tools test-failure-evidence \
        lint-tooling test-hooks test-tooling test-fixture-cell validate validate-toolchain wasm-size \
        install-runtime-server runtime-server-check test-runtime \
        install-release-tools release-tools-check \
        test-release-adapters test-package-admission validation-compilers-check \
        release-version release-preflight release-verify release-prepare-version \
        release-prepared-check release-files release-commit-check release-committed-check \
        release-tagged-check release-push-check qualify-release package publish publish-dry-run

export RELEASE_DELIVERY ?= direct
ifneq ($(filter release-%,$(MAKECMDGOALS)),)
ifneq ($(RELEASE_DELIVERY),direct)
$(error ic-persistence release adapters support only RELEASE_DELIVERY=direct)
endif
endif

# Bootstrap from the repository's simple, checked-in toolchain declaration.
# The Rust helper parses the full TOML for its own compiler identity checks.
VALIDATION_TOOLCHAIN ?= $(shell sed -n 's/^channel = "\([^"]*\)"$$/\1/p' rust-toolchain.toml)
include ci/release-tools.env
include ci/runtime-tools.env
TOOL := bash scripts/dev/run-repo-tool.sh $(VALIDATION_TOOLCHAIN)
override SHARED_TOOLING_ROOT := $(CURDIR)
# Both maintained workspaces retain the same complete formatter sequence.
FORMAT_WRITE_COMMAND = bash -ec 'cargo "+$$1" sort --workspace --grouped; \
    cargo "+$$1" sort --workspace --grouped testing/runtime-qualification; \
    cargo "+$$1" fmt --all; rustfmt "+$$1" --edition 2024 testing/package-consumer.rs; \
    cargo "+$$1" fmt --manifest-path testing/runtime-qualification/Cargo.toml --all' -- "$(VALIDATION_TOOLCHAIN)"
FORMAT_CHECK_COMMAND = bash -ec 'cargo "+$$1" sort --workspace --grouped --check; \
    cargo "+$$1" sort --workspace --grouped --check testing/runtime-qualification; \
    cargo "+$$1" fmt --all -- --check; rustfmt "+$$1" --edition 2024 --check testing/package-consumer.rs; \
    cargo "+$$1" fmt --manifest-path testing/runtime-qualification/Cargo.toml --all -- --check' -- "$(VALIDATION_TOOLCHAIN)"
# Memory uses IC executables for installed runtime qualification.
IC_TOOLS := wasm-opt
LOCAL_TOOL_INSTALL_TARGETS += install-ic-tools install-release-tools
LOCAL_TOOL_CHECK_TARGETS += ic-tools-check release-tools-check
include make/consumer.mk
export PATH := $(CURDIR)/.tools/ic/bin:$(PATH)
fmt fmt-check format-tools-check: export RUSTUP_TOOLCHAIN := $(VALIDATION_TOOLCHAIN)

# The published Testkit CLI owns PocketIC selection, admission and lifecycle.
# Keep this tool graph outside both maintained workspace lockfiles.
TESTKIT_CLI := RUSTUP_TOOLCHAIN=$(VALIDATION_TOOLCHAIN) bash scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-testkit --version $(IC_TESTKIT_VERSION) --bin ic-testkit-server --profile release --features cli
TESTKIT_SERVER_DIRECTORY := $(CURDIR)/.tools/ic-testkit-server

help:
	@echo 'Setup: install-tools (host, IC, Cargo tools), fetch-dependencies, install-hooks; prepare the Rust toolchain first.'
	@echo 'Focused checks: shared-tooling-check, tools-check, test-tools, test-failure-evidence, check-pins, check-production-graph, test-tooling, test-fixture-cell, test-release-adapters, test-hooks, fmt-check, lint-tooling.'
	@echo 'Formatting: fmt. Full release checks: validate, validate-toolchain.'
	@echo 'Installed runtime: install-runtime-server (explicit Testkit setup), runtime-server-check (offline), test-runtime (prepared server and locked caches).'
	@echo 'Report: cloc (root workspace; CLOC_MANIFEST selects an independent Cargo manifest). Fleet reports run in Shared Tooling.'
	@echo 'Maintainer releases: release-patch, release-minor, release-major; normal targets recover unfinished releases.'

# Explicit preparation; normal checks only admit the selected published editor.
install-release-tools:
	+RUSTUP_TOOLCHAIN=$(VALIDATION_TOOLCHAIN) bash scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-build --version $(IC_BUILD_RELEASE_METADATA_VERSION) --bin ic-build-release-metadata --profile debug --features cli

release-tools-check:
	+RUSTUP_TOOLCHAIN=$(VALIDATION_TOOLCHAIN) bash scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-build --version $(IC_BUILD_RELEASE_METADATA_VERSION) --bin ic-build-release-metadata --profile debug --features cli --check

# Explicit network setup; ordinary tool/library gates never provision a server.
install-runtime-server: host-tools-check
	+@cli="$$($(TESTKIT_CLI))" && "$$cli" setup --directory "$(TESTKIT_SERVER_DIRECTORY)"

# Both CLI receipt/bytes and server archive/bytes are checked without downloads.
runtime-server-check:
	+@{ cli="$$($(TESTKIT_CLI) --check)" && "$$cli" check --directory "$(TESTKIT_SERVER_DIRECTORY)"; } || \
	  { status=$$?; echo 'Prepare the selected CLI and server with make install-runtime-server.' >&2; exit "$$status"; }

# Focused installed IO/upgrade qualification, separate from the library gates.
# Build before launch so compilation does not consume the server lifetime.
test-runtime: runtime-server-check ic-tools-check
	+cargo +$(VALIDATION_TOOLCHAIN) build --locked --offline --profile wasm-size --target wasm32-unknown-unknown --example wasm-io-qualification --example wasm-storage-qualification
	+cargo +$(VALIDATION_TOOLCHAIN) build --locked --offline --manifest-path testing/runtime-qualification/Cargo.toml --target-dir target/runtime-qualification
	@mkdir -p target/qualification/runtime
	+@cli="$$($(TESTKIT_CLI) --check)" && \
	  server="$$("$$cli" check --directory "$(TESTKIT_SERVER_DIRECTORY)")" && \
	  optimizer="$(CURDIR)/.tools/ic/bin/wasm-opt" && \
	  evidence="$$(mktemp -d target/qualification/runtime/attempt.XXXXXX)" && \
	  shasum -a 256 Cargo.toml Cargo.lock .shared-tooling.snapshot ci/runtime-tools.env "$(IC_TOOL_PINS)" \
	    testing/runtime-qualification/Cargo.toml testing/runtime-qualification/Cargo.lock \
	    target/wasm32-unknown-unknown/wasm-size/examples/wasm_io_qualification.wasm \
	    target/wasm32-unknown-unknown/wasm-size/examples/wasm_storage_qualification.wasm \
	    target/runtime-qualification/debug/ic-memory-runtime-qualification "$$cli" "$$server" "$$optimizer" > "$$evidence/inputs.sha256" && \
	  for optimization in original O3 Os Oz; do \
	    wasm="$(CURDIR)/$$evidence/$$optimization.wasm"; \
	    if [ "$$optimization" = original ]; then \
	      cp target/wasm32-unknown-unknown/wasm-size/examples/wasm_io_qualification.wasm "$$wasm" || exit "$$?"; \
	    else \
	      "$$optimizer" "$$evidence/original.wasm" --enable-bulk-memory-opt "-$$optimization" -o "$$wasm" > "$$evidence/$$optimization.optimizer.log" 2>&1 || \
	        { status=$$?; cat "$$evidence/$$optimization.optimizer.log"; exit "$$status"; }; \
	    fi; \
	    storage_wasm="$(CURDIR)/$$evidence/$$optimization.storage.wasm"; \
	    if [ "$$optimization" = original ]; then \
	      cp target/wasm32-unknown-unknown/wasm-size/examples/wasm_storage_qualification.wasm "$$storage_wasm" || exit "$$?"; \
	    else \
	      "$$optimizer" "$$evidence/original.storage.wasm" --enable-bulk-memory-opt "-$$optimization" -o "$$storage_wasm" > "$$evidence/$$optimization.storage.optimizer.log" 2>&1 || \
	        { status=$$?; cat "$$evidence/$$optimization.storage.optimizer.log"; exit "$$status"; }; \
	    fi; \
	    shasum -a 256 "$$wasm" "$$storage_wasm" >> "$$evidence/inputs.sha256" || exit "$$?"; \
	    echo "Runtime qualification: $$optimization ($$evidence)"; \
	    env -u POCKET_IC_BIN -u IC_TESTKIT_POCKET_IC_URL IC_MEMORY_QUALIFICATION_WASM="$$wasm" IC_STORAGE_QUALIFICATION_WASM="$$storage_wasm" \
	      "$$cli" run --directory "$(TESTKIT_SERVER_DIRECTORY)" --ttl 900 \
	      --server-stdout "$$evidence/$$optimization.server.stdout" --server-stderr "$$evidence/$$optimization.server.stderr" \
	      -- "$(CURDIR)/target/runtime-qualification/debug/ic-memory-runtime-qualification" > "$$evidence/$$optimization.runtime.log" 2>&1; \
	    status=$$?; cat "$$evidence/$$optimization.runtime.log" || exit "$$?"; \
	    [ "$$status" = 0 ] || exit "$$status"; \
	  done && cat "$$evidence/inputs.sha256"

test-failure-evidence:
	bash scripts/ci/test-failure-evidence.sh $${FAILURE_EVIDENCE_DIRECTORY:+"$$FAILURE_EVIDENCE_DIRECTORY"}

# Memory owns its archive bindings and fixture retention. Generic installer,
# runner, Make/LOC and parser matrices qualify at Shared Tooling's owner.
test-tools: test-failure-evidence
	bash scripts/ci/test-consumer-fixture-retention.sh

# Network preparation is separate from offline checks; preserve tracked locks.
fetch-dependencies:
	+cargo +$(VALIDATION_TOOLCHAIN) fetch --locked

check-pins:
	+RUSTUP_TOOLCHAIN=$(VALIDATION_TOOLCHAIN) RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true bash scripts/ci/check-dependency-pins.sh --cargo-inheritance

# Product policy stays here; Shared observes the locked normal/build graph.
check-production-graph:
	+@substrate="$$(cargo +$(VALIDATION_TOOLCHAIN) pkgid --locked --offline -p ic-stable-structures)" && \
	  [ "$$substrate" = 'registry+https://github.com/rust-lang/crates.io-index#ic-stable-structures@0.7.2' ] || \
	  { printf 'Unsupported stable-memory substrate identity: %s\n' "$$substrate" >&2; exit 1; }
	+RUSTUP_TOOLCHAIN=$(VALIDATION_TOOLCHAIN) RUSTUP_AUTO_INSTALL=0 perl scripts/ci/check-cargo-graph.pl \
		--consumer "$(CURDIR)" --package ic-storage --package ic-metrics-stable --target wasm32-unknown-unknown \
		--forbid ic-build --forbid-pattern '^ic-host-' --forbid-pattern '^ic-testkit(?:-|$$)' \
		--forbid-pattern '^clap(?:_|$$)' \
		--forbid pocket-ic --forbid serde_json --forbid toml --forbid trybuild \
		--forbid-feature ic-memory/test-support \
		--require ic-stable-structures=1 --require ic-memory=1 --require ic-storage=1 --require ic-platform=1 --require ic-metrics=1

# Install exact, checksum-verified tools separately; these checks are offline.
lint-tooling:
	$${ACTIONLINT_BIN:-actionlint} .github/workflows/*.yml
	$${SHELLCHECK_BIN:-shellcheck} --shell=bash ci/release-tools.env ci/runtime-tools.env ci/tool-versions.env scripts/ci/*.sh scripts/dev/*.sh .githooks/pre-commit

test-tooling:
	+cargo +$(VALIDATION_TOOLCHAIN) test --locked --offline --example repo-tool

test-release-adapters:
	bash scripts/ci/test-release-adapters.sh
	bash scripts/ci/test-consolidated-bootstrap.sh

validation-compilers-check:
	bash scripts/ci/check-validation-compilers.sh $(VALIDATION_TOOLCHAIN)

test-package-admission:
	bash scripts/ci/test-validation-compilers.sh
	bash scripts/ci/test-package-admission.sh

test-hooks: format-tools-check
	+CDPATH="$(CURDIR)" RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true bash scripts/ci/test-git-hooks.sh

test:
	+cargo +$(VALIDATION_TOOLCHAIN) test --workspace --locked --offline -- --test-threads=1

test-fixture-cell:
	+cargo +$(VALIDATION_TOOLCHAIN) clippy --locked --offline --lib --tests --features test-support -- -D warnings
	+cargo +$(VALIDATION_TOOLCHAIN) test --locked --offline --lib --features test-support testing:: -- --test-threads=1
	+cargo +$(VALIDATION_TOOLCHAIN) check --locked --offline --target wasm32-unknown-unknown --lib --features test-support

version:
	+@$(TOOL) version

ensure-clean:
	+@$(TOOL) ensure-clean

# Full release suite; release/publication remain separately authorized.
validate:
	$(MAKE) --no-print-directory validate-toolchain
	+msrv="$$($(TOOL) msrv)" && \
	  cargo "+$$msrv" check --workspace --locked --offline --all-targets

validate-toolchain:
	$(MAKE) --no-print-directory shared-tooling-check tools-check format-tools-check
	$(MAKE) --no-print-directory validation-compilers-check
	+cargo +$(VALIDATION_TOOLCHAIN) clippy --workspace --locked --offline --all-targets -- -D warnings
	$(MAKE) --no-print-directory check-production-graph check-pins check-storage-boundary check-license check-doc-links test-release-adapters
	$(MAKE) --no-print-directory test-tools test-tooling test-hooks fmt-check test-fixture-cell test-package-admission
	# test-tooling already ran the repo-tool fixtures in parallel; keep product tests serial.
	+cargo +$(VALIDATION_TOOLCHAIN) test --workspace --locked --offline -- --test-threads=1 --skip repo_tool::tests::
	+RUSTDOCFLAGS='-D warnings' cargo +$(VALIDATION_TOOLCHAIN) doc --workspace --locked --offline --no-deps
	+cargo +$(VALIDATION_TOOLCHAIN) check --workspace --locked --offline --target wasm32-unknown-unknown --tests
	$(MAKE) --no-print-directory wasm-size
	# Validate working-tree edits; release admission separately requires clean source.
	+bash scripts/ci/package-workspace.sh $(VALIDATION_TOOLCHAIN) --allow-dirty

wasm-size:
	+cargo +$(VALIDATION_TOOLCHAIN) build --locked --offline --profile wasm-size --target wasm32-unknown-unknown \
		--example wasm-core-size-probe --example wasm-diagnostics-size-probe --example wasm-admission-size-probe \
		--example wasm-runtime-integration-size-probe
	+@$(TOOL) wasm-size

# Preserve release-only cache preparation for all caller settings.
# GNU Make 3.81 cannot combine target-specific override and export. Reclassify
# caller values first so these exported target policies can supersede them.
ifneq ($(origin RELEASE_CACHE_PREPARE),undefined)
override RELEASE_CACHE_PREPARE := $(RELEASE_CACHE_PREPARE)
endif
release-patch release-minor release-major release-resume: export RELEASE_CACHE_PREPARE := 1

release-version:
	+@$(TOOL) version

release-preflight release-prepare-version release-prepared-check release-files release-commit-check release-committed-check release-tagged-check release-push-check:
	+@$(TOOL) $@

# Preserve every gate attempt, including failures. The adapter owns the gate;
# these logs use Cargo's selected target directory, including overrides.
release-verify:
	+@set -eu; \
	log_dir="$$($(TOOL) target)/release-validation/attempts"; \
	mkdir -p "$$log_dir"; \
	log_file="$$(mktemp "$$log_dir/verify.XXXXXX")"; \
	if $(TOOL) release-verify > "$$log_file" 2>&1; then result=0; else result=$$?; fi; \
	cat "$$log_file"; \
	echo "Retained full-gate log: $$log_file"; \
	exit "$$result"

qualify-release:
	+$(TOOL) qualify-release

package: ensure-clean
	+bash scripts/ci/package-workspace.sh $(VALIDATION_TOOLCHAIN)

publish:
	+$(TOOL) publish

publish-dry-run:
	+$(TOOL) publish --dry-run

.PHONY: check-storage-boundary check-license check-doc-links
check-storage-boundary:
	+RUSTUP_TOOLCHAIN=$(VALIDATION_TOOLCHAIN) RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$(CURDIR)" --external-ic

check-license:
	perl scripts/ci/check-license.pl --kind mit --file LICENSE
	cmp LICENSE crates/ic-memory/LICENSE
	perl scripts/ci/check-license.pl --kind mit --file crates/ic-storage/LICENSE
	perl scripts/ci/check-license.pl --kind mit --file crates/ic-metrics-stable/LICENSE

check-doc-links:
	perl scripts/ci/check-documentation-links.pl --root "$(CURDIR)" README.md AGENTS.md docs/status/current.md docs/migration/ic-persistence.md docs/migration/ic-metrics-stable.md docs/storage/upstreams.md crates/ic-storage/README.md crates/ic-metrics-stable/README.md docs/archive/private-btree/*.md docs/history/ic-storage/index.md

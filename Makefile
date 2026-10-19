.DEFAULT_GOAL := help
SHARED_TOOLING_RUST := 1
export RELEASE_DELIVERY := direct
export RELEASE_DELIVERY_POLICY := $(CURDIR)/ci/release-delivery.txt
include make/consumer.mk
export YQ := $(CURDIR)/.tools/host/bin/yq
release-patch release-minor release-major release-resume: export IC_STORAGE_RELEASE_PREPARE := 1

.PHONY: help version check-doc-links check-license check-pins check-storage-boundary check-release-commands ci
.PHONY: install-release-tools release-tools-check
.PHONY: release-version release-preflight release-verify release-prepare-version
.PHONY: release-prepared-check release-files release-commit-check release-committed-check
.PHONY: release-tagged-check release-push-check

help:
	@echo "Setup: install-tools; offline verification: tools-check"
	@echo "Focused: shared-tooling-check, check-doc-links, check-license, check-pins, check-release-commands"
	@echo "Complete library gate: ci; current version: version"
	@echo "Release editor setup: install-release-tools; offline verification: release-tools-check"
	@echo "Authorized releases: release-patch, release-minor, release-major; recovery: release-resume VERSION=x.y.z"

version release-version:
	@bash scripts/release/adapter.sh version

install-release-tools:
	+bash scripts/release/editor.sh

release-tools-check:
	+bash scripts/release/editor.sh --check > /dev/null

check-doc-links:
	perl scripts/ci/check-documentation-links.pl --root . README.md AGENTS.md CHANGELOG.md docs/status/current.md docs/upstreams.md crates/ic-storage/README.md docs/archive/private-btree/README.md docs/archive/private-btree/icydb-btree.md docs/archive/private-btree/experimental-btree.md docs/archive/private-btree/canonical-workloads.md docs/archive/private-btree/handoff.md docs/archive/private-btree/upstreams.md

check-license:
	perl scripts/ci/check-license.pl --kind mit --file LICENSE
	cmp LICENSE crates/ic-storage/LICENSE

check-pins:
	bash scripts/ci/check-dependency-pins.sh --consumer "$(CURDIR)" --cargo-inheritance

check-storage-boundary:
	bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$(CURDIR)"

check-release-commands:
	bash scripts/ci/check-release-commands.sh . make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk make/execution.mk scripts/ci/check-make-execution.sh scripts/ci/check-release-delivery.sh ci/release-delivery.txt

ci:
	+bash scripts/ci/run-validation-targets.sh --fail-fast shared-tooling-check check-doc-links check-license check-pins check-storage-boundary check-release-commands fmt-check check clippy docs-check test wasm-check msrv package

release-preflight:
	@bash scripts/release/adapter.sh preflight

release-verify:
	+VALIDATION_FAILURE_LOG_DIR="$$(git rev-parse --git-path release-state)/validation-failures" bash scripts/ci/run-validation-targets.sh ci

release-prepare-version:
	@bash scripts/release/adapter.sh prepare

release-prepared-check release-committed-check release-tagged-check release-push-check:
	@bash scripts/release/adapter.sh check

release-commit-check:
	@bash scripts/release/adapter.sh commit-check

release-files:
	@printf 'Cargo.toml\0Cargo.lock\0CHANGELOG.md\0'

.PHONY: check clippy docs-check test wasm-check msrv package publish publish-check
check:
	+cargo check --workspace --all-targets --locked
clippy:
	+cargo clippy --workspace --all-targets --locked -- -D warnings
docs-check:
	+RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
msrv:
	+cargo +1.91.0 check -p ic-storage --all-targets --locked
package:
	+cargo package -p ic-storage --locked
publish publish-check:
	+bash scripts/release/publish.sh "$@"

test:
	+cargo test -p ic-storage --locked
wasm-check:
	+cargo check -p ic-storage --target wasm32-unknown-unknown --locked

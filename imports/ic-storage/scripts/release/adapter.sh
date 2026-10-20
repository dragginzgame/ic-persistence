#!/usr/bin/env bash
set -euo pipefail
operation="${1:-}"
[[ $# -eq 1 ]] || exit 2
bash scripts/ci/check-release-delivery.sh ci/release-delivery.txt "${RELEASE_DELIVERY:-direct}"
prepare="${IC_STORAGE_RELEASE_PREPARE:-0}"
unset IC_STORAGE_RELEASE_PREPARE
case "$operation" in
    version) bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml ;;
    preflight)
        bash scripts/ci/check-release-source.sh --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md
        current="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
        [[ "$current" == "${RELEASE_PREVIOUS:?}" ]] || exit 1
        awk -v version="${RELEASE_VERSION:?}" -v previous="$current" -v date="${RELEASE_DATE:?}" \
            -f scripts/ci/finalize-release-changelog.awk CHANGELOG.md > /dev/null
        bash scripts/ci/cargo-release-preflight.sh "$PWD" "$prepare" \
            --pin ci/tool-versions.env --pin scripts/release/editor.sh \
            --check tools-check --check release-tools-check \
            --prepare install-tools --prepare install-release-tools \
            --prepare tools-check --prepare release-tools-check
        ;;
    prepare)
        bash scripts/ci/check-release-source.sh --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md
        current="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
        if [[ "$current" == "${RELEASE_VERSION:?}" ]]; then
            bash scripts/ci/cargo-release-metadata.sh check ic-storage --manifest-order-grouped
        else
            CARGO_RELEASE_METADATA_EDITOR="$(bash scripts/release/editor.sh --check)"
            export CARGO_RELEASE_METADATA_EDITOR
            # Bounds each manifest/lock input; current documents are under 8 KiB.
            export CARGO_RELEASE_METADATA_MAX_BYTES=1048576
            bash scripts/ci/cargo-release-metadata.sh prepare ic-storage
        fi
        ;;
    check|commit-check)
        bash scripts/ci/check-release-source.sh --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md
        bash scripts/ci/cargo-release-metadata.sh check ic-storage --manifest-order-grouped
        if [[ "$operation" == commit-check ]]; then
            [[ -z "${RELEASE_COMMIT:-}" ]] || exit 1
            git diff --quiet -- Cargo.toml Cargo.lock CHANGELOG.md
            for path in Cargo.toml Cargo.lock CHANGELOG.md; do git cat-file -e ":$path"; done
        fi
        ;;
    *) echo 'usage: adapter.sh version|preflight|prepare|check|commit-check' >&2; exit 2 ;;
esac

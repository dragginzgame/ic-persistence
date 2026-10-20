#!/usr/bin/env bash
# Qualify the exact two archives without fetching an unpublished dependency.
set -euo pipefail
[[ $# -ge 1 && $# -le 2 ]] || exit 2
pin="$1"
shift
[[ $# -eq 0 || "$1" == --allow-dirty ]] || exit 2
export RUSTUP_AUTO_INSTALL=0
cargo "+$pin" package --workspace --registry crates-io --locked --offline --no-verify "$@"
metadata="$(cargo "+$pin" metadata --no-deps --locked --offline --format-version 1)"
target="$(printf '%s' "$metadata" | jq -er '.target_directory')"
mkdir -p "$target/package-qualification"
attempt="$(mktemp -d "$target/package-qualification/attempt.XXXXXX")"
echo "Retained package qualification: $attempt"
for package in ic-memory ic-storage; do
    version="$(printf '%s' "$metadata" | jq -er --arg name "$package" '.packages[] | select(.name == $name) | .version')"
    archive="$target/package/$package-$version.crate"
    shasum -a 256 "$archive" >> "$attempt/archives.sha256"
    tar -xzf "$archive" -C "$attempt"
    if [[ "$package" == ic-memory ]]; then memory="$attempt/$package-$version"; else storage="$attempt/$package-$version"; fi
done
# A generated workspace isolates extracted manifests from this checkout without
# editing any packaged manifest. Normalized dependency requirements stay intact.
printf '[workspace]\nresolver = "3"\nmembers = ["%s", "%s"]\n' "${memory##*/}" "${storage##*/}" > "$attempt/Cargo.toml"
# The overrides point only at just-extracted, hashed archives. This cannot use
# workspace sources or a previously published Memory with the same version.
perl -MJSON::PP -e '
    my ($memory,$storage,$file)=@ARGV;
    open my $out, ">", $file or die $!;
    print $out "[patch.crates-io]\nic-memory = { path = ", encode_json($memory), " }\n";
    print $out "ic-storage = { path = ", encode_json($storage), " }\n";
    close $out or die $!;
' "$memory" "$storage" "$attempt/patch.toml"
for package in "$memory" "$storage"; do
    cargo "+$pin" check --offline --manifest-path "$package/Cargo.toml" --config "$attempt/patch.toml" --target-dir "$attempt/build"
done
mkdir -p "$attempt/consumer/src"
cp testing/package-consumer.rs "$attempt/consumer/src/main.rs"
perl -MJSON::PP -e '
    my ($memory,$storage,$file)=@ARGV;
    open my $out, ">", $file or die $!;
    print $out "[workspace]\n[package]\nname = \"persistence-package-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\nrust-version = \"1.91\"\n[dependencies]\n";
    print $out "ic-memory = { path = ",encode_json($memory),", features = [\"test-support\"] }\n";
    print $out "ic-storage = { path = ",encode_json($storage)," }\n";
    close $out or die $!;
' "$memory" "$storage" "$attempt/consumer/Cargo.toml"
# Lock only the disposable consumer graph; maintained lockfiles never change.
cargo "+$pin" generate-lockfile --offline --manifest-path "$attempt/consumer/Cargo.toml" --config "$attempt/patch.toml"
for compiler in "$pin" 1.91.0; do
    cargo "+$compiler" run --locked --offline --manifest-path "$attempt/consumer/Cargo.toml" --config "$attempt/patch.toml" --target-dir "$attempt/consumer-build"
done
cargo "+$pin" check --locked --offline --target wasm32-unknown-unknown --manifest-path "$attempt/consumer/Cargo.toml" --config "$attempt/patch.toml" --target-dir "$attempt/consumer-build"
shasum -a 256 -c "$attempt/archives.sha256"

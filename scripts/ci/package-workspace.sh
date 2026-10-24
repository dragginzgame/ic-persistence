#!/usr/bin/env bash
# Qualify the exact workspace archives without fetching an unpublished dependency.
set -euo pipefail
[[ $# -ge 1 && $# -le 2 ]] || exit 2
pin="$1"
shift
[[ $# -eq 0 || "$1" == --allow-dirty ]] || exit 2
export RUSTUP_AUTO_INSTALL=0
msrv="$(bash scripts/ci/check-validation-compilers.sh "$pin")"
source_root="$(pwd -P)"
cargo "+$pin" package --workspace --registry crates-io --locked --offline --no-verify "$@"
metadata="$(cargo "+$pin" metadata --no-deps --locked --offline --format-version 1)"
target="$(printf '%s' "$metadata" | jq -er '.target_directory')"
mkdir -p "$target/package-qualification"
attempt="$(mktemp -d "$target/package-qualification/attempt.XXXXXX")"
echo "Retained package qualification: $attempt"
shasum -a 256 Cargo.toml Cargo.lock crates/ic-memory/Cargo.toml crates/ic-storage/Cargo.toml crates/ic-metrics-stable/Cargo.toml > "$attempt/source.sha256"
yq -p toml -o json '.' Cargo.lock > "$attempt/selected-lock.json"
for package in ic-memory ic-storage ic-metrics-stable; do
    version="$(printf '%s' "$metadata" | jq -er --arg name "$package" '.packages[] | select(.name == $name) | .version')"
    archive="$target/package/$package-$version.crate"
    shasum -a 256 "$archive" >> "$attempt/archives.sha256"
    tar -xzf "$archive" -C "$attempt"
    case "$package" in
        ic-memory) memory="$attempt/$package-$version" ;;
        ic-storage) storage="$attempt/$package-$version" ;;
        ic-metrics-stable) metrics="$attempt/$package-$version" ;;
    esac
done
# A generated workspace isolates extracted manifests from this checkout without
# editing any packaged manifest. Normalized dependency requirements stay intact.
printf '[workspace]\nresolver = "3"\nmembers = ["%s", "%s", "%s"]\n' "${memory##*/}" "${storage##*/}" "${metrics##*/}" > "$attempt/Cargo.toml"
# The overrides point only at just-extracted, hashed archives. This cannot use
# workspace sources or a previously published Memory with the same version.
perl -MJSON::PP -e '
    my ($memory,$storage,$metrics,$file)=@ARGV;
    open my $out, ">", $file or die $!;
    print $out "[patch.crates-io]\nic-memory = { path = ", encode_json($memory), " }\n";
    print $out "ic-storage = { path = ", encode_json($storage), " }\n";
    print $out "ic-metrics-stable = { path = ", encode_json($metrics), " }\n";
    close $out or die $!;
' "$memory" "$storage" "$metrics" "$attempt/patch.toml"
# Seed each disposable graph from the selected lock. Cargo may prune unused
# entries or add the fixture root, but external selections must remain exact.
admit_graph() {
    local manifest="$1" directory="$2"
    cp "$source_root/Cargo.lock" "$directory/Cargo.lock"
    cargo "+$pin" metadata --offline --format-version 1 --manifest-path "$manifest" --config "$attempt/patch.toml" > "$directory/metadata.json"
    yq -p toml -o json '.' "$directory/Cargo.lock" | jq -e --slurpfile selected "$attempt/selected-lock.json" '
        def identity: {name, version, source, checksum};
        ($selected[0].package | map(select(.source != null) | identity)) as $allowed |
        [.package[] | select(.source != null) | identity | select(. as $p | $allowed | index($p) | not)] |
        if length == 0 then true else error("archive dependency selection drift: " + tostring) end' > /dev/null
    jq -e --arg memory "$memory/Cargo.toml" --arg storage "$storage/Cargo.toml" --arg metrics "$metrics/Cargo.toml" --arg fixture "$attempt/consumer/Cargo.toml" '
        (([.packages[] | select(.name == "ic-memory")] | length == 1 and .[0].manifest_path == $memory and .[0].source == null) and
        ([.packages[] | select(.name == "ic-storage")] | length == 1 and .[0].manifest_path == $storage and .[0].source == null) and
        ([.packages[] | select(.name == "ic-metrics-stable")] | length == 1 and .[0].manifest_path == $metrics and .[0].source == null) and
        all(.packages[]; .source != null or .manifest_path == $memory or .manifest_path == $storage or .manifest_path == $metrics or .manifest_path == $fixture)) |
        if . then true else error("archive graph must use exactly the extracted Persistence packages") end
    ' "$directory/metadata.json" > /dev/null
}
find "$memory" "$storage" "$metrics" -type f -print0 | xargs -0 shasum -a 256 > "$attempt/unpacked.sha256"
admit_graph "$attempt/Cargo.toml" "$attempt"
for package in "$memory" "$storage" "$metrics"; do
    cargo "+$pin" check --locked --offline --manifest-path "$package/Cargo.toml" --config "$attempt/patch.toml" --target-dir "$attempt/build"
done
mkdir -p "$attempt/consumer/src"
cp testing/package-consumer.rs "$attempt/consumer/src/main.rs"
# Match the public arithmetic types selected by the exact extracted adapter.
metrics_requirement="$(yq -p toml -o yaml -r '.dependencies.ic-metrics.version' "$metrics/Cargo.toml")"
perl -MJSON::PP -e '
    my ($memory,$storage,$metrics,$metrics_requirement,$msrv,$file)=@ARGV;
    open my $out, ">", $file or die $!;
    print $out "[workspace]\n[package]\nname = \"persistence-package-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\nrust-version = \"$msrv\"\n[dependencies]\n";
    print $out "ic-memory = { path = ",encode_json($memory),", features = [\"test-support\"] }\n";
    print $out "ic-storage = { path = ",encode_json($storage)," }\n";
    print $out "ic-metrics-stable = { path = ",encode_json($metrics)," }\nic-metrics = ",encode_json($metrics_requirement),"\n";
    close $out or die $!;
' "$memory" "$storage" "$metrics" "$metrics_requirement" "$msrv" "$attempt/consumer/Cargo.toml"
admit_graph "$attempt/consumer/Cargo.toml" "$attempt/consumer"
for compiler in "$pin" "$msrv"; do
    cargo "+$compiler" run --locked --offline --manifest-path "$attempt/consumer/Cargo.toml" --config "$attempt/patch.toml" --target-dir "$attempt/consumer-build"
done
cargo "+$pin" check --locked --offline --target wasm32-unknown-unknown --manifest-path "$attempt/consumer/Cargo.toml" --config "$attempt/patch.toml" --target-dir "$attempt/consumer-build"
shasum -a 256 -c "$attempt/archives.sha256"
shasum -a 256 -c "$attempt/unpacked.sha256" > "$attempt/unpacked-check.log"
shasum -a 256 -c "$attempt/source.sha256"

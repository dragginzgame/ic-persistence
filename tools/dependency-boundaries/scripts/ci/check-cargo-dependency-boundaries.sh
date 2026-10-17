#!/usr/bin/env bash
# Consumer-owned policy; canonical Cargo package names include renamed dependencies.
set -euo pipefail
[[ $# -eq 2 && "$1" == --consumer ]] || {
    echo 'usage: check-cargo-dependency-boundaries.sh --consumer ROOT' >&2
    exit 2
}
cd -- "$2"
policy=ci/cargo-dependency-boundaries.json
[[ -f "$policy" && ! -L "$policy" ]] || {
    echo "missing or non-regular dependency boundary policy: $policy" >&2
    exit 1
}
CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo metadata --no-deps --locked --format-version 1 |
    perl -MJSON::PP -e '
        use strict;
        use warnings;
        local $/;
        open my $file, "<", $ARGV[0] or die "cannot read boundary policy: $!\n";
        my $policy = decode_json(<$file>);
        ref($policy) eq "ARRAY" && @$policy or die "expected a nonempty policy array\n";
        my %rules;
        for my $rule (@$policy) {
            ref($rule) eq "HASH" && keys(%$rule) == 2
                && exists($rule->{dependency}) && exists($rule->{allowed_packages})
                or die "expected dependency and allowed_packages\n";
            my $name = $rule->{dependency};
            defined($name) && !ref($name) && $name =~ /^[A-Za-z0-9_-]+$/
                or die "invalid dependency name\n";
            exists($rules{$name}) and die "duplicate dependency rule: $name\n";
            ref($rule->{allowed_packages}) eq "ARRAY" or die "expected package array\n";
            my %allowed;
            for my $package (@{$rule->{allowed_packages}}) {
                defined($package) && !ref($package) && $package =~ /^[A-Za-z0-9_-]+$/
                    or die "invalid allowed package\n";
                $allowed{$package}++ and die "duplicate allowed package: $package\n";
            }
            $rules{$name} = \%allowed;
        }
        my $metadata = decode_json(<STDIN>);
        my %members = map { $_ => 1 } @{$metadata->{workspace_members}};
        my @violations;
        for my $package (@{$metadata->{packages}}) {
            next unless $members{$package->{id}};
            for my $dependency (@{$package->{dependencies}}) {
                my $name = $dependency->{name};
                push @violations, "$package->{name} -> $name"
                    if exists($rules{$name}) && !$rules{$name}{$package->{name}};
            }
        }
        die "Forbidden direct Cargo dependencies:\n" . join("\n", sort @violations) . "\n"
            if @violations;
        print "Cargo dependency boundaries passed.\n";
    ' "$policy"

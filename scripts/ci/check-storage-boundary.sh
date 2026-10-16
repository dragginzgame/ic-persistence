#!/usr/bin/env bash
# Check direct workspace dependencies; only the storage infrastructure package may depend on the substrate.
set -euo pipefail
cargo metadata --no-deps --locked --format-version 1 | perl -MJSON::PP -e '
  local $/;
  my $metadata = decode_json(<STDIN>);
  my @violations;
  for my $package (@{$metadata->{packages}}) {
    for my $dependency (@{$package->{dependencies}}) {
      push @violations, $package->{name}
        if $dependency->{name} eq "ic-stable-structures"
          && $package->{name} ne "ic-storage";
    }
  }
  die "Use IC Storage adapters, not direct ic-stable-structures dependencies: @violations\n"
    if @violations;
  print "Storage dependency boundary passed.\n";
'

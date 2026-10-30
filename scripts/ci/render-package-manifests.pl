#!/usr/bin/env perl
# Render disposable workspace, overrides and consumer from one observed package map.
use strict;
use warnings;
use JSON::PP qw(decode_json encode_json);
@ARGV == 5 or die "usage: render-package-manifests.pl map metrics logging msrv output\n";
my ($map,$metrics_requirement,$logging_requirement,$msrv,$directory)=@ARGV;
open my $in, "<", $map or die $!;
my $packages=decode_json(do { local $/; <$in> });
close $in or die $!;
open my $out, ">", "$directory/Cargo.toml" or die $!;
print $out "[workspace]\nresolver = \"3\"\nmembers = [", join(", ", map { encode_json($_->{root}) } @$packages), "]\n";
close $out or die $!;
open $out, ">", "$directory/patch.toml" or die $!;
print $out "[patch.crates-io]\n";
print $out $_->{name}, " = { path = ", encode_json($_->{root}), " }\n" for @$packages;
close $out or die $!;
open $out, ">", "$directory/consumer/Cargo.toml" or die $!;
print $out "[workspace]\n[package]\nname = \"persistence-package-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\nrust-version = \"$msrv\"\n[dependencies]\n";
for my $package (@$packages) {
    print $out $package->{name}, " = { path = ", encode_json($package->{root});
    print $out ", features = [\"test-support\"]" if $package->{name} eq "ic-memory";
    print $out " }\n";
}
print $out "ic-metrics = ", encode_json($metrics_requirement), "\n";
print $out "ic-logging = ", encode_json($logging_requirement), "\n";
close $out or die $!;

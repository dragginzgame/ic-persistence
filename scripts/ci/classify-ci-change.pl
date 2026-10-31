#!/usr/bin/env perl
# Shared companions: docs/ci-change-selection.md
use strict;
use warnings;
use Getopt::Long qw(GetOptions);

# Conservative, offline selection. A failed observation always retains full CI.
my ($base, $head, $event, $ref, $policy);
GetOptions('base=s' => \$base, 'head=s' => \$head, 'event=s' => \$event,
    'ref=s' => \$ref, 'policy=s' => \$policy) && !@ARGV or die "invalid arguments\n";
sub git_read {
    open my $fh, '-|', 'git', @_ or die "cannot start git\n";
    local $/;
    my $value = <$fh>;
    close $fh or die "git observation failed\n";
    return $value // '';
}
sub path_valid {
    my ($path) = @_;
    return $path ne '' && $path !~ m{(?:\A/|[\x00-\x1f\x7f]|(?:\A|/)\.\.?(/|\z)|//|/\z)};
}
sub classify {
    die "unsupported event or ref\n" unless defined $event && defined $ref
        && ($event eq 'pull_request' || $event eq 'push') && $ref =~ m{\Arefs/heads/[^\x00-\x20]+\z};
    die "missing commit identities\n" unless defined $base && defined $head
        && $base =~ /\A[0-9a-f]{40}(?:[0-9a-f]{24})?\z/
        && $head =~ /\A[0-9a-f]{40}(?:[0-9a-f]{24})?\z/;
    die "invalid policy path\n" unless defined $policy && path_valid($policy);
    git_read('cat-file', '-e', "$base^{commit}");
    git_read('cat-file', '-e', "$head^{commit}");
    my $start = $base;
    if ($event eq 'pull_request') {
        $start = git_read('merge-base', $base, $head);
        chomp $start;
        die "ambiguous merge base\n" unless $start =~ /\A[0-9a-f]{40}(?:[0-9a-f]{24})?\z/;
    } else {
        git_read('merge-base', '--is-ancestor', $base, $head);
    }
    for my $revision ($start, $head) {
        my $entry = git_read('ls-tree', '-z', $revision, '--', $policy);
        die "policy must be a regular non-executable file\n"
            unless $entry =~ /\A100644 blob [0-9a-f]+\t\Q$policy\E\0\z/;
    }
    my $rules = git_read('show', "$start:$policy");
    die "policy changed\n" unless $rules eq git_read('show', "$head:$policy");
    my %allowed;
    for my $path (split /\n/, $rules) {
        next if $path eq '' || $path =~ /^#/;
        die "invalid policy entry\n" unless path_valid($path) && $path =~ /\.md\z/
            && $path !~ m{\A(?:\.github|scripts|ci|make)/} && $path ne $policy;
        die "duplicate policy entry\n" if $allowed{$path}++;
    }
    die "empty policy\n" unless %allowed;
    my @records = split /\0/, git_read('diff', '--raw', '-z', '--no-renames', '--no-abbrev',
        '--ignore-submodules=none', '--no-relative', $start, $head, '--');
    die "empty comparison\n" unless @records;
    while (@records) {
        my $metadata = shift @records;
        my $path = shift @records;
        die "unknown diff record\n" unless defined $path && path_valid($path) && $allowed{$path}
            && $metadata =~ /\A:(000000|100644) (000000|100644) [0-9a-f]+ [0-9a-f]+ [AMD]\z/;
    }
    return 'docs';
}
my $scope = eval { classify() };
if (!defined $scope) { warn "Full CI selected: $@"; $scope = 'full'; }
print "$scope\n";

#!/usr/bin/env perl
# Shared companions: licenses/MIT.txt licenses/Dragginz-ICP-Burn-1.0.txt licenses/Dragginz-ICP-Burn-1.1.txt licenses/prior-mit-notice.txt
use strict;
use warnings;
use FindBin;
use Getopt::Long qw(GetOptions);

my ($kind, $file, $prior_mit, $license_version, $archive, $member);
GetOptions('kind=s' => \$kind, 'file=s' => \$file, 'prior-mit' => \$prior_mit,
    'license-version=s' => \$license_version, 'archive=s' => \$archive, 'member=s' => \$member)
    && !@ARGV && defined($kind) && defined($file)
    && ($kind eq 'mit' || $kind eq 'dragginz')
    && (!$prior_mit || $kind eq 'dragginz')
    && (!defined($license_version) || ($kind eq 'dragginz' && $license_version =~ /\A1\.[01]\z/))
    && (defined($archive) == defined($member))
    or die "usage: check-license.pl --kind mit|dragginz --file LICENSE [--prior-mit] [--license-version 1.0|1.1] [--archive PACKAGE.crate --member PACKAGE-VERSION/LICENSE]\n";
$license_version //= '1.1';
my $root = "$FindBin::Bin/../../licenses";

sub read_file {
    my ($path) = @_;
    -f $path && !-l $path or die "not a regular non-symlink license file: $path\n";
    open my $fh, '<', $path or die "cannot read $path: $!\n";
    local $/;
    my $text = <$fh>;
    close $fh or die "cannot close $path: $!\n";
    $text =~ s/\r\n/\n/g unless $_[1];
    return $text;
}

sub normalized {
    my ($text) = @_;
    $text =~ s/\s+/ /g;
    $text =~ s/^ | $//g;
    return $text;
}

sub check_text {
    my ($actual, $template) = @_;
    my ($prefix, $suffix) = split /\Q{{COPYRIGHT_NOTICES}}\E/, $template, -1;
    defined($suffix) or die "invalid canonical template\n";
    # Only the opening copyright block varies. Do not strip notices in clauses.
    $actual =~ /\A\Q$prefix\E((?:Copyright \(c\) [^\n]+\n)+)\n(.*)\z/s
        or die "missing or malformed copyright block in $file\n";
    my ($notices, $body) = ($1, $2);
    $notices !~ /\{\{|\}\}|[\x00-\x08\x0b-\x1f]/
        or die "invalid copyright notice in $file\n";
    normalized($body) eq normalized($suffix)
        or die "license terms differ from canonical $kind template: $file\n";
}

my $expected = read_file($file, 1);
my $actual = $expected;
$actual =~ s/\r\n/\n/g;
if ($prior_mit) {
    my $separator = read_file("$root/prior-mit-notice.txt");
    my @parts = split /\Q$separator\E/, $actual, -1;
    @parts == 2 or die "missing or repeated prior MIT notice: $file\n";
    check_text($parts[1], read_file("$root/MIT.txt"));
    $actual = $parts[0];
}
check_text($actual, read_file("$root/" . ($kind eq 'mit'
    ? 'MIT.txt' : "Dragginz-ICP-Burn-$license_version.txt")));
if (defined $archive) {
    -f $archive && !-l $archive or die "archive must be a regular non-symlink file\n";
    length($member) && $member =~ m{/} && $member !~ m{\A/|(?:\A|/)\.{1,2}(?:/|\z)|//|/\z|[\r\n\0]}
        or die "select a canonical package-relative license member\n";
    # Check the complete gzip stream too; a valid prefix must not hide a corrupt
    # trailer. No extraction, publication or filesystem output occurs.
    require File::Spec;
    my $input = File::Spec->rel2abs($archive);
    system('gzip', '-t', '--', $input) == 0 or die "archive gzip integrity failed\n";
    require Archive::Tar;
    my $tar = Archive::Tar->new;
    local $SIG{__WARN__} = sub { die "archive parsing failed: @_" };
    my $read = $tar->read($input, 1, {filter_cb => sub {
        my ($entry) = @_;
        # Leave format-extension records to the tar parser, not filesystem code.
        return 0 if $entry->is_longlink || $entry->is_label || $entry->type =~ /\A[xg]\z/;
        my $name = $entry->full_path;
        $name =~ s{/$}{} if $entry->is_dir;
        length($name) && $name !~ m{\A/|(?:\A|/)\.{1,2}(?:/|\z)|//|/\z}
            or die "noncanonical archive member\n";
        $entry->is_file || $entry->is_dir or die "nonregular archive member: $name\n";
        return 0 unless $name eq $member;
        $entry->is_file && $entry->size == length($expected)
            or die "archive license type or length differs: $member\n";
        return 1;
    }});
    defined($read) && $read == 1 && !$tar->error
        or die "expected exactly one readable license member: $member\n";
    my @entries = $tar->get_files;
    $entries[0]->get_content eq $expected
        or die "archive license bytes differ from $file\n";
    print "[OK] $archive:$member matches $file byte-for-byte\n";
}
print "[OK] $file matches canonical $kind terms\n";

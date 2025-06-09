use strict;
use Archive::Zip qw(:ERROR_CODES);

my $zip = Archive::Zip->new();
my $status = $zip->read($ARGV[0]);
die 'Failed to read ZIP' if $status != AZ_OK;
$status = $zip->extractTree('', $ARGV[1]);
die 'Failed to extract ZIP' if $status != AZ_OK;

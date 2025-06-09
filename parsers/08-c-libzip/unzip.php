<?php

$zip = new ZipArchive;

if ($zip->open($argv[1], ZipArchive::CHECKCONS) === true) {
    $zip->extractTo($argv[2]);
    $zip->close();
} else {
    exit(1);
}

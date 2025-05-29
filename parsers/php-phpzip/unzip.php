<?php

require __DIR__ . "/vendor/autoload.php";

$zipFile = new \PhpZip\ZipFile();
$zipFile->openFile($argv[1])->extractTo($argv[2]);

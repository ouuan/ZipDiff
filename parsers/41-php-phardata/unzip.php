<?php

$zip = new PharData($argv[1]);
$zip->extractTo($argv[2], null, true);

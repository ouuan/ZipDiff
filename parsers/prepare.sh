#!/bin/bash

set -euo pipefail

base="$(dirname "$(dirname "$(realpath "$0")")")"

cd "$base"
input_dir="${INPUT_DIR:-../evaluation/input}"
output_dir="${OUTPUT_DIR:-../evaluation/output}"

cd parsers
echo "services:" > docker-compose.yml

for i in */; do
    cp unzip-all.sh parallel-unzip-all.sh testcase.sh "$i"
    parser=${i%/}
    echo "  $parser:
    build: $parser
    volumes:
      - $input_dir:/input:ro
      - $output_dir/$parser:/output" >> docker-compose.yml
done

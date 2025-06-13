#!/bin/bash

set -euo pipefail

base="$(dirname "$(dirname "$(realpath "$0")")")"

"$base"/tools/prepare.sh

sudo rm -rf "$base"/evaluation/{input,output}
mkdir -p "$base/evaluation/input"

for i in $(seq 1 $#); do
    testcase="$(realpath "${!i}")"
    cp "$testcase" "$base/evaluation/input/$i.zip"
done

cd "$base/parsers"
sudo docker compose up

for i in $(seq 1 $#); do
    testcase="$(realpath "${!i}")"
    result="$base/evaluation/results/${testcase#"$base/"}"
    sudo rm -rf "$result"
    mkdir -p "$result"
    for p in "$base/parsers/"*/; do
        parser="$(basename "$p")"
        sudo mv "$base/evaluation/output/$parser/$i.zip" "$result/$parser" &
    done
done

wait

#!/bin/bash

set -euo pipefail

base="$(dirname "$(dirname "$(realpath "$0")")")"
export base

./prepare.sh

sudo rm -rf "$base/bind"
mkdir -p "$base/bind/input"

for i in $(seq 1 $#); do
    testcase="$(realpath "${!i}")"
    cp "$testcase" "$base/bind/input/$i.zip"
done

/usr/bin/time sudo docker compose up

mv_testcase() {
    set -euo pipefail
    testcase="$(realpath "$2")"
    result="$base/results/${testcase#"$base/data/"}"
    sudo rm -rf "$result"
    mkdir -p "$result"
    for p in "$base/parsers/"*/; do
        parser="$(basename "$p")"
        sudo mv "$base/bind/output/$parser/$1.zip" "$result/$parser"
    done
    sudo chown -R "$(whoami):" "$result"
}
export -f mv_testcase

for i in $(seq 1 $#); do
    printf "%s\0%s\0" "$i" "${!i}"
done | /usr/bin/time parallel -0 -n2 mv_testcase

#!/bin/bash

set -xeuo pipefail

TIMES=5
BATCH_SIZE=500
STOP_SECONDS=$(( 24 * 60 * 60 ))
base="$(dirname "$(dirname "$(realpath "$0")")")"
DATA="$base/evaluation"

for _ in $(seq 1 $TIMES); do
    for i in full argmax-ucb byte-only; do
        cd "$base/zip-diff"
        case "$i" in
            full) arg= ;;
            argmax-ucb) arg=--argmax-ucb ;;
            byte-only) arg=--byte-mutation-only ;;
        esac
        key="$(date -Is)-$i"
        session="$DATA/sessions/$key"
        target/release/fuzz -b "$BATCH_SIZE" -s "$STOP_SECONDS" $arg \
            --input-dir "$DATA/bind/input" \
            --output-dir "$DATA/bind/output" \
            --samples-dir "$session/samples" \
            --results-dir "$session/results" \
            --stats-file "$DATA/stats/$key.json"
        cd ../parsers
        docker compose down
    done
done

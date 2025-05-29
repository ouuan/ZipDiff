#!/bin/sh

set -eu

cd /input

for i in *; do
    mkdir -p /output/"$i"
    if ! timeout 1m /unzip "$(realpath "$i")" /output/"$i"; then
        while ! rm -rf /output/"$i"; do echo "Failed to rm -rf /output/$i"; done
        touch /output/"$i"
    fi
done

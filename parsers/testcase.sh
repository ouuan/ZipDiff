#!/bin/sh

set -eu

mkdir -p /output/"$1"
if ! timeout 1m /unzip "$(realpath "$1")" /output/"$1"; then
    while ! rm -rf /output/"$1"; do echo "Failed to rm -rf /output/$1"; done
    touch /output/"$1"
fi

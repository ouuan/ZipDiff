#!/bin/sh

export WINEDEBUG=-all
xvfb-run -a /parallel-unzip-all.sh 50%

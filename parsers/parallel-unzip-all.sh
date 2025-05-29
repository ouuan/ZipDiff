#!/bin/sh

cd /input && parallel -j"${1:-25%}" /testcase.sh ::: *

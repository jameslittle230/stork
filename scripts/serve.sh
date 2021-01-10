#!/usr/bin/env bash
set -eux

python3 -m http.server --directory ./dist 8025 2> /dev/null &
sleep 1
open http://127.0.0.1:8025
#!/usr/bin/env bash
# corpora is the plural of corpus
set -eux

__dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

rm -rf "${__dir}/../test/federalist"
git clone git@github.com:jameslittle230/federalist.git "${__dir}/../test/federalist"

# And do the same for 3b1b, eventually
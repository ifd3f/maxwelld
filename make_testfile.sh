#!/usr/bin/env bash
set -euxo pipefail
dd bs=$1 if=/dev/urandom of=testfile count=1
cp testfile testfile.orig
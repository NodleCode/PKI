#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

if [ -z $CI_PROJECT_NAME ] ; then
   rustup update nightly
   rustup update stable
fi

rustup default stable
rustup uninstall nightly
rustup toolchain install nightly-2020-06-01
rustup target add wasm32-unknown-unknown --toolchain nightly-2020-06-01
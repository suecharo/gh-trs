#!/bin/bash
set -euxC

GIT_ROOT=$(git rev-parse --show-toplevel)
cd ${GIT_ROOT}
docker run --rm -it -v $(pwd):/workdir -w /workdir ekidd/rust-musl-builder cargo build --release

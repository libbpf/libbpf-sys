#!/bin/bash

DOCKER="docker"

set -eu -o pipefail

${DOCKER} build -t libbpf-sys-builder - <<'EOF'
FROM amd64/ubuntu:focal AS libbpf-sys-builder

ENV LANG=C.UTF-8 \
    LC_ALL=C.UTF-8

VOLUME /usr/local/src/libbpf-sys
WORKDIR /usr/local/src/libbpf-sys

SHELL ["/bin/bash", "-eu", "-o", "pipefail", "-c"]

RUN \
	export DEBIAN_FRONTEND=noninteractive; \
	apt-get -q update; \
	apt-get -q install -y curl build-essential linux-headers-generic zlib1g-dev libelf-dev libclang-dev llvm clang pkg-config; \
	apt-get -q clean autoclean;

RUN \
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable;

ENTRYPOINT \
	source $HOME/.cargo/env; \
	cargo build --features bindgen --release --verbose;
EOF

${DOCKER} run --rm -v "$(pwd):/usr/local/src/libbpf-sys" libbpf-sys-builder

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

RUN \
	source $HOME/.cargo/env; \
	cargo install bindgen;

ENTRYPOINT \
	source $HOME/.cargo/env; \
	bindgen \
		--verbose \
		--with-derive-default \
		--whitelist-function "bpf_.+" \
		--whitelist-function "btf_.+" \
		--whitelist-function "libbpf_.+" \
		--whitelist-function "xsk_.+" \
		--whitelist-function "_xsk_.+" \
		--whitelist-function "perf_buffer_.+" \
		--whitelist-function "ring_buffer_.+" \
		--whitelist-type "bpf_.*" \
		--whitelist-type "xdp_.*" \
		--whitelist-type "xsk_.*" \
		--whitelist-var "BPF_.+" \
		--whitelist-var "BTF_.+" \
		--whitelist-var "XSK_.+" \
		--whitelist-var "XDP_.+" \
		--default-enum-style consts \
		--no-prepend-enum-name \
		--no-layout-tests \
		--no-doc-comments \
		--builtins \
		--output $(pwd)/src/bindings.rs \
		$(pwd)/bindings.h \
		-- \
		-I$(pwd)/libbpf/include \
		-I$(pwd)/libbpf/include/uapi; \
	cargo build --release --verbose;
EOF

${DOCKER} run --rm -v "$(pwd):/usr/local/src/libbpf-sys" libbpf-sys-builder

#!/bin/bash

set -eu -o pipefail

docker build -t libbpf-sys-builder - <<'EOF'
FROM amd64/ubuntu:bionic AS libbpf-sys-builder

ENV LANG=C.UTF-8
ENV LC_ALL C.UTF-8

VOLUME /usr/local/src/libbpf-sys
WORKDIR /usr/local/src/libbpf-sys

SHELL ["/bin/bash", "-e", "-u", "-o", "pipefail", "-c"]

RUN \
	DEBIAN_FRONTEND=noninteractive apt-get -q update; \
	DEBIAN_FRONTEND=noninteractive apt-get -q install -y curl build-essential linux-headers-generic zlib1g-dev libelf-dev libclang-dev llvm clang; \
	DEBIAN_FRONTEND=noninteractive apt-get -q clean autoclean;

RUN \
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable; \
	source $HOME/.cargo/env; \
	cargo install bindgen;

ENTRYPOINT \
	source $HOME/.cargo/env; \
	bindgen \
		--verbose \
		--whitelist-function "bpf_.+" \
		--whitelist-function "btf_.+" \
		--whitelist-function "libbpf_.+" \
		--whitelist-function "xsk_.+" \
		--whitelist-function "perf_buffer_.+" \
		--whitelist-var "BPF_.+" \
		--whitelist-var "BTF_.+" \
		--whitelist-var "XSK_.+" \
		--default-enum-style consts \
		--no-prepend-enum-name \
		--no-layout-tests \
		--builtins \
		--output $(pwd)/src/bindings.rs \
		$(pwd)/bindings.h \
		-- \
		-I$(pwd)/libbpf/include \
		-I$(pwd)/libbpf/include/uapi; \
	cargo build --release;
EOF

docker run --rm -v "$(pwd):/usr/local/src/libbpf-sys" libbpf-sys-builder
